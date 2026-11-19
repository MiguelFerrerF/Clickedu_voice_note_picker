//! Medir en local (D8, H3a): pasar los dictados del registro por la versión
//! de ahora y **contar**, sin enseñar un nombre.
//!
//! Cada dictado guardado tiene lo oído y, por lo que hizo el profesor
//! después, lo que había que apuntar ([`verdad`]). Se vuelve a entender lo
//! oído con los alumnos de su clase y se juzga ([`juzgar`]) lo que decidió
//! la app entonces y lo que decide ahora, con la misma vara. Así se sabe si
//! un cambio en [`crate::voz::entender`] mejora o empeora con casos de
//! verdad: sobre todo, si hay alguna nota **mal puesta en silencio**.
//!
//! Mide de lo oído en adelante. **El audio no se guarda** (el usuario,
//! 2026-11-19), así que un cambio de Whisper o del modelo no se mide aquí:
//! para eso están las 100 grabaciones de H3a (`examples/probar_entender`).
//! Lo lanza `examples/medir_registro`.

use std::collections::HashMap;

use super::{Caso, Que, Respuesta};
use crate::voz::entender::{entender, Entendido};

/// Lo que se sabe de lo que había que apuntar, por lo que hizo el profesor.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdad {
    /// Esta nota a este alumno: la eligió, la corrigió o la dejó estar.
    Es { clave: String, nota: f64 },
    /// No se sabe qué era, pero esto no: lo deshizo o lo descartó.
    NoEs(Vec<(String, f64)>),
    /// No hizo nada con una duda, o no se entendió.
    NoSeSabe,
}

/// Lo que había que apuntar, por lo que entendió la app y lo que hizo el
/// profesor con ello, en orden. Un seguro que nadie tocó estaba bien.
pub fn verdad(entendido: &Entendido<String>, respuestas: &[&Respuesta]) -> Verdad {
    // La nota apuntada en cada momento, si hay una.
    let mut apuntada = match entendido {
        Entendido::Seguro { alumno, nota } => Some((alumno.clone(), *nota)),
        _ => None,
    };
    let mut es = apuntada.clone();
    let mut no_es = Vec::new();
    for r in respuestas {
        match r {
            Respuesta::Elegida { clave, nota } => {
                apuntada = Some((clave.clone(), *nota));
                es = apuntada.clone();
            }
            Respuesta::Deshecha => {
                if let Some(a) = apuntada.take() {
                    no_es.push(a);
                    es = None;
                }
            }
            Respuesta::Descartada => {
                if let Entendido::Duda { alumnos, notas, .. } = entendido {
                    no_es.extend(alumnos.iter().flat_map(|a| notas.iter().map(move |n| (a.clone(), *n))));
                }
            }
            Respuesta::Corregida { nota } => {
                if let Some((clave, antes)) = apuntada.take() {
                    match nota {
                        Some(n) => {
                            apuntada = Some((clave, *n));
                            es = apuntada.clone();
                        }
                        // La borró: esa nota no iba.
                        None => {
                            no_es.push((clave, antes));
                            es = None;
                        }
                    }
                }
            }
        }
    }
    match es {
        Some((clave, nota)) => Verdad::Es { clave, nota },
        None if !no_es.is_empty() => Verdad::NoEs(no_es),
        None => Verdad::NoSeSabe,
    }
}

/// Cómo le fue a la app con un dictado. Del mejor al peor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Veredicto {
    /// Apuntada sin preguntar, y era eso.
    ApuntadaBien,
    /// Apuntada sin preguntar, y no se sabe si era eso.
    ApuntadaSinSaber,
    /// Preguntada, y lo bueno estaba entre las opciones.
    PreguntadaConLaBuena,
    /// Preguntada, y no se sabe qué era.
    PreguntadaSinSaber,
    /// Preguntada, sin lo bueno entre las opciones: hubo que corregir a mano.
    PreguntadaSinLaBuena,
    NoEntendida,
    /// Apuntada sin preguntar, y no era eso. **La que no puede pasar.**
    MalEnSilencio,
}

impl Veredicto {
    pub const TODOS: [Veredicto; 7] = [
        Self::ApuntadaBien,
        Self::ApuntadaSinSaber,
        Self::PreguntadaConLaBuena,
        Self::PreguntadaSinSaber,
        Self::PreguntadaSinLaBuena,
        Self::NoEntendida,
        Self::MalEnSilencio,
    ];

    pub fn en_palabras(self) -> &'static str {
        match self {
            Self::ApuntadaBien => "apuntada, bien",
            Self::ApuntadaSinSaber => "apuntada, sin saber si bien",
            Self::PreguntadaConLaBuena => "preguntada, con la buena",
            Self::PreguntadaSinSaber => "preguntada, sin saber cuál era",
            Self::PreguntadaSinLaBuena => "preguntada, sin la buena",
            Self::NoEntendida => "no entendida",
            Self::MalEnSilencio => "MAL EN SILENCIO",
        }
    }
}

