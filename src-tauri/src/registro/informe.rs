//! El informe del registro (D8): lo que **sí** puede salir del ordenador.
//!
//! Lleva la **forma** de cada caso, no su contenido, como
//! `examples/revisar_plantillas`:
//!
//! - **Lo oído**, con cada palabra que no sea de dictar una nota (números,
//!   «con», «coma», «y medio», el relleno) cambiada por su forma: «Lucía
//!   Martín nueve y medio» → «▢5 ▢6 nueve y medio». Es lista blanca: lo que
//!   no se conoce, se tapa. Y se tapa también lo conocido si es una palabra
//!   del nombre de algún alumno.
//! - **Los alumnos**, como A, B, C en cada dictado, con sus números
//!   (parecido y distancia): con eso se ajustan los umbrales de
//!   [`crate::voz::entender`].
//! - **Las clases**, como «clase 1», «clase 2».
//! - **Los fallos**, con las rutas cambiadas por `%DOCUMENTOS%` y `~` y las
//!   palabras de los nombres de los alumnos tapadas.
//!
//! Nunca lleva voz (el registro no la guarda) ni nombres.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use super::{dia, respuestas, resumir, Caso, Que, Respuesta, Resumen, VERSION};
use crate::voz::entender::{Entendido, MARGEN, PARECIDO_MINIMO, PARECIDO_SEGURO};
use crate::voz::interprete::{es_de_dictar, normalizar};

/// Los dictados más recientes que van, como mucho.
pub const DICTADOS: usize = 2000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Informe {
    pub que_es: &'static str,
    pub version: &'static str,
    pub generado: String,
    pub desde: Option<String>,
    pub resumen: Resumen,
    pub umbrales: Umbrales,
    pub dictados: Vec<DictadoInforme>,
    pub fallos: Vec<FalloInforme>,
}

#[derive(Debug, Serialize)]
pub struct Umbrales {
    pub parecido_minimo: f32,
    pub parecido_seguro: f32,
    pub margen: usize,
}

#[derive(Debug, Serialize)]
pub struct DictadoInforme {
    pub dia: String,
    pub version: String,
    pub clase: String,
    pub oido: String,
    /// Lo entendido, con los alumnos como A, B, C.
    pub entendido: Entendido<String>,
    pub nombre_oido: Vec<String>,
    pub candidatos: Vec<CandidatoInforme>,
    /// Lo que hizo el profesor; `None` si nada (un seguro que estaba bien, o
    /// una duda que quedó sin contestar).
    pub respuesta: Option<Respuesta>,
}

#[derive(Debug, Serialize)]
pub struct CandidatoInforme {
    pub alumno: String,
    pub parecido: f32,
    pub distancia: usize,
}

#[derive(Debug, Serialize)]
pub struct FalloInforme {
    pub donde: String,
    pub error: String,
    pub veces: usize,
    pub ultimo: String,
}

/// Lo que hay que tapar: las palabras de los nombres de los alumnos y las
/// rutas del ordenador.
#[derive(Debug, Default)]
pub struct Tapar {
    palabras: HashSet<String>,
    /// La ruta y lo que se pone en su lugar, la más larga primero.
    rutas: Vec<(String, String)>,
}

impl Tapar {
    /// `nombres`: los de los alumnos de todas las clases. `rutas`: la ruta
    /// y su sustituto (`Documentos` → `%DOCUMENTOS%`, la carpeta del usuario
    /// → `~`).
    pub fn new<S: AsRef<str>>(nombres: &[S], rutas: Vec<(String, String)>) -> Self {
        let mut palabras = HashSet::new();
        for n in nombres {
            palabras.extend(n.as_ref().split(|c: char| !c.is_alphabetic()).filter(|p| !p.is_empty()).map(normalizar));
        }
        let mut rutas: Vec<_> = rutas.into_iter().filter(|(r, _)| !r.is_empty()).collect();
        rutas.sort_by_key(|(r, _)| std::cmp::Reverse(r.len()));
        Self { palabras, rutas }
    }

    /// Una palabra del nombre de algún alumno. Las de dos letras («de»,
    /// «la», «i») no: no dicen de quién es.
    fn es_de_alumno(&self, palabra: &str) -> bool {
        let p = normalizar(palabra);
        p.chars().count() > 2 && self.palabras.contains(&p)
    }

    /// Lo oído, con solo las palabras de dictar a la vista.
    pub fn frase(&self, texto: &str) -> String {
        por_palabras(texto, |p| es_de_dictar(p) && !self.es_de_alumno(p))
    }

    /// Un error, sin rutas del ordenador ni nombres de alumnos.
    pub fn error(&self, texto: &str) -> String {
        let mut t = texto.to_string();
        for (ruta, por) in &self.rutas {
            t = reemplazar_sin_mayusculas(&t, ruta, por);
        }
        por_palabras(&t, |p| !self.es_de_alumno(p))
    }
}

