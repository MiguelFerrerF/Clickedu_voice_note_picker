//! **Planear** (H5, la tercera parte de traer): con lo bajado ya leído, qué
//! se hace con cada materia y con cada plantilla de la raíz, y a dónde pasan
//! las notas. Puro: no toca disco (eso es [`super::aplicar`]).
//!
//! - Cada materia es **nueva**, **igual** (mismo nombre, mismos alumnos y
//!   notas: no se toca ni se aparta), **cambia**, **vacía** (sin alumnos en
//!   Clickedu: no se guarda) o **fallo**.
//! - A `antiguas` va todo lo de la raíz salvo lo igual y lo de Clickedu de
//!   materias no traídas esta vez (las que han fallado).
//! - Las notas de cada apartada (las de la sesión, por encima de su columna
//!   «Nota») pasan, alumno por alumno y por su `Id`, a: lo recordado
//!   (`traidas.json`), el mismo nombre, y si no, la nueva que tenga al menos
//!   la mitad de sus alumnos (si quedan varias, la que lleve la primera
//!   palabra de la materia en el nombre viejo). Si no queda una, **duda**: se
//!   pregunta, y sin respuesta la vieja **se queda en la raíz**.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::{AvisoNombre, FalloBajada, Nombre, Retoque};
use crate::clickedu::fichas::{Estado, Materia};
use crate::plantillas::{plegar, Nota, Plantilla};
use crate::sesion::{self, Sesion};

/// Lo que se recuerda de la última vez, en `clickedu\traidas.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Recordadas {
    /// Ya se confirmó la primera vez: desde entonces se hace y se cuenta.
    #[serde(default)]
    pub confirmada: bool,
    /// El `id` de cada materia traída → su fichero.
    #[serde(default)]
    pub materias: BTreeMap<String, String>,
}

/// Una materia, bajada y leída.
pub struct Traida {
    pub materia: Materia,
    pub nombre: Nombre,
    pub contenido: Contenido,
}

pub enum Contenido {
    Plantilla {
        plantilla: Plantilla,
        estado_evaluaciones: Estado,
    },
    /// Sin alumnos en Clickedu (una pendiente puede venir así): no se guarda.
    Vacia,
    Fallo(FalloMateria),
}

/// Por qué una materia no se ha traído.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum FalloMateria {
    Bajar {
        fallo: FalloBajada,
    },
    /// Ha llegado un Excel, pero no se lee como plantilla.
    NoSeLee {
        error: String,
    },
}

/// Una plantilla de la raíz de la carpeta, leída.
pub struct EnLaRaiz {
    pub fichero: String,
    pub lectura: Result<Plantilla, String>,
}

