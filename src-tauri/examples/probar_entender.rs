//! La prueba de H3b: ¿entiende la app las 100 frases grabadas en H3a?
//!
//!     cargo run --release --example probar_entender [-- <carpeta>…] [--de-nuevo] [--todos]
//!
//! Por cada carpeta de grabaciones (por defecto, las dos de H3a:
//! `Documentos\Gestor de Notas\prueba-voz` y su `mesa`), pasa las 50 frases
//! por Whisper (`small` `q5_1` recortado, lo de D2) y guarda lo oído, con la
//! confianza de cada letra, en `resultados\oidos.json`; la siguiente vez lo
//! lee de ahí (`--de-nuevo` vuelve a transcribir). Luego entiende cada frase
//! con la clase inventada del corpus y cuenta:
//!
//! - **apuntada**: la app está segura, y alumno y nota son los dichos;
//! - **preguntada**: duda, y lo dicho está entre lo que ofrece; o duda y no
//!   está (se cuenta aparte: el profesor tendrá que corregirla a mano);
//! - **no entendida**: lo dice, y no apunta nada;
//! - **MAL EN SILENCIO**: segura, y alumno o nota no son los dichos. Es la
//!   que no puede pasar.
//!
//! El detalle, frase a frase, en `prueba-voz\entender.txt`.
//!
//! `--todos` entiende además lo que dejó H3a en `resultados\` y
//! `resultados\experimentos\`: los otros motores (Google, Windows) y las
//! variantes de Whisper, con sus «825», «7.30» y bucles. Es para ver que la
//! regla aguanta con más que una pasada.

#[path = "comun/corpus.rs"]
mod corpus;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use gestor_de_notas_lib::voz::entender::{entender, Entendido};
use gestor_de_notas_lib::voz::whisper::{Oido, Whisper};
use serde::{Deserialize, Serialize};

const MODELO: &str = "ggml-small-q5_1.bin";

#[derive(Serialize, Deserialize)]
struct Grabada {
    n: u32,
    oido: Oido,
    ms: u64,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let de_nuevo = args.iter().any(|a| a == "--de-nuevo");
    let mut carpetas: Vec<PathBuf> = args.iter().filter(|a| !a.starts_with("--")).map(PathBuf::from).collect();
    if carpetas.is_empty() {
        let base = corpus::carpeta(None);
        carpetas = vec![base.join("mesa"), base];
    }
    let frases = corpus::frases();
    let clase = corpus::clase();

    let mut whisper: Option<Whisper> = None;
    let mut totales = Cuenta::default();
    let mut detalle = String::new();
    let mut tabla = String::new();
    for carpeta in &carpetas {
        let grabadas = oidos(carpeta, &frases, de_nuevo, &mut whisper);
        if grabadas.is_empty() {
            println!("{}: no hay grabaciones.", carpeta.display());
            continue;
        }
        let mut cuenta = Cuenta::default();
        writeln!(detalle, "\n══ {}", carpeta.display()).unwrap();
        for g in &grabadas {
            let Some(f) = frases.iter().find(|f| f.n == g.n) else {
                continue;
            };
            let entendido = entender(&g.oido.texto, &clase);
            let veredicto = cuenta.apuntar(&entendido, &clase, &f.alumno, f.nota);
            writeln!(detalle, "\n{:02} · {}  →  {} · {}", f.n, f.frase, f.alumno, f.nota).unwrap();
            writeln!(detalle, "     oído: {}", con_confianza(&g.oido)).unwrap();
            writeln!(detalle, "     {veredicto}: {}", describir(&entendido, &clase)).unwrap();
        }
        writeln!(tabla, "{:50} {}", carpeta.display().to_string(), cuenta).unwrap();
        totales.sumar(&cuenta);
    }
    writeln!(tabla, "{:50} {}", "total", totales).unwrap();
    if args.iter().any(|a| a == "--todos") {
        writeln!(tabla, "\nLo que dejó H3a, por motor:").unwrap();
        for carpeta in &carpetas {
            for sub in ["resultados", "resultados/experimentos"] {
                otros_motores(&carpeta.join(sub), &frases, &clase, &mut tabla, &mut detalle);
            }
        }
    }
    print!("\n{tabla}");
    let ruta = carpetas[0].parent().unwrap_or(&carpetas[0]).join("entender.txt");
    std::fs::write(&ruta, format!("{tabla}{detalle}")).ok();
    println!("\nFrase a frase: {}", ruta.display());
}

#[derive(Deserialize)]
struct Resultado {
    motor: String,
    frases: Vec<Transcrita>,
}

#[derive(Deserialize)]
struct Transcrita {
    n: u32,
    texto: String,
}

/// Entiende los `*.json` de resultados de `probar_voz` y de los scripts.
fn otros_motores(dir: &Path, frases: &[corpus::Frase], clase: &[String], tabla: &mut String, detalle: &mut String) {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return;
    };
    let mut rutas: Vec<PathBuf> = entradas.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    rutas.sort();
    let sitio = dir.ancestors().find(|p| p.file_name().is_some_and(|n| n != "resultados" && n != "experimentos"));
    let sitio = sitio.and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    for ruta in rutas {
        // Los scripts de PowerShell pueden dejar el BOM de UTF-8.
        let Some(r) = std::fs::read_to_string(&ruta)
            .ok()
            .and_then(|t| serde_json::from_str::<Resultado>(t.trim_start_matches('\u{feff}')).ok())
        else {
            continue;
        };
        let mut cuenta = Cuenta::default();
        for t in &r.frases {
            let Some(f) = frases.iter().find(|f| f.n == t.n) else {
                continue;
            };
            let entendido = entender(&t.texto, clase);
            if cuenta.apuntar(&entendido, clase, &f.alumno, f.nota) == "MAL EN SILENCIO" {
                let que = describir(&entendido, clase);
                writeln!(
                    detalle,
                    "\nMAL EN SILENCIO · {sitio} · {} · {:02} · {} → «{}»: {que}",
                    r.motor, f.n, f.frase, t.texto
                )
                .unwrap();
            }
        }
        writeln!(tabla, "{:50} {}", format!("{sitio} · {}", r.motor), cuenta).unwrap();
    }
}