/// Cada palabra que no pasa `dejar`, cambiada por su forma (`▢5`). Lo que
/// no son letras (cifras, espacios, puntuación) se queda.
fn por_palabras(texto: &str, dejar: impl Fn(&str) -> bool) -> String {
    let mut sale = String::new();
    let mut palabra = String::new();
    let cerrar = |palabra: &mut String, sale: &mut String| {
        if palabra.is_empty() {
            return;
        }
        if dejar(palabra) {
            sale.push_str(palabra);
        } else {
            sale.push_str(&format!("▢{}", palabra.chars().count()));
        }
        palabra.clear();
    };
    for c in texto.chars() {
        if c.is_alphabetic() {
            palabra.push(c);
        } else {
            cerrar(&mut palabra, &mut sale);
            sale.push(c);
        }
    }
    cerrar(&mut palabra, &mut sale);
    sale
}

/// Las rutas de Windows no distinguen mayúsculas.
fn reemplazar_sin_mayusculas(texto: &str, ruta: &str, por: &str) -> String {
    let bajo = texto.to_lowercase();
    let ruta_baja = ruta.to_lowercase();
    // Solo si pasar a minúscula no cambia las posiciones (ASCII y casi todo).
    if bajo.len() != texto.len() {
        return texto.replace(ruta, por);
    }
    let mut sale = String::new();
    let mut i = 0;
    while let Some(j) = bajo[i..].find(&ruta_baja) {
        sale.push_str(&texto[i..i + j]);
        sale.push_str(por);
        i += j + ruta.len();
    }
    sale.push_str(&texto[i..]);
    sale
}

pub fn generar(casos: &[Caso], tapar: &Tapar, ahora: u64) -> Informe {
    let contestadas = respuestas(casos);
    let mut clases: HashMap<&str, String> = HashMap::new();
    let mut dictados = Vec::new();
    let mut fallos: Vec<FalloInforme> = Vec::new();
    for c in casos {
        match &c.que {
            Que::Dictado { dictado, fichero, nombre, candidatos } => {
                let n = clases.len() + 1;
                let clase = clases.entry(fichero).or_insert_with(|| format!("clase {n}")).clone();
                // Los alumnos, como A, B, C: los candidatos primero, por orden.
                let mut letras: HashMap<String, String> = HashMap::new();
                let mut letra = |clave: &str| {
                    let n = letras.len();
                    letras.entry(clave.to_string()).or_insert_with(|| nombre_de_letra(n)).clone()
                };
                let candidatos = candidatos
                    .iter()
                    .map(|k| CandidatoInforme { alumno: letra(&k.clave), parecido: k.parecido, distancia: k.distancia })
                    .collect();
                let entendido = dictado.entendido.clone().con(|clave| letra(&clave));
                let respuesta = contestadas.get(dictado.id.as_str()).map(|r| match r {
                    Respuesta::Elegida { clave, nota } => Respuesta::Elegida { clave: letra(clave), nota: *nota },
                    otra => (*otra).clone(),
                });
                dictados.push(DictadoInforme {
                    dia: dia(c.cuando),
                    version: c.version.clone(),
                    clase,
                    oido: tapar.frase(&dictado.oido),
                    entendido,
                    nombre_oido: nombre.iter().map(|p| format!("▢{}", p.chars().count())).collect(),
                    candidatos,
                    respuesta,
                });
            }
            Que::Fallo { donde, error } => {
                let error = tapar.error(error);
                match fallos.iter_mut().find(|f| &f.donde == donde && f.error == error) {
                    Some(f) => {
                        f.veces += 1;
                        f.ultimo = dia(c.cuando);
                    }
                    None => fallos.push(FalloInforme { donde: donde.clone(), error, veces: 1, ultimo: dia(c.cuando) }),
                }
            }
            Que::Respuesta { .. } => {}
        }
    }
    let sobran = dictados.len().saturating_sub(DICTADOS);
    dictados.drain(..sobran);
    Informe {
        que_es: "Informe del Gestor de Notas para mejorar el dictado. No lleva nombres de alumnos ni voz: \
                 cada alumno es una letra, y cada palabra que no es de dictar una nota, su número de letras (▢5).",
        version: VERSION,
        generado: dia(ahora),
        desde: casos.first().map(|c| dia(c.cuando)),
        resumen: resumir(casos),
        umbrales: Umbrales { parecido_minimo: PARECIDO_MINIMO, parecido_seguro: PARECIDO_SEGURO, margen: MARGEN },
        dictados,
        fallos,
    }
}

