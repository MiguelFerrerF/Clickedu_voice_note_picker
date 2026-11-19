//! El intérprete (H3b): de lo que escribió Whisper a **la nota** y **las
//! palabras del nombre**. Quién es el alumno lo decide [`super::emparejar`].
//!
//! Porta `_extract_info` de la v1 (`voice_processor.py`), arreglando lo que
//! hacía mal y añadiendo lo que Whisper escribe y Google no, visto en las 100
//! frases grabadas de H3a:
//!
//! - **La nota, dicha o escrita**: «siete con cinco», «7,5», «nueve y medio»,
//!   «9 y medio», «ocho coma veinticinco», «cinco con 25», «siete coma
//!   setenta y cinco», «7,50». La v1 cambiaba «un» por 1 y luego cogía el
//!   último número: «un diez» le daba 10 por suerte.
//! - **«un» y «una» delante de un número son artículo** («un diez», «ponle
//!   un cuatro a…»), no un 1.
//! - **Bucles**: Whisper repite a veces la frase hasta cortarse («Pablo
//!   García, un 10. Pablo García, un 10. Pablo…»). Se queda la primera.
//! - **Lo que se puede leer de dos maneras no se decide en silencio**:
//!   «825» (ocho con veinticinco, escrito sin coma) y «5x25» se entienden
//!   pero quedan [`dudosa`](Nota::dudosa); «7.30» puede ser «siete y media»
//!   escrito como una hora, así que se ofrecen 7,5 y 7,3.
//! - Las **palabras de relleno** («ponle», «para», «tiene»…) se quitan solo
//!   en los extremos del nombre: «Mohamed El Amrani» lleva un «el» dentro.

