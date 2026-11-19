//! La prueba de voz de H3a, que cierra D2: pasa las 50 frases grabadas por
//! Whisper y compara con lo que dieron los otros motores.
//!
//!     cargo run --release --example probar_voz [-- <carpeta>] [--informe] [--solo-recorte]
//!
//! - Sin `--informe`: por cada modelo `ggml-*.bin` de
//!   `%APPDATA%\es.gestornotas.app\modelos`, transcribe todas las frases
//!   cuatro veces —sin contexto o con la lista de la clase; con la ventana
//!   entera de 30 s o recortada a lo que dura la frase— y deja lo que oyó,
//!   con cuánto tardó, en
//!   `<carpeta>\resultados\whisper-<modelo>[+lista][+recorte].json`.
//!   `--solo-recorte` se salta la ventana entera, que es lenta.
//! - Luego (o solo eso, con `--informe`) junta **todos** los
//!   `resultados\*.json` —también `windows*.json` y `google.json`, que dejan
//!   los scripts de `scripts/probar-voz/`— y dice, por motor, cuántas notas y
//!   cuántos nombres salen bien y cuánto tarda. El detalle, frase a frase, en
//!   `resultados\informe.txt`.
//!
//! **Cómo se puntúa** (provisional: lo de verdad es H3b): la **nota** está
//! bien si el último número de la frase, leído en cifras o en palabras («siete
//! con cinco», «7,5», «nueve y medio»), es la nota; el **nombre** está bien si
//! cada palabra dicha del nombre aparece en lo oído, comparando como suena
//! (b/v, ll/y, h muda, c/z/s…) y aceptando una letra de cada cuatro mal: lo
//! que el emparejamiento con la lista de la clase podrá arreglar en H3b.

#[path = "comun/corpus.rs"]
mod corpus;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use gestor_de_notas_lib::voz::whisper::Whisper;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Resultado {
    motor: String,
    frases: Vec<Oida>,
}

#[derive(Serialize, Deserialize)]
struct Oida {
    n: u32,
    texto: String,
    /// Desde que se tiene el audio hasta que se tiene el texto.
    ms: u64,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let solo_informe = args.iter().any(|a| a == "--informe");
    let carpeta = corpus::carpeta(args.iter().find(|a| !a.starts_with("--")).map(String::as_str));
    let frases = corpus::frases();

    // El corpus tiene que entenderse a sí mismo; si no, la puntuación miente.
    for f in &frases {
        assert_eq!(nota(&f.frase), Some(f.nota), "la frase {} no da su nota", f.n);
        assert!(nombre_reconocible(&f.frase, &f.dicho), "la frase {} no da su nombre", f.n);
    }

    let resultados = carpeta.join("resultados");
    std::fs::create_dir_all(&resultados).expect("no se puede crear resultados/");
    if !solo_informe {
        probar_whisper(&carpeta, &resultados, &frases, args.iter().any(|a| a == "--solo-recorte"));
    }
    informe(&resultados, &frases);
}

fn carpeta_modelos() -> PathBuf {
    dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app").join("modelos")
}