/// A, B… Z, AA, AB…
fn nombre_de_letra(n: usize) -> String {
    let letra = |i: usize| char::from(b'A' + i as u8);
    if n < 26 {
        letra(n).to_string()
    } else {
        format!("{}{}", letra(n / 26 - 1), letra(n % 26))
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::dictado;
    use super::super::{Caso, Que, Respuesta};
    use super::*;
    use crate::voz::entender::Motivo;

    const CLASE: &[&str] = &["Martín Ortega, Lucía", "Martínez Soler, Lucía", "Fernández de la Fuente, Ana"];

    fn tapar() -> Tapar {
        Tapar::new(
            CLASE,
            vec![
                (r"C:\Users\Profe".into(), "~".into()),
                (r"C:\Users\Profe\OneDrive - Colegio\Documentos".into(), "%DOCUMENTOS%".into()),
            ],
        )
    }

    #[test]
    fn la_frase_sin_nombres() {
        let t = tapar();
        assert_eq!(t.frase("Lucía Martín, nueve y medio."), "▢5 ▢6, nueve y medio.");
        assert_eq!(t.frase("Ponle un 10 a Noria Villa SK"), "Ponle un 10 a ▢5 ▢5 ▢2");
        assert_eq!(t.frase("Elena Gómez 825"), "▢5 ▢5 825");
        assert_eq!(t.frase("Un10 Un10"), "Un10 Un10");
        // De dictar, pero del nombre de una alumna: también se tapa.
        let t = Tapar::new(&["Cinco Ruiz, Mar"], vec![]);
        assert_eq!(t.frase("Mar Cinco siete"), "▢3 ▢5 siete");
    }

    #[test]
    fn el_error_sin_rutas_ni_nombres() {
        let t = tapar();
        assert_eq!(
            t.error(r"no se puede leer c:\users\profe\OneDrive - Colegio\Documentos\Gestor de Notas\x.xlsx"),
            r"no se puede leer %DOCUMENTOS%\Gestor de Notas\x.xlsx"
        );
        assert_eq!(t.error(r"C:\Users\Profe\AppData: falta"), r"~\AppData: falta");
        assert_eq!(t.error("la fila 7 (Martínez Soler) no tiene nota"), "la fila 7 (▢8 ▢5) no tiene nota");
    }

    #[test]
    fn el_informe_no_lleva_nombres() {
        let duda = Entendido::Duda {
            alumnos: vec!["1001".to_string(), "1002".to_string()],
            notas: vec![9.5],
            motivos: vec![Motivo::NombreParecido],
        };
        let casos = vec![
            Caso { cuando: 1_795_046_400, version: "2.0.0".into(), que: dictado("b", duda) },
            Caso {
                cuando: 1_795_046_460,
                version: "2.0.0".into(),
                que: Que::Respuesta {
                    id: "b".into(),
                    respuesta: Respuesta::Elegida { clave: "1002".into(), nota: 9.5 },
                },
            },
            Caso {
                cuando: 1_795_046_500,
                version: "2.0.0".into(),
                que: Que::Fallo { donde: "leer_clase".into(), error: "Lucía no está".into() },
            },
        ];
        let informe = generar(&casos, &tapar(), 1_795_100_000);
        let d = &informe.dictados[0];
        assert_eq!(
            (d.clase.as_str(), d.oido.as_str(), d.dia.as_str()),
            ("clase 1", "▢5 ▢6 nueve y medio", "2026-11-19")
        );
        assert_eq!(d.nombre_oido, vec!["▢5", "▢6"]);
        assert_eq!(
            d.entendido,
            Entendido::Duda {
                alumnos: vec!["A".into(), "B".into()],
                notas: vec![9.5],
                motivos: vec![Motivo::NombreParecido]
            }
        );
        assert_eq!(d.respuesta, Some(Respuesta::Elegida { clave: "B".into(), nota: 9.5 }));
        assert_eq!(informe.fallos[0].error, "▢5 no está");

        let json = serde_json::to_string(&informe).unwrap();
        for prohibido in ["Lucía", "Lucia", "lucia", "Martín", "martin", "1001", "1002", "mates", "Profe"] {
            assert!(!json.contains(prohibido), "el informe lleva «{prohibido}»: {json}");
        }
    }

    /// Las 50 frases del corpus de H3a, con su clase inventada: ninguna
    /// palabra de un nombre pasa, y la nota se ve.
    #[test]
    fn el_corpus_sin_nombres() {
        let clase: Vec<&str> =
            include_str!("../../tests/datos/voz/clase.txt").lines().filter(|l| !l.trim().is_empty()).collect();
        let t = Tapar::new(&clase, vec![]);
        let largas: HashSet<String> =
            clase.iter().flat_map(|n| n.split_whitespace()).map(normalizar).filter(|p| p.chars().count() > 2).collect();
        for linea in include_str!("../../tests/datos/voz/frases.tsv").lines().skip(1) {
            let frase = linea.split('\t').nth(4).unwrap();
            let tapada = t.frase(frase);
            for p in tapada.split(|c: char| !c.is_alphabetic()).filter(|p| !p.is_empty()) {
                assert!(!largas.contains(&normalizar(p)), "«{frase}» → «{tapada}» deja «{p}»");
            }
            assert!(tapada.chars().any(|c| c.is_alphanumeric() && c != '▢'), "«{frase}» → «{tapada}»: sin la nota");
        }
    }

    #[test]
    fn letras() {
        assert_eq!(
            (nombre_de_letra(0), nombre_de_letra(25), nombre_de_letra(26)),
            ("A".into(), "Z".into(), "AA".into())
        );
    }
}