/// Lo que se saca de una frase.
#[derive(Debug, Clone, PartialEq)]
pub struct Frase {
    /// La nota, si se oyó alguna.
    pub nota: Option<Nota>,
    /// Las palabras que quedan para el nombre, en minúscula y sin acentos.
    pub nombre: Vec<String>,
    /// La nota se dijo de una forma que no se interpreta ([`NOTA_RARA`]).
    pub nota_rara: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nota {
    /// Las lecturas posibles, la más probable primero. Todas son notas
    /// válidas: de 0 a 10, con dos decimales como mucho.
    pub valores: Vec<f64>,
    /// Por qué no se puede apuntar sin preguntar.
    pub dudosa: Option<PorQueDudosa>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PorQueDudosa {
    /// «825», «5x25», «7.30»: escrita de una forma que hay que reinterpretar.
    Reinterpretada,
    /// Se oyeron dos notas distintas.
    Varias,
    /// Además de la nota, un número que no lo es («ochenta», «con cinco»
    /// sin nada delante): algo no se ha entendido.
    NumeroDeMas,
}

/// Maneras de decir una nota que **no se interpretan**: «siete y cuarto»,
/// «seis menos», «notable». No salieron en las grabaciones de H3a, así que
/// no se adivinan (el usuario, 2026-11-19): la frase no se apunta y se dice
/// por qué. Si salen en casos reales (D8), se verá cómo leerlas. «Mas» no
/// está: es un apellido catalán.
pub const NOTA_RARA: &[&str] =
    &["cuarto", "cuartos", "menos", "notable", "sobresaliente", "aprobado", "suspenso", "suficiente", "insuficiente"];

/// Palabras que no son del nombre cuando están en sus extremos.
const RELLENO: &[&str] = &[
    "ponle", "ponme", "pon", "poner", "ponerle", "ponedle", "a", "al", "para", "nota", "la", "de", "del", "tiene",
    "saca", "saco", "sacado", "ha", "un", "una", "y", "con", "que", "es", "le", "lo", "se", "alumno", "alumna", "vale",
    "bueno", "eh", "em", "mm",
];

/// Si `palabra` es de dictar una nota y no de un nombre: un número dicho,
/// «con», «coma», «y medio» o el relleno. Lo demás de una frase puede ser
/// un nombre, y el informe del registro (D8) lo cambia por su forma antes
/// de que salga del ordenador.
pub fn es_de_dictar(palabra: &str) -> bool {
    let p = normalizar(palabra);
    numero_dicho(&p).is_some()
        || RELLENO.contains(&p.as_str())
        || NOTA_RARA.contains(&p.as_str())
        || matches!(p.as_str(), "coma" | "punto" | "medio" | "media" | "cien")
}

pub fn interpretar(texto: &str) -> Frase {
    let trozos = sin_bucles(trocear(texto));
    let nota_rara = trozos.iter().any(|t| NOTA_RARA.contains(&t.as_str()));
    let elementos = leer_numeros(&trozos);
    let (notas, usados) = buscar_notas(&elementos);
    let sobra_un_numero = elementos
        .iter()
        .zip(&usados)
        .any(|(e, usado)| !usado && matches!(e, Elemento::Entero { .. } | Elemento::Decimal(_)));

    // El nombre: lo que no es nota, por tramos seguidos, sin el relleno de
    // los extremos de cada tramo.
    let mut nombre = Vec::new();
    let mut tramo: Vec<String> = Vec::new();
    let cerrar = |tramo: &mut Vec<String>, nombre: &mut Vec<String>| {
        let ini = tramo.iter().position(|p| !RELLENO.contains(&p.as_str())).unwrap_or(tramo.len());
        let fin = tramo.iter().rposition(|p| !RELLENO.contains(&p.as_str())).map_or(ini, |f| f + 1);
        nombre.extend(tramo.drain(..).skip(ini).take(fin - ini));
    };
    for (i, e) in elementos.iter().enumerate() {
        match e {
            Elemento::Palabra(p) if !usados[i] && p.chars().all(char::is_alphabetic) => tramo.push(p.clone()),
            _ => cerrar(&mut tramo, &mut nombre),
        }
    }
    cerrar(&mut tramo, &mut nombre);

    let mut nota = juntar_notas(notas);
    if let Some(n) = nota.as_mut().filter(|n| n.dudosa.is_none() && sobra_un_numero) {
        n.dudosa = Some(PorQueDudosa::NumeroDeMas);
    }
    Frase { nota, nombre, nota_rara }
}

// ── Trocear ─────────────────────────────────────────────────────────────

/// Separa palabras y cifras («Un10» → «un», «10»), en minúscula y sin
/// acentos; quita la puntuación y deja juntos los números con decimales:
/// «7,5», «7'5» y «5-75» pasan a «7.5» y «5.75»; «5x25» y «7:30» se quedan
/// así, y se leen aparte.
fn trocear(texto: &str) -> Vec<String> {
    trocear_con_inicio(texto).into_iter().map(|(t, _)| t).collect()
}

/// Como [`trocear`], con la letra de `texto` en la que empieza cada trozo:
/// cada letra de `texto` es una de las que se trocean.
fn trocear_con_inicio(texto: &str) -> Vec<(String, usize)> {
    let letras: Vec<char> = texto.chars().map(sin_acento).collect();
    let cifra = |i: usize| letras.get(i).is_some_and(char::is_ascii_digit);
    let mut trozos: Vec<(String, usize)> = Vec::new();
    let mut actual = String::new();
    let mut inicio = 0;
    for (i, &c) in letras.iter().enumerate() {
        let entre_cifras = i > 0 && cifra(i - 1) && cifra(i + 1);
        let c = match c {
            ',' | '.' | '\'' | '-' if entre_cifras => '.',
            'x' | ':' if entre_cifras => c,
            c if c.is_alphanumeric() => c,
            _ => {
                if !actual.is_empty() {
                    trozos.push((std::mem::take(&mut actual), inicio));
                }
                continue;
            }
        };
        // De letras a cifras o al revés: otro trozo («Un10»). Un número
        // sigue con su separador y sus decimales («5x25»).
        let es_numero = actual.starts_with(|c: char| c.is_ascii_digit());
        if !actual.is_empty() && es_numero != c.is_ascii_digit() && !(es_numero && entre_cifras) {
            trozos.push((std::mem::take(&mut actual), inicio));
        }
        if actual.is_empty() {
            inicio = i;
        }
        actual.push(c);
    }
    if !actual.is_empty() {
        trozos.push((actual, inicio));
    }
    trozos
}

/// En minúscula y sin acentos, como trabaja el intérprete.
pub fn normalizar(palabra: &str) -> String {
    palabra.chars().map(sin_acento).collect()
}

fn sin_acento(c: char) -> char {
    let c = c.to_lowercase().next().unwrap_or(c);
    match c {
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        // La ñ se queda: «año» no es «ano». La fonética la iguala si hace falta.
        otra => otra,
    }
}

/// Si la frase es la misma repetida (un bucle de Whisper, cortado donde
/// sea), solo la primera vez.
fn sin_bucles(mut trozos: Vec<String>) -> Vec<String> {
    if let Some(periodo) = bucle(&trozos) {
        trozos.truncate(periodo);
    }
    trozos
}

/// Si los trozos son una frase repetida en bucle, cuántos trozos tiene la
/// frase.
fn bucle(trozos: &[String]) -> Option<usize> {
    let n = trozos.len();
    (1..=n / 2).find(|&periodo| {
        (periodo..n).all(|i| {
            let (a, b) = (&trozos[i % periodo], &trozos[i]);
            // La última repetición puede quedar cortada a media palabra.
            a == b || (i == n - 1 && a.starts_with(b.as_str()))
        })
    })
}

/// Si Whisper repitió la frase en bucle, la primera vez, como la escribió:
/// es lo que se enseña, diciendo que venía repetida. Si no, nada.
pub fn una_vez(texto: &str) -> Option<String> {
    let trozos = trocear_con_inicio(texto);
    let solo: Vec<String> = trozos.iter().map(|(t, _)| t.clone()).collect();
    let periodo = bucle(&solo)?;
    // `periodo` es como mucho la mitad: hay un trozo detrás.
    let corte = trozos[periodo].1;
    Some(texto.chars().take(corte).collect::<String>().trim_end().to_string())
}

// ── Los números ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Elemento {
    /// Un número entero, dicho («siete», «setenta y cinco») o escrito
    /// («7», «825»). `cifras`: cómo se escribió, si fue en cifras («05»).
    Entero {
        valor: u32,
        cifras: Option<String>,
    },
    /// Escrito con decimales: «7.5», «5x25», «7:30».
    Decimal(String),
    Palabra(String),
}

const UNIDADES: [&str; 30] = [
    "cero",
    "uno",
    "dos",
    "tres",
    "cuatro",
    "cinco",
    "seis",
    "siete",
    "ocho",
    "nueve",
    "diez",
    "once",
    "doce",
    "trece",
    "catorce",
    "quince",
    "dieciseis",
    "diecisiete",
    "dieciocho",
    "diecinueve",
    "veinte",
    "veintiuno",
    "veintidos",
    "veintitres",
    "veinticuatro",
    "veinticinco",
    "veintiseis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];
const DECENAS: [&str; 7] = ["treinta", "cuarenta", "cincuenta", "sesenta", "setenta", "ochenta", "noventa"];

fn numero_dicho(p: &str) -> Option<u32> {
    UNIDADES
        .iter()
        .position(|u| *u == p)
        .map(|i| i as u32)
        .or_else(|| DECENAS.iter().position(|d| *d == p).map(|i| 30 + 10 * i as u32))
}

fn en_cifras(t: &str) -> bool {
    t.starts_with(|c: char| c.is_ascii_digit())
}

/// Pasa los números a [`Elemento`]s: «setenta y cinco» → 75; «un» y «una»
/// cuentan como 1 solo si no van delante de otro número.
fn leer_numeros(trozos: &[String]) -> Vec<Elemento> {
    let es_numero = |t: &String| en_cifras(t) || numero_dicho(t).is_some();
    let mut elementos = Vec::new();
    let mut i = 0;
    while i < trozos.len() {
        let t = &trozos[i];
        if en_cifras(t) {
            if t.chars().all(|c| c.is_ascii_digit()) {
                let valor = t.parse().unwrap_or(u32::MAX);
                elementos.push(Elemento::Entero { valor, cifras: Some(t.clone()) });
            } else {
                elementos.push(Elemento::Decimal(t.clone()));
            }
        } else if let Some(d) = numero_dicho(t) {
            // «setenta y cinco»
            let unidad = trozos.get(i + 2).and_then(|u| numero_dicho(u)).filter(|u| (1..10).contains(u));
            match (d >= 30, trozos.get(i + 1).map(String::as_str), unidad) {
                (true, Some("y"), Some(u)) => {
                    elementos.push(Elemento::Entero { valor: d + u, cifras: None });
                    i += 2;
                }
                _ => elementos.push(Elemento::Entero { valor: d, cifras: None }),
            }
        } else if matches!(t.as_str(), "un" | "una") && !trozos.get(i + 1).is_some_and(es_numero) {
            // «Yolanda Llorente un»: el «uno» cortado.
            elementos.push(Elemento::Entero { valor: 1, cifras: None });
        } else {
            elementos.push(Elemento::Palabra(t.clone()));
        }
        i += 1;
    }
    elementos
}

/// Una nota encontrada en la frase, antes de juntarlas.
struct Encontrada {
    valores: Vec<f64>,
    reinterpretada: bool,
}

fn valida(n: f64) -> bool {
    (0.0..=10.0).contains(&n) && ((n * 100.0).round() - n * 100.0).abs() < 1e-6
}

fn redondear(n: f64) -> f64 {
    (n * 100.0).round() / 100.0
}

/// La parte decimal de lo dicho tras «con» o «coma»: «cinco» → 0,5,
/// «veinticinco» → 0,25, «setenta y cinco» → 0,75; en cifras, como se
/// escribió: «5» → 0,5, «50» → 0,5, «05» → 0,05.
fn decimales(valor: u32, cifras: &Option<String>) -> Option<f64> {
    match cifras {
        Some(c) if c.len() <= 2 => Some(valor as f64 / 10f64.powi(c.len() as i32)),
        Some(_) => None,
        None if valor < 10 => Some(valor as f64 / 10.0),
        None if valor < 100 => Some(valor as f64 / 100.0),
        None => None,
    }
}

/// Lee lo escrito con decimales: «7.5» y «5.75» tal cual; «5x25» como «5,25»
/// y «7.30» o «7:30» también como «7 y media», dudosas.
fn leer_decimal(texto: &str) -> Option<(Vec<f64>, bool)> {
    let (entera, sep, dec) = texto
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(i, c)| (&texto[..i], c, &texto[i + c.len_utf8()..]))?;
    if dec.is_empty() || !dec.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: f64 = format!("{entera}.{dec}").parse().ok()?;
    let mut valores = vec![n];
    let mut reinterpretada = sep == 'x';
    if dec == "30" || sep == ':' {
        // Una hora: «siete y media».
        valores.insert(0, entera.parse::<f64>().ok()? + 0.5);
        reinterpretada = true;
    }
    valores.retain(|v| valida(*v));
    valores.dedup();
    (!valores.is_empty()).then_some((valores, reinterpretada))
}

/// «825» → 8,25 y «75» → 7,5: un número de más de 10, escrito en cifras, que
/// solo es nota con la coma tras la primera cifra (o tras el «10»).
fn reinterpretar(cifras: &str) -> Vec<f64> {
    let mut valores = Vec::new();
    for corte in [1, 2] {
        if cifras.len() > corte {
            if let Ok(n) = format!("{}.{}", &cifras[..corte], &cifras[corte..]).parse::<f64>() {
                if valida(n) && !valores.contains(&n) {
                    valores.push(n);
                }
            }
        }
    }
    // «100» es antes 10 que 1,00.
    valores.sort_by(|a, b| b.total_cmp(a));
    valores
}

/// Busca las notas de la frase. Devuelve también qué elementos se usaron:
/// los números que no, sobran.
fn buscar_notas(e: &[Elemento]) -> (Vec<Encontrada>, Vec<bool>) {
    let mut notas = Vec::new();
    let mut usados = vec![false; e.len()];
    let palabra = |i: usize| match e.get(i) {
        Some(Elemento::Palabra(p)) => Some(p.as_str()),
        _ => None,
    };
    let mut i = 0;
    while i < e.len() {
        match &e[i] {
            Elemento::Decimal(texto) => {
                if let Some((valores, reinterpretada)) = leer_decimal(texto) {
                    notas.push(Encontrada { valores, reinterpretada });
                    usados[i] = true;
                }
            }
            Elemento::Entero { valor, cifras } => {
                let entera = *valor as f64;
                // «siete con cinco», «ocho coma veinticinco», «5 con 25»
                if let (Some("con" | "coma" | "punto"), Some(Elemento::Entero { valor: d, cifras: dc })) =
                    (palabra(i + 1), e.get(i + 2))
                {
                    // «siete coma cero cinco»
                    let (dec, largo) = match (d, dc, e.get(i + 3)) {
                        (0, None, Some(Elemento::Entero { valor: u @ 1..=9, cifras: None })) => {
                            (Some(*u as f64 / 100.0), 4)
                        }
                        _ => (decimales(*d, dc), 3),
                    };
                    if let Some(n) = dec.map(|d| redondear(entera + d)).filter(|n| valida(*n) && *valor <= 10) {
                        notas.push(Encontrada { valores: vec![n], reinterpretada: false });
                        usados[i..i + largo].fill(true);
                        i += largo;
                        continue;
                    }
                }
                // «nueve y medio», «9 y media»
                if palabra(i + 1) == Some("y") && matches!(palabra(i + 2), Some("medio" | "media")) && *valor < 10 {
                    notas.push(Encontrada { valores: vec![entera + 0.5], reinterpretada: false });
                    usados[i..i + 3].fill(true);
                    i += 3;
                    continue;
                }
                // «con cinco» sin nada delante: unos decimales sin la parte
                // entera, que se ha perdido. No es un cinco.
                let huerfano = i > 0 && matches!(palabra(i - 1), Some("con" | "coma" | "punto"));
                if huerfano {
                } else if *valor <= 10 {
                    notas.push(Encontrada { valores: vec![entera], reinterpretada: false });
                    usados[i] = true;
                } else if let Some(c) = cifras {
                    let valores = reinterpretar(c);
                    if !valores.is_empty() {
                        notas.push(Encontrada { valores, reinterpretada: true });
                        usados[i] = true;
                    }
                }
            }
            Elemento::Palabra(_) => {}
        }
        i += 1;
    }
    (notas, usados)
}

fn juntar_notas(notas: Vec<Encontrada>) -> Option<Nota> {
    let primera = notas.first()?;
    if notas.iter().all(|n| n.valores == primera.valores) {
        let dudosa = notas.iter().any(|n| n.reinterpretada).then_some(PorQueDudosa::Reinterpretada);
        return Some(Nota { valores: primera.valores.clone(), dudosa });
    }
    let mut valores: Vec<f64> = Vec::new();
    for v in notas.iter().flat_map(|n| n.valores.iter()) {
        if !valores.contains(v) {
            valores.push(*v);
        }
    }
    Some(Nota { valores, dudosa: Some(PorQueDudosa::Varias) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nota(texto: &str) -> Option<(Vec<f64>, Option<PorQueDudosa>)> {
        interpretar(texto).nota.map(|n| (n.valores, n.dudosa))
    }

    fn clara(texto: &str) -> Option<f64> {
        match nota(texto) {
            Some((v, None)) if v.len() == 1 => Some(v[0]),
            otra => panic!("«{texto}»: se esperaba una nota clara y sale {otra:?}"),
        }
    }

    fn nombre(texto: &str) -> String {
        interpretar(texto).nombre.join(" ")
    }

    #[test]
    fn notas_dichas() {
        assert_eq!(clara("Elena Gómez siete con cinco"), Some(7.5));
        assert_eq!(clara("Lucía Martín nueve y medio"), Some(9.5));
        assert_eq!(clara("Lucía Martínez ocho coma veinticinco"), Some(8.25));
        assert_eq!(clara("Pablo García un diez"), Some(10.0));
        assert_eq!(clara("Gonzalo Jiménez siete coma setenta y cinco"), Some(7.75));
        assert_eq!(clara("Hugo Herrero Iglesias siete con cincuenta"), Some(7.5));
        assert_eq!(clara("Núria Vilaseca nueve punto cinco"), Some(9.5));
        assert_eq!(clara("Carmen Vázquez cero"), Some(0.0));
        assert_eq!(clara("Yolanda Llorente uno"), Some(1.0));
        assert_eq!(clara("Ponle un cuatro a Bárbara Villalba"), Some(4.0));
        assert_eq!(clara("Siete y media para Lucía Martínez"), Some(7.5));
        assert_eq!(clara("Elena siete coma cero cinco"), Some(7.05));
    }

    #[test]
    fn notas_escritas_por_whisper() {
        assert_eq!(clara("Lucia Martínez 8,25"), Some(8.25));
        assert_eq!(clara("Maleria Benítez, 5,75."), Some(5.75));
        assert_eq!(clara("Lucia Martínez, 9 y medio."), Some(9.5));
        assert_eq!(clara("Hubo Herrero Iglesias 7,50"), Some(7.5));
        assert_eq!(clara("Carmen Vázquez 8 con 25"), Some(8.25));
        assert_eq!(clara("Pablo García, un 10."), Some(10.0));
        assert_eq!(clara("Sofía y Bañez, un 10."), Some(10.0));
        assert_eq!(clara("Mohamed tiene un 6."), Some(6.0));
        assert_eq!(clara("Jordi Puigvert, siete."), Some(7.0));
        assert_eq!(clara("Gonzalo a cinco."), Some(5.0));
        assert_eq!(clara("Pablo García Un10"), Some(10.0));
        assert_eq!(clara("Yolanda Llorente Tres y Medio"), Some(3.5));
    }

    #[test]
    fn lo_que_se_lee_de_dos_maneras_queda_dudoso() {
        use PorQueDudosa::*;
        assert_eq!(nota("Carmen Vázquez 825"), Some((vec![8.25], Some(Reinterpretada))));
        assert_eq!(nota("Mar-Bar-Vial-Barray 5x25"), Some((vec![5.25], Some(Reinterpretada))));
        assert_eq!(nota("7.30 para Lucía Martínez."), Some((vec![7.5, 7.3], Some(Reinterpretada))));
        assert_eq!(nota("Elena 75"), Some((vec![7.5], Some(Reinterpretada))));
        assert_eq!(nota("Elena 100"), Some((vec![10.0, 1.0], Some(Reinterpretada))));
        assert_eq!(nota("Elena siete y cinco"), Some((vec![7.0, 5.0], Some(Varias))));
        assert_eq!(nota("9 para al bárbara ochenta"), Some((vec![9.0], Some(NumeroDeMas))));
    }

    #[test]
    fn sin_nota() {
        assert_eq!(nota("Way this."), None);
        assert_eq!(nota("Si menos, no hay medio"), None);
        assert_eq!(nota("Elena doce"), None);
        assert_eq!(nota("Elena 7.555"), None);
        assert_eq!(nota(""), None);
        // Se perdió el «cuatro» de «cuatro con cinco»: no es un cinco.
        assert_eq!(nota("Ximena Quiroga con cinco"), None);
    }

    #[test]
    fn los_bucles_se_quedan_en_la_primera_vez() {
        let t = "Elena Rémez, 7.5 Elena Rémez, 7.5 Elena Rémez, 7.5 Elena Rémez, 7.5 Elena";
        assert_eq!(clara(t), Some(7.5));
        assert_eq!(nombre(t), "elena remez");
        let t = "Ponle un 4 a barbara Villalba. Ponle un 4 a barbara Villalba. Ponle un 4 a barbara Vill";
        assert_eq!(clara(t), Some(4.0));
        assert_eq!(nombre(t), "barbara villalba");
        // Dos números distintos no son un bucle.
        assert_eq!(nota("Elena 7 Elena 8").map(|n| n.1), Some(Some(PorQueDudosa::Varias)));
    }

    #[test]
    fn el_bucle_se_ensena_una_vez_como_se_escribio() {
        // El de la prueba a mano de H3a (2026-11-19).
        let t = "Marta Ortera Lucia, 7 Marta Ortera Lucia, 7 Marta Ortera Lucia, 7 Marta Ortera Lucia, 7 Mart";
        assert_eq!(una_vez(t).as_deref(), Some("Marta Ortera Lucia, 7"));
        let t = "Pablo García, un 10. Pablo García, un 10. Pablo";
        assert_eq!(una_vez(t).as_deref(), Some("Pablo García, un 10."));
        let t = "Elena Rémez, 7.5 Elena Rémez, 7.5 Elena";
        assert_eq!(una_vez(t).as_deref(), Some("Elena Rémez, 7.5"));
        assert_eq!(una_vez("Lucía Martín Ortega nueve"), None);
        assert_eq!(una_vez("Elena 7 Elena 8"), None);
        assert_eq!(una_vez(""), None);
    }

    #[test]
    fn el_nombre_sin_relleno_en_los_extremos() {
        assert_eq!(nombre("Ponle un 4 a barbara Villalba"), "barbara villalba");
        assert_eq!(nombre("9 para Álvaro Cuesta"), "alvaro cuesta");
        assert_eq!(nombre("Mohamed tiene un 6."), "mohamed");
        assert_eq!(nombre("Mohamed el Amrani 7.5"), "mohamed el amrani");
        assert_eq!(nombre("Hay tan a Cervera un 7."), "hay tan a cervera");
        assert_eq!(nombre("Sofí el baño y 10."), "sofi el baño");
        assert_eq!(nombre("Eche verría a 7."), "eche verria");
        assert_eq!(nombre("Núria 8"), "nuria");
    }
}