fn probar_whisper(carpeta: &Path, resultados: &Path, frases: &[corpus::Frase], solo_recorte: bool) {
    let audios: Vec<(u32, Vec<f32>)> = frases
        .iter()
        .filter_map(|f| leer_wav(&corpus::wav(carpeta, f.n)).map(|a| (f.n, a)))
        .collect();
    if audios.is_empty() {
        println!("No hay grabaciones en {}: primero `cargo run --release --example grabar_frases`.", carpeta.display());
        return;
    }
    let modelos_dir = carpeta_modelos();
    let mut modelos: Vec<PathBuf> = std::fs::read_dir(&modelos_dir)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();
    modelos.retain(|p| {
        let n = p.file_name().unwrap().to_string_lossy();
        n.starts_with("ggml-") && n.ends_with(".bin")
    });
    modelos.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0));
    if modelos.is_empty() {
        println!("No hay modelos de Whisper en {}.", modelos_dir.display());
        return;
    }

    // Whisper copia el estilo del contexto: con solo la lista de nombres,
    // escribe a veces las notas en números romanos («V», «X10»). Con un
    // ejemplo de notas dictadas, las escribe como se dicen.
    let lista = format!(
        "Dictado de notas: siete con cinco, nueve y medio, ocho coma veinticinco, un diez. Alumnos de la clase: {}.",
        corpus::clase().join(", ")
    );
    for modelo in modelos {
        let nombre = modelo.file_stem().unwrap().to_string_lossy().trim_start_matches("ggml-").to_string();
        let mb = std::fs::metadata(&modelo).map(|m| m.len() / 1_000_000).unwrap_or(0);
        let t = Instant::now();
        let mut whisper = match Whisper::cargar(&modelo) {
            Ok(w) => w,
            Err(e) => {
                println!("{nombre}: {e}");
                continue;
            }
        };
        println!("{nombre} ({mb} MB): cargado en {} ms", t.elapsed().as_millis());
        // La primera transcripción tarda más (reserva memoria): no se cuenta.
        let _ = whisper.transcribir(&audios[0].1, None);

        let variantes = [
            ("", None, false),
            ("+lista", Some(lista.as_str()), false),
            ("+recorte", None, true),
            ("+lista+recorte", Some(lista.as_str()), true),
        ];
        for (sufijo, contexto, recortar) in variantes {
            let motor = format!("whisper-{nombre}{sufijo}");
            if solo_recorte && !recortar {
                continue;
            }
            whisper.recortar = recortar;
            let mut oidas = Vec::new();
            for (n, audio) in &audios {
                let t = Instant::now();
                let texto = whisper.transcribir(audio, contexto).unwrap_or_else(|e| format!("[error: {e}]"));
                oidas.push(Oida { n: *n, texto, ms: t.elapsed().as_millis() as u64 });
                print!("\r  {motor}: {}/{}", oidas.len(), audios.len());
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
            println!();
            let r = Resultado { motor: motor.clone(), frases: oidas };
            let json = serde_json::to_string_pretty(&r).unwrap();
            std::fs::write(resultados.join(format!("{motor}.json")), json).expect("no se puede escribir el resultado");
        }
    }
}

fn leer_wav(ruta: &Path) -> Option<Vec<f32>> {
    let mut r = hound::WavReader::open(ruta).ok()?;
    let spec = r.spec();
    assert_eq!((spec.channels, spec.sample_rate), (1, 16_000), "{}: no es mono a 16 kHz", ruta.display());
    Some(r.samples::<i16>().filter_map(Result::ok).map(|m| m as f32 / i16::MAX as f32).collect())
}

// ── El informe ──────────────────────────────────────────────────────────