/// Lo que se va a hacer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Plan {
    pub materias: Vec<PlanMateria>,
    /// Lo que va a `antiguas`, con a dónde pasan sus notas.
    pub apartadas: Vec<Apartada>,
    /// Con notas y sin saber a dónde: se quedan en la raíz si no se
    /// responde.
    pub dudas: Vec<Duda>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanMateria {
    /// Lo que pone Clickedu, para enseñarlo al lado.
    pub materia: Materia,
    pub fichero: String,
    pub pendiente: bool,
    pub retoques: Vec<Retoque>,
    pub avisos: Vec<AvisoNombre>,
    pub estado: EstadoMateria,
    /// Degradado: el aviso temprano de que Clickedu está cambiando.
    pub estado_evaluaciones: Option<Estado>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum EstadoMateria {
    Nueva {
        alumnos: usize,
    },
    Igual {
        alumnos: usize,
    },
    /// Ya había una con su nombre, distinta: va a `antiguas`.
    Cambia {
        alumnos: usize,
    },
    Vacia,
    Fallo {
        fallo: FalloMateria,
    },
}

impl EstadoMateria {
    /// Se escribe en la raíz.
    pub fn se_escribe(&self) -> bool {
        matches!(self, EstadoMateria::Nueva { .. } | EstadoMateria::Cambia { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Apartada {
    pub fichero: String,
    /// `None` si no tiene notas.
    pub notas: Option<Paso>,
}

/// Las notas de una apartada que pasan a una nueva.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Paso {
    pub a: String,
    pub por: Por,
    /// La clave del alumno en la nueva → su nota. Sin nombres, pero no hace
    /// falta en la interfaz.
    #[serde(skip)]
    pub notas: BTreeMap<String, f64>,
    pub pasan: usize,
    /// Con nota y ya no están en Clickedu.
    pub sin_alumno: usize,
    /// Con nota pero sin un `Id` que sirva (sin él, o repetido): no se
    /// pueden pasar.
    pub sin_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Por {
    Recordada,
    MismoNombre,
    Alumnos,
    Respuesta,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Duda {
    pub fichero: String,
    /// Cuántos alumnos con nota.
    pub notas: usize,
    /// Las nuevas que podrían ser; si ninguna se parece, todas.
    pub candidatas: Vec<String>,
}

/// Lo mismo para Windows: no distingue mayúsculas.
fn mismo(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Planea traer `traidas` a una carpeta con `raiz`. `respuestas`: a qué
/// nueva van las notas de cada duda (fichero viejo → nuevo).
pub fn planear(
    traidas: &[Traida],
    raiz: &[EnLaRaiz],
    sesion: &Sesion,
    recordadas: &Recordadas,
    respuestas: &HashMap<String, String>,
) -> Plan {
    let en_raiz = |fichero: &str| raiz.iter().find(|r| mismo(&r.fichero, fichero));

    // Las materias.
    let mut materias = Vec::new();
    for t in traidas {
        let (estado, estado_evaluaciones) = match &t.contenido {
            Contenido::Fallo(f) => (EstadoMateria::Fallo { fallo: f.clone() }, None),
            Contenido::Vacia => (EstadoMateria::Vacia, None),
            Contenido::Plantilla { plantilla, estado_evaluaciones } => {
                let alumnos = plantilla.alumnos.len();
                let estado = match en_raiz(&t.nombre.fichero) {
                    None => EstadoMateria::Nueva { alumnos },
                    Some(EnLaRaiz { lectura: Ok(vieja), .. }) if iguales(vieja, plantilla) => {
                        EstadoMateria::Igual { alumnos }
                    }
                    Some(_) => EstadoMateria::Cambia { alumnos },
                };
                (estado, Some(estado_evaluaciones.clone()))
            }
        };
        materias.push(PlanMateria {
            materia: t.materia.clone(),
            fichero: t.nombre.fichero.clone(),
            pendiente: t.nombre.pendiente,
            retoques: t.nombre.retoques.clone(),
            avisos: t.nombre.avisos.clone(),
            estado,
            estado_evaluaciones,
        });
    }

    // Lo que se queda en la raíz: lo igual y lo de las materias que no se
    // han traído esta vez (por su nombre de ahora o el recordado).
    let traidas_ahora: HashSet<&str> =
        traidas.iter().filter(|t| !matches!(t.contenido, Contenido::Fallo(_))).map(|t| t.materia.id.as_str()).collect();
    let mut se_quedan: Vec<&str> = Vec::new();
    for (t, m) in traidas.iter().zip(&materias) {
        if matches!(m.estado, EstadoMateria::Igual { .. } | EstadoMateria::Fallo { .. }) {
            se_quedan.push(&t.nombre.fichero);
        }
    }
    for (id, fichero) in &recordadas.materias {
        if !traidas_ahora.contains(id.as_str()) {
            se_quedan.push(fichero);
        }
    }

    // Las nuevas que se escriben: a ellas pueden pasar notas.
    let nuevas: Vec<(&Traida, &Plantilla)> = traidas
        .iter()
        .zip(&materias)
        .filter(|(_, m)| matches!(m.estado, EstadoMateria::Nueva { .. } | EstadoMateria::Cambia { .. }))
        .filter_map(|(t, _)| match &t.contenido {
            Contenido::Plantilla { plantilla, .. } => Some((t, plantilla)),
            _ => None,
        })
        .collect();
    // También a una igual (su nombre se recordaba con otro texto).
    let destinos: Vec<(&Traida, &Plantilla)> = traidas
        .iter()
        .zip(&materias)
        .filter(|(_, m)| matches!(m.estado, EstadoMateria::Igual { .. }))
        .filter_map(|(t, _)| match &t.contenido {
            Contenido::Plantilla { plantilla, .. } => Some((t, plantilla)),
            _ => None,
        })
        .chain(nuevas.iter().copied())
        .collect();

    let mut apartadas = Vec::new();
    let mut dudas = Vec::new();
    for vieja in raiz {
        if se_quedan.iter().any(|f| mismo(f, &vieja.fichero)) {
            continue;
        }
        let (notas, sin_id) = notas_de(&vieja.fichero, vieja.lectura.as_ref().ok(), sesion);
        if notas.is_empty() && sin_id == 0 {
            apartadas.push(Apartada { fichero: vieja.fichero.clone(), notas: None });
            continue;
        }
        match destino(vieja, &notas, &destinos, recordadas, respuestas) {
            Ok((t, plantilla, por)) => {
                let paso = pasar(&notas, sin_id, &t.nombre.fichero, plantilla, por);
                apartadas.push(Apartada { fichero: vieja.fichero.clone(), notas: Some(paso) });
            }
            Err(candidatas) => dudas.push(Duda {
                fichero: vieja.fichero.clone(),
                notas: notas.len() + sin_id,
                candidatas: if candidatas.is_empty() {
                    destinos.iter().map(|(t, _)| t.nombre.fichero.clone()).collect()
                } else {
                    candidatas
                },
            }),
        }
    }
    Plan { materias, apartadas, dudas }
}

/// Mismos alumnos, en el mismo orden, con los mismos nombres y notas.
fn iguales(a: &Plantilla, b: &Plantilla) -> bool {
    let forma =
        |p: &Plantilla| p.alumnos.iter().map(|a| (a.id.clone(), a.nombre.clone(), a.nota.clone())).collect::<Vec<_>>();
    forma(a) == forma(b)
}

/// Las notas de una plantilla de la raíz, por `Id`: las de la sesión y, del
/// que no tenga, la de su columna «Nota». Y cuántas no tienen un `Id` que
/// sirva.
fn notas_de(fichero: &str, plantilla: Option<&Plantilla>, sesion: &Sesion) -> (BTreeMap<String, f64>, usize) {
    let puestas = sesion.puestas(fichero);
    let mut notas = BTreeMap::new();
    let mut sin_id = 0;
    match plantilla {
        Some(p) => {
            for a in &p.alumnos {
                if let Nota::Valor(v) = sesion::nota_de(a, &puestas) {
                    if a.clave == a.id {
                        notas.insert(a.id.clone(), v);
                    } else {
                        sin_id += 1;
                    }
                }
            }
        }
        // No se lee: solo lo de la sesión, que va por clave.
        None => {
            for (clave, nota) in &puestas {
                match nota {
                    Some(_) if clave.starts_with("fila:") => sin_id += 1,
                    Some(v) => {
                        notas.insert(clave.clone(), *v);
                    }
                    None => {}
                }
            }
        }
    }
    (notas, sin_id)
}

/// A qué nueva pasan las notas de `vieja`; si no queda una, las que
/// podrían ser.
fn destino<'a>(
    vieja: &EnLaRaiz,
    notas: &BTreeMap<String, f64>,
    destinos: &[(&'a Traida, &'a Plantilla)],
    recordadas: &Recordadas,
    respuestas: &HashMap<String, String>,
) -> Result<(&'a Traida, &'a Plantilla, Por), Vec<String>> {
    if let Some(a) = respuestas.get(&vieja.fichero) {
        if let Some((t, p)) = destinos.iter().find(|(t, _)| mismo(&t.nombre.fichero, a)) {
            return Ok((t, p, Por::Respuesta));
        }
    }
    let recordada = recordadas.materias.iter().find(|(_, f)| mismo(f, &vieja.fichero)).map(|(id, _)| id);
    if let Some((t, p)) = recordada.and_then(|id| destinos.iter().find(|(t, _)| &t.materia.id == id)) {
        return Ok((t, p, Por::Recordada));
    }
    if let Some((t, p)) = destinos.iter().find(|(t, _)| mismo(&t.nombre.fichero, &vieja.fichero)) {
        return Ok((t, p, Por::MismoNombre));
    }

    // Por los alumnos: al menos la mitad de los suyos en la nueva.
    let suyos: HashSet<&str> = match &vieja.lectura {
        Ok(p) => p.alumnos.iter().map(|a| a.id.as_str()).filter(|id| !id.is_empty()).collect(),
        Err(_) => notas.keys().map(String::as_str).collect(),
    };
    let mut candidatas: Vec<(&Traida, &Plantilla)> = destinos
        .iter()
        .filter(|(_, p)| {
            let estan = p.alumnos.iter().filter(|a| suyos.contains(a.id.as_str())).count();
            !suyos.is_empty() && estan * 2 >= suyos.len()
        })
        .copied()
        .collect();
    if candidatas.len() > 1 {
        let viejo = plegar(&vieja.fichero);
        let con_palabra: Vec<_> = candidatas
            .iter()
            .filter(|(t, _)| t.materia.nombre.split_whitespace().next().is_some_and(|p| viejo.contains(&plegar(p))))
            .copied()
            .collect();
        if !con_palabra.is_empty() {
            candidatas = con_palabra;
        }
    }
    match candidatas.as_slice() {
        [(t, p)] => Ok((t, p, Por::Alumnos)),
        _ => Err(candidatas.iter().map(|(t, _)| t.nombre.fichero.clone()).collect()),
    }
}

/// Las notas por `Id` de una vieja, a las claves de la nueva.
fn pasar(notas: &BTreeMap<String, f64>, sin_id: usize, a: &str, nueva: &Plantilla, por: Por) -> Paso {
    // Solo los `Id` que sirven de clave (no repetidos).
    let claves: HashMap<&str, &str> =
        nueva.alumnos.iter().filter(|a| a.clave == a.id).map(|a| (a.id.as_str(), a.clave.as_str())).collect();
    let mut paso = Paso { a: a.to_string(), por, notas: BTreeMap::new(), pasan: 0, sin_alumno: 0, sin_id };
    for (id, v) in notas {
        match claves.get(id.as_str()) {
            Some(clave) => {
                paso.notas.insert(clave.to_string(), *v);
                paso.pasan += 1;
            }
            None => paso.sin_alumno += 1,
        }
    }
    paso
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plantillas::{Alumno, Columna, Columnas};

    /// Una plantilla inventada: `(id, nota)`; el nombre sale del id.
    fn plantilla(alumnos: &[(&str, Option<f64>)]) -> Plantilla {
        let col = |indice, cabecera: &str| Columna { indice, cabecera: cabecera.into() };
        Plantilla {
            alumnos: alumnos
                .iter()
                .enumerate()
                .map(|(i, (id, nota))| Alumno {
                    fila: i as u32 + 2,
                    id: id.to_string(),
                    nombre: format!("Sanz Gil, Pablo {id}"),
                    nota: nota.map_or(Nota::Vacia, Nota::Valor),
                    clave: if id.is_empty() { format!("fila:{}", i + 2) } else { id.to_string() },
                })
                .collect(),
            columnas: Columnas {
                hoja: "Worksheet".into(),
                fila_cabecera: 1,
                id: col(0, "Id"),
                nombre: col(1, "Alumnos"),
                nota: col(2, "Nota"),
            },
            avisos: vec![],
        }
    }

    fn ids(ids: &[&'static str]) -> Vec<(&'static str, Option<f64>)> {
        ids.iter().map(|i| (*i, None)).collect()
    }

    fn traida(id: &str, nombre: &str, grupo: &str, contenido: Contenido) -> Traida {
        let materia =
            Materia { id: id.into(), nombre: nombre.into(), grupo: grupo.into(), curso: Some("Tercero de ESO".into()) };
        let nombre = super::super::nombres(std::slice::from_ref(&materia)).remove(0);
        Traida { materia, nombre, contenido }
    }

    fn con(p: Plantilla) -> Contenido {
        Contenido::Plantilla { plantilla: p, estado_evaluaciones: Estado::Bien }
    }

    fn raiz(fichero: &str, p: Result<Plantilla, &str>) -> EnLaRaiz {
        EnLaRaiz { fichero: fichero.into(), lectura: p.map_err(String::from) }
    }

    fn sesion(notas: &[(&str, &str, Option<f64>)]) -> Sesion {
        let mut s = Sesion::default();
        for (f, clave, nota) in notas {
            s.clases.entry(f.to_string()).or_default().insert(clave.to_string(), *nota);
        }
        s
    }

    const BIO: &str = "Biología y Geología - 3º ESO A.xlsx";
    const FQ: &str = "Física y Química - 3º ESO A.xlsx";

    /// La primera vez, con las plantillas de la v1: los dos grupos tienen
    /// los mismos alumnos, y decide la primera palabra de la materia.
    #[test]
    fn la_primera_vez_con_las_de_la_v1() {
        let traidas = [
            traida("1", "Biología y Geología", "3º E.S.O.-A", con(plantilla(&ids(&["11", "12", "13", "14", "15"])))),
            traida("2", "Física y Química", "3º E.S.O.-A", con(plantilla(&ids(&["11", "12", "13", "14", "15"])))),
            traida("3", "Física y Química 2ºESO", "PENDIENTE FÍSICA Y QUÍMICA 2ºESO", Contenido::Vacia),
            traida(
                "4",
                "Música",
                "3º E.S.O.-A",
                Contenido::Fallo(FalloMateria::NoSeLee { error: "no es un Excel".into() }),
            ),
        ];
        let raiz = [
            // Dos puestas en la sesión y una en su columna.
            raiz(
                "biologia_3_a_eso.xlsx",
                Ok(plantilla(&[("11", None), ("12", None), ("13", Some(6.5)), ("14", None)])),
            ),
            raiz("fisicaQuimica_3_a_eso.xlsx", Ok(plantilla(&ids(&["11", "12", "13", "14"])))),
            raiz("rota.xlsx", Err("no tiene ningún alumno")),
        ];
        let s = sesion(&[("biologia_3_a_eso.xlsx", "11", Some(7.0)), ("biologia_3_a_eso.xlsx", "12", Some(4.25))]);
        let plan = planear(&traidas, &raiz, &s, &Recordadas::default(), &HashMap::new());

        let estados: Vec<&EstadoMateria> = plan.materias.iter().map(|m| &m.estado).collect();
        assert_eq!(estados[0], &EstadoMateria::Nueva { alumnos: 5 });
        assert_eq!(estados[1], &EstadoMateria::Nueva { alumnos: 5 });
        assert_eq!(estados[2], &EstadoMateria::Vacia);
        assert!(matches!(estados[3], EstadoMateria::Fallo { .. }));
        assert!(plan.materias[2].pendiente);

        assert_eq!(plan.dudas, []);
        let [bio, fq, rota] = plan.apartadas.as_slice() else { panic!("{:?}", plan.apartadas) };
        let paso = bio.notas.as_ref().unwrap();
        assert_eq!((paso.a.as_str(), paso.por, paso.pasan), (BIO, Por::Alumnos, 3));
        assert_eq!(paso.notas, BTreeMap::from([("11".into(), 7.0), ("12".into(), 4.25), ("13".into(), 6.5)]));
        assert_eq!((fq.fichero.as_str(), &fq.notas), ("fisicaQuimica_3_a_eso.xlsx", &None));
        assert_eq!((rota.fichero.as_str(), &rota.notas), ("rota.xlsx", &None));
    }

    /// La segunda vez: lo igual no se toca; la que Clickedu ha renombrado
    /// lleva sus notas a la nueva por lo recordado; la que ha fallado se
    /// queda.
    #[test]
    fn la_segunda_vez() {
        let alumnos = ids(&["11", "12", "13"]);
        let traidas = [
            traida("1", "Biología y Geología", "3º E.S.O.-A", con(plantilla(&alumnos))),
            traida("2", "Física y Química LOMLOE", "3º E.S.O.-A", con(plantilla(&alumnos))),
            traida(
                "4",
                "Música",
                "3º E.S.O.-A",
                Contenido::Fallo(FalloMateria::Bajar {
                    fallo: FalloBajada::NoEsExcel { estado: 500, contenido: "".into() },
                }),
            ),
        ];
        let raiz = [
            raiz(BIO, Ok(plantilla(&alumnos))),
            raiz(FQ, Ok(plantilla(&alumnos))),
            raiz("Música - 3º ESO A.xlsx", Ok(plantilla(&alumnos))),
        ];
        let recordadas = Recordadas {
            confirmada: true,
            materias: BTreeMap::from([
                ("1".into(), BIO.into()),
                ("2".into(), FQ.into()),
                ("4".into(), "Música - 3º ESO A.xlsx".into()),
            ]),
        };
        let s = sesion(&[(FQ, "12", Some(9.0)), (BIO, "11", Some(5.0))]);
        let plan = planear(&traidas, &raiz, &s, &recordadas, &HashMap::new());
        assert_eq!(plan.materias[0].estado, EstadoMateria::Igual { alumnos: 3 });
        assert_eq!(plan.materias[1].estado, EstadoMateria::Nueva { alumnos: 3 });
        assert_eq!(plan.materias[1].fichero, "Física y Química LOMLOE - 3º ESO A.xlsx");
        let [fq] = plan.apartadas.as_slice() else { panic!("{:?}", plan.apartadas) };
        let paso = fq.notas.as_ref().unwrap();
        assert_eq!(
            (fq.fichero.as_str(), paso.a.as_str(), paso.por),
            (FQ, "Física y Química LOMLOE - 3º ESO A.xlsx", Por::Recordada)
        );
        assert_eq!(paso.notas, BTreeMap::from([("12".into(), 9.0)]));
    }

    /// La misma, cambiada: la vieja se aparta y sus notas pasan a la nueva;
    /// la del alumno que ya no está y la que no tiene `Id`, se cuentan.
    #[test]
    fn la_misma_cambiada() {
        let traidas = [traida("1", "Biología y Geología", "3º E.S.O.-A", con(plantilla(&ids(&["11", "13", "14"]))))];
        let raiz = [raiz(BIO, Ok(plantilla(&[("11", None), ("12", Some(3.0)), ("", Some(8.0))])))];
        let s = sesion(&[(BIO, "11", Some(7.5))]);
        let plan = planear(&traidas, &raiz, &s, &Recordadas::default(), &HashMap::new());
        assert_eq!(plan.materias[0].estado, EstadoMateria::Cambia { alumnos: 3 });
        let paso = plan.apartadas[0].notas.as_ref().unwrap();
        assert_eq!((paso.a.as_str(), paso.por), (BIO, Por::MismoNombre));
        assert_eq!((paso.pasan, paso.sin_alumno, paso.sin_id), (1, 1, 1));
        assert_eq!(paso.notas, BTreeMap::from([("11".into(), 7.5)]));
    }

    /// Con notas y sin saber a dónde: se pregunta. Respondida, pasan.
    #[test]
    fn la_duda_se_pregunta() {
        let alumnos = ids(&["11", "12"]);
        let traidas = [
            traida("1", "Biología y Geología", "3º E.S.O.-A", con(plantilla(&alumnos))),
            traida("2", "Física y Química", "3º E.S.O.-A", con(plantilla(&alumnos))),
        ];
        let mia = [raiz("mi clase.xlsx", Ok(plantilla(&[("11", Some(5.0)), ("12", None)])))];
        let plan = planear(&traidas, &mia, &Sesion::default(), &Recordadas::default(), &HashMap::new());
        assert_eq!(plan.apartadas, []);
        assert_eq!(
            plan.dudas,
            [Duda { fichero: "mi clase.xlsx".into(), notas: 1, candidatas: vec![BIO.into(), FQ.into()] }]
        );

        let respuestas = HashMap::from([("mi clase.xlsx".to_string(), FQ.to_string())]);
        let plan = planear(&traidas, &mia, &Sesion::default(), &Recordadas::default(), &respuestas);
        assert_eq!(plan.dudas, []);
        let paso = plan.apartadas[0].notas.as_ref().unwrap();
        assert_eq!((paso.a.as_str(), paso.por, paso.pasan), (FQ, Por::Respuesta, 1));

        // Sin ninguna que se parezca, las candidatas son todas.
        let otra = [raiz("otra.xlsx", Ok(plantilla(&[("91", Some(5.0))])))];
        let plan = planear(&traidas, &otra, &Sesion::default(), &Recordadas::default(), &HashMap::new());
        assert_eq!(plan.dudas[0].candidatas, [BIO, FQ]);
    }
}
