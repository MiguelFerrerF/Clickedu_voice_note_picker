//! Emparejar lo oído con un alumno de la clase (H3b): «Noria Villa SK» →
//! Núria Vilaseca.
//!
//! Porta el emparejamiento de la v1 (`_normalize_phonetics` +
//! `token_set_ratio` de `rapidfuzz`), cambiando dos cosas que con Whisper no
//! funcionan:
//!
//! - **Se compara todo junto, sin espacios**: Whisper parte y junta los
//!   nombres a su manera («Hay tan la Cervera», «Weicheng», «Eche verría»),
//!   y por palabras no se parecen a nada.
//! - **Cada alumno se compara con cada forma de decir su nombre**: entero,
//!   nombre y primer apellido, solo el apellido… Es decir, con cada
//!   subsecuencia de sus palabras, en orden. «Etxeberria» se parece del todo
//!   a Iker Etxeberria Zubiri, aunque falten tres palabras.
//!
//! La fonética iguala lo que en castellano (y en catalán, para los nombres)
//! suena igual o casi: b/v, ll/y/i, h muda, c/z/s, qu/c/k, g/j ante e/i,
//! x/s, tx/ch, y la «ig» final catalana («Puig» suena «puch»).

/// Un alumno de la clase y cuánto se parece a lo oído: de 0 a 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Parecido {
    /// Su posición en la lista de la clase.
    pub alumno: usize,
    pub parecido: f32,
    /// Cuántas letras (de cómo suena) habría que cambiar en lo oído para
    /// que fuera su nombre, en la forma que más se parece.
    pub distancia: usize,
}

/// Los alumnos de la clase ordenados de más a menos parecido a lo oído.
/// `oido`: las palabras del nombre, como las deja el intérprete.
pub fn emparejar<S: AsRef<str>>(oido: &[&str], clase: &[S]) -> Vec<Parecido> {
    let oido: String = oido.iter().map(|p| fonetica(p)).collect();
    if oido.is_empty() {
        return Vec::new();
    }
    let mut lista: Vec<Parecido> = clase
        .iter()
        .enumerate()
        .map(|(i, nombre)| {
            let palabras: Vec<String> =
                nombre.as_ref().split_whitespace().map(fonetica).filter(|p| !p.is_empty()).collect();
            let (parecido, distancia) = formas(&palabras)
                .map(|f| parecido(&oido, &f))
                .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
                .unwrap_or((0.0, usize::MAX));
            Parecido { alumno: i, parecido, distancia }
        })
        .collect();
    lista.sort_by(|a, b| b.parecido.total_cmp(&a.parecido));
    lista
}

/// Cada forma de decir un nombre: las subsecuencias de sus palabras, en
/// orden y juntas. Con seis palabras como mucho (63 formas); de más, se
/// toman las seis primeras. Una partícula sola («el» de «El Amrani», «de»)
/// no es una forma: si no, «el cinco» sería de Mohamed El Amrani.
fn formas(palabras: &[String]) -> impl Iterator<Item = String> + '_ {
    let n = palabras.len().min(6);
    (1u32..(1 << n))
        .map(move |m| (0..n).filter(|i| m & (1 << i) != 0).map(|i| palabras[i].as_str()).collect::<Vec<_>>())
        .filter(|f| f.iter().any(|p| !PARTICULAS.contains(p)))
        .map(|f| f.concat())
}

/// Las partículas de los apellidos, ya en [`fonetica`].
const PARTICULAS: &[&str] = &[
    "de", "del", "la", "las", "los", "el", "i", "da", "das", "do", "dos", "di", "du", "le", "al", "ben", "ban", "bon",
    "der",
];

/// 1 si suenan igual; 0 si no se parecen en nada. Con la distancia.
fn parecido(a: &str, b: &str) -> (f32, usize) {
    let largo = a.chars().count().max(b.chars().count());
    let d = distancia(a, b);
    if largo == 0 {
        return (0.0, d);
    }
    (1.0 - d as f32 / largo as f32, d)
}

/// Levenshtein, por letras.
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