fn informe(resultados: &Path, frases: &[corpus::Frase]) {
    let mut motores: Vec<Resultado> = std::fs::read_dir(resultados)
        .expect("no se puede leer resultados/")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| {
            // Los scripts de PowerShell pueden dejar el BOM de UTF-8.
            let texto = std::fs::read_to_string(&p).ok()?;
            serde_json::from_str(texto.trim_start_matches('\u{feff}'))
                .map_err(|e| println!("{}: no se entiende ({e})", p.display()))
                .ok()
        })
        .collect();
    motores.sort_by(|a, b| a.motor.cmp(&b.motor));
    if motores.is_empty() {
        println!("Aún no hay resultados en {}.", resultados.display());
        return;
    }

    let por_n: BTreeMap<u32, &corpus::Frase> = frases.iter().map(|f| (f.n, f)).collect();
    let ancho = motores.iter().map(|m| m.motor.chars().count()).max().unwrap().max(5);
    let mut tabla = format!(
        "{:ancho$}  frases  nota   nombre  las dos  mediana   p90     máx\n",
        "motor"
    );
    let mut detalle = String::new();
    for m in &motores {
        let (mut bien_nota, mut bien_nombre, mut bien_las_dos) = (0, 0, 0);
        let mut ms: Vec<u64> = Vec::new();
        for o in &m.frases {
            let Some(f) = por_n.get(&o.n) else { continue };
            let a = nota(&o.texto) == Some(f.nota);
            let b = nombre_reconocible(&o.texto, &f.dicho);
            bien_nota += a as u32;
            bien_nombre += b as u32;
            bien_las_dos += (a && b) as u32;
            ms.push(o.ms);
        }
        ms.sort();
        let total = ms.len();
        let pct = |x: u32| if total == 0 { 0.0 } else { 100.0 * x as f64 / total as f64 };
        let q = |p: f64| if total == 0 { 0 } else { ms[((total - 1) as f64 * p).round() as usize] };
        writeln!(
            tabla,
            "{:ancho$}  {:>6}  {:>4.0} %  {:>4.0} %  {:>5.0} %  {:>5} ms {:>5} ms {:>5} ms",
            m.motor,
            total,
            pct(bien_nota),
            pct(bien_nombre),
            pct(bien_las_dos),
            q(0.5),
            q(0.9),
            ms.last().copied().unwrap_or(0)
        )
        .unwrap();
    }

    for f in frases {
        writeln!(detalle, "\n{:02} · {}", f.n, f.frase).unwrap();
        for m in &motores {
            if let Some(o) = m.frases.iter().find(|o| o.n == f.n) {
                let marca = |ok: bool| if ok { '✓' } else { '✗' };
                writeln!(
                    detalle,
                    "   {}{} {:ancho$} {:>5} ms  {}",
                    marca(nota(&o.texto) == Some(f.nota)),
                    marca(nombre_reconocible(&o.texto, &f.dicho)),
                    m.motor,
                    o.ms,
                    o.texto
                )
                .unwrap();
            }
        }
    }

    print!("\n{tabla}");
    let ruta = resultados.join("informe.txt");
    std::fs::write(&ruta, format!("{tabla}\nnota · nombre, frase a frase:\n{detalle}")).ok();
    println!("\nFrase a frase: {}", ruta.display());
}

// ── Puntuar (provisional, hasta H3b) ────────────────────────────────────

fn sin_acentos(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 's',
            otro => otro,
        })
        .collect()
}

/// Las palabras de `texto`, en minúscula y sin acentos; `7,5` y `7'5` pasan a
/// `7.5`, y el resto de la puntuación se quita.
fn palabras(texto: &str) -> Vec<String> {
    let c: Vec<char> = sin_acentos(texto).chars().collect();
    let mut limpio = String::new();
    for (i, &ch) in c.iter().enumerate() {
        let entre_cifras = i > 0 && i + 1 < c.len() && c[i - 1].is_ascii_digit() && c[i + 1].is_ascii_digit();
        match ch {
            // «5-75»: Whisper lo escribe así a veces.
            ',' | '.' | '\'' | '-' if entre_cifras => limpio.push('.'),
            ch if ch.is_alphanumeric() => limpio.push(ch),
            _ => limpio.push(' '),
        }
    }
    limpio.split_whitespace().map(String::from).collect()
}

fn numero(p: &str) -> Option<f64> {
    // Solo cifras: `parse` también acepta «nan» o «inf».
    if p.starts_with(|c: char| c.is_ascii_digit()) {
        return p.parse().ok();
    }
    const UNIDADES: [&str; 30] = [
        "cero", "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez", "once", "doce",
        "trece", "catorce", "quince", "dieciseis", "diecisiete", "dieciocho", "diecinueve", "veinte", "veintiuno",
        "veintidos", "veintitres", "veinticuatro", "veinticinco", "veintiseis", "veintisiete", "veintiocho",
        "veintinueve",
    ];
    const DECENAS: [(&str, f64); 7] = [
        ("treinta", 30.0),
        ("cuarenta", 40.0),
        ("cincuenta", 50.0),
        ("sesenta", 60.0),
        ("setenta", 70.0),
        ("ochenta", 80.0),
        ("noventa", 90.0),
    ];
    UNIDADES
        .iter()
        .position(|u| *u == p)
        .map(|i| i as f64)
        .or_else(|| DECENAS.iter().find(|(d, _)| *d == p).map(|(_, v)| *v))
}