/// Lo oído en una carpeta: de `resultados\oidos.json`, o transcribiendo.
fn oidos(carpeta: &Path, frases: &[corpus::Frase], de_nuevo: bool, whisper: &mut Option<Whisper>) -> Vec<Grabada> {
    let cache = carpeta.join("resultados").join("oidos.json");
    if !de_nuevo {
        if let Some(g) = std::fs::read_to_string(&cache).ok().and_then(|t| serde_json::from_str(&t).ok()) {
            return g;
        }
    }
    let audios: Vec<(u32, Vec<f32>)> =
        frases.iter().filter_map(|f| corpus::leer_wav(&corpus::wav(carpeta, f.n)).map(|a| (f.n, a))).collect();
    if audios.is_empty() {
        return Vec::new();
    }
    let w = whisper.get_or_insert_with(|| {
        let modelo =
            dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app/modelos").join(MODELO);
        let mut w = Whisper::cargar(&modelo).expect("no se puede cargar el modelo");
        // La primera transcripción tarda más (reserva memoria): no se cuenta.
        let _ = w.transcribir(&audios[0].1, None);
        w
    });
    let mut grabadas = Vec::new();
    for (n, audio) in &audios {
        let t = Instant::now();
        let oido = w
            .transcribir(audio, None)
            .unwrap_or_else(|e| Oido { texto: format!("[error: {e}]"), ..Default::default() });
        grabadas.push(Grabada { n: *n, oido, ms: t.elapsed().as_millis() as u64 });
        print!("\r{}: {}/{}", carpeta.display(), grabadas.len(), audios.len());
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
    println!();
    std::fs::create_dir_all(cache.parent().unwrap()).ok();
    std::fs::write(&cache, serde_json::to_string_pretty(&grabadas).unwrap()).expect("no se puede guardar lo oído");
    grabadas
}

/// El texto oído con la confianza de cada palabra: «Lucia(98) Martínez(41)».
fn con_confianza(o: &Oido) -> String {
    let mut s = String::new();
    let letras: Vec<(char, f32)> = o.texto.chars().zip(o.confianza.iter().copied()).collect();
    for palabra in letras.split(|(c, _)| c.is_whitespace()).filter(|p| !p.is_empty()) {
        let texto: String = palabra.iter().map(|(c, _)| c).collect();
        let p = palabra.iter().map(|(_, p)| *p).fold(1.0, f32::min);
        write!(s, "{texto}({:.0}) ", p * 100.0).unwrap();
    }
    s.trim_end().into()
}

fn describir(e: &Entendido, clase: &[String]) -> String {
    match e {
        Entendido::Seguro { alumno, nota } => format!("{} · {nota}", clase[*alumno]),
        Entendido::Duda { alumnos, notas, motivos } => {
            let a: Vec<&str> = alumnos.iter().map(|a| clase[*a].as_str()).collect();
            format!("¿{}? · ¿{notas:?}? ({motivos:?})", a.join(" o "))
        }
        Entendido::NoEntendido { motivo } => format!("{motivo:?}"),
    }
}

#[derive(Default)]
struct Cuenta {
    total: u32,
    apuntadas: u32,
    preguntadas: u32,
    preguntadas_sin_lo_dicho: u32,
    no_entendidas: u32,
    mal_en_silencio: u32,
}

impl Cuenta {
    fn apuntar(&mut self, e: &Entendido, clase: &[String], alumno: &str, nota: f64) -> &'static str {
        self.total += 1;
        match e {
            Entendido::Seguro { alumno: a, nota: n } if clase[*a] == alumno && *n == nota => {
                self.apuntadas += 1;
                "apuntada"
            }
            Entendido::Seguro { .. } => {
                self.mal_en_silencio += 1;
                "MAL EN SILENCIO"
            }
            Entendido::Duda { alumnos, notas, .. } => {
                if alumnos.iter().any(|a| clase[*a] == alumno) && notas.contains(&nota) {
                    self.preguntadas += 1;
                    "preguntada"
                } else {
                    self.preguntadas_sin_lo_dicho += 1;
                    "preguntada, sin lo dicho"
                }
            }
            Entendido::NoEntendido { .. } => {
                self.no_entendidas += 1;
                "no entendida"
            }
        }
    }

    fn sumar(&mut self, o: &Cuenta) {
        self.total += o.total;
        self.apuntadas += o.apuntadas;
        self.preguntadas += o.preguntadas;
        self.preguntadas_sin_lo_dicho += o.preguntadas_sin_lo_dicho;
        self.no_entendidas += o.no_entendidas;
        self.mal_en_silencio += o.mal_en_silencio;
    }
}

impl std::fmt::Display for Cuenta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:>3} frases · {:>3} apuntadas · {:>3} preguntadas (+{} sin lo dicho) · {:>2} no entendidas · {} MAL EN SILENCIO",
            self.total,
            self.apuntadas,
            self.preguntadas,
            self.preguntadas_sin_lo_dicho,
            self.no_entendidas,
            self.mal_en_silencio
        )
    }
}