/// Cómo suena una palabra: «Etxeberria» → «echeberia», «Vázquez» →
/// «baskes», «Llorente» → «iorente».
pub fn fonetica(palabra: &str) -> String {
    let c: Vec<char> = palabra
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => Some('a'),
            'é' | 'è' | 'ë' | 'ê' => Some('e'),
            'í' | 'ì' | 'ï' | 'î' => Some('i'),
            'ó' | 'ò' | 'ö' | 'ô' => Some('o'),
            'ú' | 'ù' | 'ü' | 'û' => Some('u'),
            'ñ' => Some('n'),
            'ç' => Some('s'),
            c if c.is_alphabetic() => Some(c),
            _ => None,
        })
        .collect();
    let antes_de_e_i = |i: usize| matches!(c.get(i), Some('e' | 'i'));
    // «ch» y «tx» se escriben `C`, que no sale de ninguna otra letra.
    let mut s = String::new();
    let mut i = 0;
    while i < c.len() {
        let siguiente = c.get(i + 1).copied();
        let (sonido, salto) = match (c[i], siguiente) {
            ('c', Some('h')) => ("C", 2),
            ('t', Some('x' | 'z')) => ("C", 2),
            ('c', _) if antes_de_e_i(i + 1) => ("s", 1),
            ('c' | 'k', _) => ("k", 1),
            ('q', Some('u')) => ("k", 2),
            ('q', _) => ("k", 1),
            ('g', Some('u')) if antes_de_e_i(i + 2) => ("g", 2),
            ('g', _) if antes_de_e_i(i + 1) => ("j", 1),
            // «Puig»: la «ig» catalana del final, o ante consonante.
            ('i', Some('g')) if i > 0 && !matches!(c.get(i + 2), Some('a' | 'e' | 'i' | 'o' | 'u')) => ("C", 2),
            ('l', Some('l')) => ("i", 2),
            ('n', Some('y')) => ("n", 2),
            ('s', Some('h')) => ("s", 2),
            ('h', _) => ("", 1),
            ('y', _) => ("i", 1),
            ('v' | 'b', _) => ("b", 1),
            ('w', _) => ("u", 1),
            ('z' | 'x' | 's', _) => ("s", 1),
            (otra, _) => {
                s.push(otra);
                i += 1;
                continue;
            }
        };
        s.push_str(sonido);
        i += salto;
    }
    // Las dobles suenan como una: «rr», «ss», «nn».
    let mut sin_dobles = String::new();
    for ch in s.chars() {
        if !sin_dobles.ends_with(ch) {
            sin_dobles.push(ch);
        }
    }
    sin_dobles
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLASE: &[&str] = &[
        "Lucía Martín Ortega",
        "Lucía Martínez Soler",
        "Pablo García Ruiz",
        "Pablo Gracia Navarro",
        "Núria Vilaseca Roig",
        "Iker Etxeberria Zubiri",
        "Aitana Cervera Llull",
        "Wei Chen",
        "Montserrat Xirau Bosch",
        "Jordi Puigvert Casals",
    ];

    fn primero(oido: &str) -> (&'static str, f32) {
        let palabras: Vec<&str> = oido.split_whitespace().collect();
        let l = emparejar(&palabras, CLASE);
        (CLASE[l[0].alumno], l[0].parecido)
    }

    #[test]
    fn fonetica_iguala_lo_que_suena_igual() {
        assert_eq!(fonetica("Etxeberria"), fonetica("Echeverría"));
        assert_eq!(fonetica("Vázquez"), fonetica("Basques"));
        assert_eq!(fonetica("Llorente"), fonetica("Yorente"));
        assert_eq!(fonetica("Hugo"), fonetica("Ugo"));
        assert_eq!(fonetica("Xirau"), fonetica("Sirau"));
        assert_eq!(fonetica("Puigvert"), fonetica("Puchbert"));
        assert_eq!(fonetica("Gil"), fonetica("Jil"));
        assert_eq!(fonetica("Guerra"), "gera");
        assert_eq!(fonetica("Gerardo"), fonetica("Jerardo"));
        assert_ne!(fonetica("Gómez"), fonetica("Jómez"));
    }

    #[test]
    fn lo_que_whisper_escribe_se_empareja() {
        assert_eq!(primero("noria villa sk").0, "Núria Vilaseca Roig");
        assert_eq!(primero("hay tan la cervera").0, "Aitana Cervera Llull");
        assert_eq!(primero("weicheng").0, "Wei Chen");
        assert_eq!(primero("eche verria").0, "Iker Etxeberria Zubiri");
        assert_eq!(primero("moncerrat sera").0, "Montserrat Xirau Bosch");
        assert_eq!(primero("jordi puthbert").0, "Jordi Puigvert Casals");
        assert_eq!(primero("pablo gracias").0, "Pablo Gracia Navarro");
        // García con Navarro: Navarro solo lo tiene Gracia.
        assert_eq!(primero("pablo garcia navarro").0, "Pablo Gracia Navarro");
    }

    #[test]
    fn el_nombre_exacto_da_uno() {
        assert_eq!(primero("lucia martinez"), ("Lucía Martínez Soler", 1.0));
        assert_eq!(primero("etxeberria"), ("Iker Etxeberria Zubiri", 1.0));
    }

    #[test]
    fn dos_iguales_empatan() {
        let l = emparejar(&["lucia"], CLASE);
        assert_eq!((l[0].parecido, l[1].parecido), (1.0, 1.0));
    }

    #[test]
    fn una_particula_sola_no_es_un_nombre() {
        let clase = ["Mohamed El Amrani", "Iker Etxeberria Zubiri"];
        assert!(emparejar(&["el"], &clase)[0].parecido < 0.5);
        assert_eq!(emparejar(&["el", "amrani"], &clase)[0].parecido, 1.0);
    }

    #[test]
    fn nada_oido_nada_emparejado() {
        assert!(emparejar(&[], CLASE).is_empty());
    }
}