/// La nota de la frase: el último número, con su parte decimal si la lleva
/// («con», «coma», «punto», «y medio»).
fn nota(texto: &str) -> Option<f64> {
    let p = palabras(texto);
    // Primero, «setenta y cinco» → 75.
    let mut nums: Vec<Option<f64>> = Vec::new();
    let mut i = 0;
    while i < p.len() {
        let n = numero(&p[i]);
        if let (Some(d), Some(y), Some(u)) = (n, p.get(i + 1), p.get(i + 2).and_then(|u| numero(u))) {
            if d >= 30.0 && d % 10.0 == 0.0 && y == "y" && (1.0..10.0).contains(&u) {
                nums.push(Some(d + u));
                nums.push(None);
                nums.push(None);
                i += 3;
                continue;
            }
        }
        nums.push(n);
        i += 1;
    }
    let mut ultima = None;
    let mut i = 0;
    while i < p.len() {
        let Some(entera) = nums[i] else {
            i += 1;
            continue;
        };
        let separador = p.get(i + 1).map(String::as_str);
        let siguiente = nums.get(i + 2).copied().flatten();
        let media = p.get(i + 2).is_some_and(|m| m == "medio" || m == "media");
        match (separador, siguiente) {
            (Some("con" | "coma" | "punto"), Some(d)) if entera.fract() == 0.0 && d.fract() == 0.0 => {
                ultima = Some(entera + if d < 10.0 { d / 10.0 } else { d / 100.0 });
                i += 3;
            }
            (Some("y"), _) if media => {
                ultima = Some(entera + 0.5);
                i += 3;
            }
            _ => {
                ultima = Some(entera);
                i += 1;
            }
        }
    }
    ultima.map(|n| (n * 100.0).round() / 100.0)
}

/// Cómo suena una palabra, para comparar nombres escritos de otra manera.
fn fonetica(palabra: &str) -> String {
    let mut s = sin_acentos(palabra);
    for (de, a) in [
        ("ch", "\u{1}"),
        ("tx", "\u{1}"),
        ("ll", "y"),
        ("h", ""),
        ("qu", "k"),
        ("ce", "se"),
        ("ci", "si"),
        ("c", "k"),
        ("\u{1}", "ch"),
        ("v", "b"),
        ("z", "s"),
        ("ge", "je"),
        ("gi", "ji"),
        ("y", "i"),
    ] {
        s = s.replace(de, a);
    }
    let mut sin_dobles = String::new();
    for c in s.chars() {
        if !sin_dobles.ends_with(c) {
            sin_dobles.push(c);
        }
    }
    sin_dobles
}

fn distancia(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut fila: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut diagonal = fila[0];
        fila[0] = i;
        for j in 1..=b.len() {
            let arriba = fila[j];
            fila[j] = (fila[j] + 1).min(fila[j - 1] + 1).min(diagonal + (a[i - 1] != b[j - 1]) as usize);
            diagonal = arriba;
        }
    }
    fila[b.len()]
}

fn parecida(dicha: &str, oida: &str) -> bool {
    dicha == oida || (dicha.chars().count() >= 4 && distancia(dicha, oida) * 4 <= dicha.chars().count())
}

/// Cada palabra de `dicho` está en `texto`, como suena. Si el motor junta o
/// separa palabras («Juanpablo», «Etxe Berria»), se compara todo junto.
fn nombre_reconocible(texto: &str, dicho: &str) -> bool {
    let oidas: Vec<String> = palabras(texto).iter().map(|p| fonetica(p)).collect();
    let dichas: Vec<String> = palabras(dicho).iter().map(|p| fonetica(p)).collect();
    if dichas.iter().all(|d| oidas.iter().any(|o| parecida(d, o))) {
        return true;
    }
    let junto = dichas.concat();
    let n = dichas.len().max(1);
    (0..oidas.len()).any(|i| (i + 1..=oidas.len().min(i + n + 1)).any(|j| parecida(&junto, &oidas[i..j].concat())))
}