fn misma(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Lo que decidió la app, juzgado con lo que había que apuntar.
pub fn juzgar(entendido: &Entendido<String>, verdad: &Verdad) -> Veredicto {
    match entendido {
        Entendido::Seguro { alumno, nota } => match verdad {
            Verdad::Es { clave, nota: buena } if clave == alumno && misma(*buena, *nota) => Veredicto::ApuntadaBien,
            Verdad::Es { .. } => Veredicto::MalEnSilencio,
            Verdad::NoEs(mal) if mal.iter().any(|(c, n)| c == alumno && misma(*n, *nota)) => Veredicto::MalEnSilencio,
            _ => Veredicto::ApuntadaSinSaber,
        },
        Entendido::Duda { alumnos, notas, .. } => match verdad {
            Verdad::Es { clave, nota } if alumnos.contains(clave) && notas.iter().any(|n| misma(*n, *nota)) => {
                Veredicto::PreguntadaConLaBuena
            }
            Verdad::Es { .. } => Veredicto::PreguntadaSinLaBuena,
            _ => Veredicto::PreguntadaSinSaber,
        },
        Entendido::NoEntendido { .. } => Veredicto::NoEntendida,
    }
}

/// Un dictado del registro, entonces y ahora.
#[derive(Debug, Clone, PartialEq)]
pub struct Medido {
    pub fichero: String,
    pub oido: String,
    pub voz: bool,
    pub antes: Veredicto,
    pub ahora: Veredicto,
}

#[derive(Debug, Default)]
pub struct Medida {
    pub medidos: Vec<Medido>,
    /// Escritas en el campo de desarrollo: no cuentan si no se piden.
    pub escritas: usize,
    /// De una clase que ya no se puede leer.
    pub sin_clase: usize,
}

/// Vuelve a entender cada dictado del registro (solo los dictados con
/// ESPACIO, salvo con `escritas`) y lo juzga entonces y ahora. `clase` da
/// los alumnos de una plantilla, como `(clave, nombre)`, o nada si ya no se
/// puede leer.
pub fn medir(casos: &[Caso], escritas: bool, mut clase: impl FnMut(&str) -> Option<Vec<(String, String)>>) -> Medida {
    let mut respuestas: HashMap<&str, Vec<&Respuesta>> = HashMap::new();
    for c in casos {
        if let Que::Respuesta { id, respuesta } = &c.que {
            respuestas.entry(id.as_str()).or_default().push(respuesta);
        }
    }
    let mut clases: HashMap<String, Option<Vec<(String, String)>>> = HashMap::new();
    let mut medida = Medida::default();
    for c in casos {
        let Que::Dictado { dictado, fichero, voz, .. } = &c.que else { continue };
        if !voz && !escritas {
            medida.escritas += 1;
            continue;
        }
        let Some(alumnos) = clases.entry(fichero.clone()).or_insert_with(|| clase(fichero)) else {
            medida.sin_clase += 1;
            continue;
        };
        let verdad = verdad(&dictado.entendido, respuestas.get(dictado.id.as_str()).map_or(&[], |r| &r[..]));
        let nombres: Vec<&str> = alumnos.iter().map(|(_, n)| n.as_str()).collect();
        let ahora = entender(&dictado.oido, &nombres).con(|i| alumnos[i].0.clone());
        medida.medidos.push(Medido {
            fichero: fichero.clone(),
            oido: dictado.oido.clone(),
            voz: *voz,
            antes: juzgar(&dictado.entendido, &verdad),
            ahora: juzgar(&ahora, &verdad),
        });
    }
    medida
}

/// Cuántos hay de cada veredicto.
pub fn contar(medidos: &[Medido], cuando: impl Fn(&Medido) -> Veredicto) -> HashMap<Veredicto, usize> {
    let mut cuenta = HashMap::new();
    for m in medidos {
        *cuenta.entry(cuando(m)).or_insert(0) += 1;
    }
    cuenta
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registro::{Candidato, Dictado};
    use crate::voz::entender::Motivo;

    fn seguro(alumno: &str, nota: f64) -> Entendido<String> {
        Entendido::Seguro { alumno: alumno.into(), nota }
    }

    fn duda(alumnos: &[&str], notas: &[f64]) -> Entendido<String> {
        Entendido::Duda {
            alumnos: alumnos.iter().map(|a| a.to_string()).collect(),
            notas: notas.to_vec(),
            motivos: vec![Motivo::NombreParecido],
        }
    }

    fn es(clave: &str, nota: f64) -> Verdad {
        Verdad::Es { clave: clave.into(), nota }
    }

    #[test]
    fn la_verdad_sale_de_lo_que_hizo_el_profesor() {
        let elegida = Respuesta::Elegida { clave: "B".into(), nota: 7.5 };
        // Un seguro que nadie tocó estaba bien.
        assert_eq!(verdad(&seguro("A", 9.0), &[]), es("A", 9.0));
        assert_eq!(verdad(&seguro("A", 9.0), &[&Respuesta::Deshecha]), Verdad::NoEs(vec![("A".into(), 9.0)]));
        assert_eq!(verdad(&seguro("A", 9.0), &[&Respuesta::Corregida { nota: Some(8.0) }]), es("A", 8.0));
        assert_eq!(
            verdad(&seguro("A", 9.0), &[&Respuesta::Corregida { nota: None }]),
            Verdad::NoEs(vec![("A".into(), 9.0)])
        );
        assert_eq!(verdad(&duda(&["A", "B"], &[7.5]), &[]), Verdad::NoSeSabe);
        assert_eq!(verdad(&duda(&["A", "B"], &[7.5]), &[&elegida]), es("B", 7.5));
        assert_eq!(
            verdad(&duda(&["A", "B"], &[7.5]), &[&elegida, &Respuesta::Deshecha]),
            Verdad::NoEs(vec![("B".into(), 7.5)])
        );
        assert_eq!(
            verdad(&duda(&["A", "B"], &[7.5, 7.3]), &[&Respuesta::Descartada]),
            Verdad::NoEs(vec![("A".into(), 7.5), ("A".into(), 7.3), ("B".into(), 7.5), ("B".into(), 7.3)])
        );
    }

    #[test]
    fn juzgar_con_la_misma_vara() {
        use Veredicto::*;
        assert_eq!(juzgar(&seguro("A", 9.0), &es("A", 9.0)), ApuntadaBien);
        assert_eq!(juzgar(&seguro("A", 9.0), &es("B", 9.0)), MalEnSilencio);
        assert_eq!(juzgar(&seguro("A", 9.0), &es("A", 8.0)), MalEnSilencio);
        assert_eq!(juzgar(&seguro("A", 9.0), &Verdad::NoEs(vec![("A".into(), 9.0)])), MalEnSilencio);
        assert_eq!(juzgar(&seguro("A", 9.0), &Verdad::NoEs(vec![("B".into(), 9.0)])), ApuntadaSinSaber);
        assert_eq!(juzgar(&seguro("A", 9.0), &Verdad::NoSeSabe), ApuntadaSinSaber);
        assert_eq!(juzgar(&duda(&["A", "B"], &[9.0]), &es("B", 9.0)), PreguntadaConLaBuena);
        assert_eq!(juzgar(&duda(&["A", "B"], &[9.0]), &es("C", 9.0)), PreguntadaSinLaBuena);
        assert_eq!(juzgar(&duda(&["A", "B"], &[9.0]), &Verdad::NoSeSabe), PreguntadaSinSaber);
        assert_eq!(juzgar(&Entendido::NoEntendido { motivo: Motivo::SinNota }, &es("A", 9.0)), NoEntendida);
    }

    fn caso(id: &str, oido: &str, entendido: Entendido<String>, fichero: &str, voz: bool) -> Caso {
        Caso {
            cuando: 1,
            version: "2.0.0".into(),
            que: Que::Dictado {
                dictado: Dictado { id: id.into(), oido: oido.into(), repetida: None, entendido },
                fichero: fichero.into(),
                nombre: vec![],
                candidatos: Vec::<Candidato>::new(),
                voz,
            },
        }
    }

    fn respuesta(id: &str, respuesta: Respuesta) -> Caso {
        Caso { cuando: 2, version: "2.0.0".into(), que: Que::Respuesta { id: id.into(), respuesta } }
    }

    #[test]
    fn medir_entonces_y_ahora() {
        let casos = vec![
            // Entonces preguntó, y era Ana: ahora la apunta.
            caso("a", "Ana García siete", duda(&["1", "2"], &[7.0]), "mates.xlsx", true),
            respuesta("a", Respuesta::Elegida { clave: "1".into(), nota: 7.0 }),
            // Entonces apuntó a Jordi y se deshizo: ahora lo vuelve a apuntar.
            caso("b", "Jordi Ferrer siete", seguro("2", 7.0), "mates.xlsx", true),
            respuesta("b", Respuesta::Deshecha),
            // Escrita en desarrollo: no cuenta.
            caso("c", "Ana García ocho", seguro("1", 8.0), "mates.xlsx", false),
            // De una clase que ya no está.
            caso("d", "Ana García ocho", seguro("1", 8.0), "borrada.xlsx", true),
        ];
        let clase = |f: &str| {
            (f == "mates.xlsx").then(|| {
                vec![("1".to_string(), "García López, Ana".to_string()), ("2".into(), "Ferrer Puig, Jordi".into())]
            })
        };
        let m = medir(&casos, false, clase);
        assert_eq!((m.escritas, m.sin_clase), (1, 1));
        let veredictos: Vec<_> = m.medidos.iter().map(|m| (m.antes, m.ahora)).collect();
        assert_eq!(
            veredictos,
            vec![
                (Veredicto::PreguntadaConLaBuena, Veredicto::ApuntadaBien),
                (Veredicto::MalEnSilencio, Veredicto::MalEnSilencio),
            ]
        );
        assert_eq!(medir(&casos, true, clase).medidos.len(), 3);
        assert_eq!(contar(&m.medidos, |m| m.ahora)[&Veredicto::ApuntadaBien], 1);
    }
}
