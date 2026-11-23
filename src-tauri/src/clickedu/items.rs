//! **Los ítems de cada materia** (H6, `docs/01`): a qué se pueden subir las
//! notas. Se leen al traer las plantillas ([`bajar`]) y se guardan en
//! `clickedu\items.json` ([`guardar`]), para que Clases los enseñe y se
//! califique por ítem también sin conexión.
//!
//! De cada evaluación, la página de importar da sus ítems
//! ([`fichas::items`]) y `GetGrades`, las notas que ya tienen en Clickedu
//! ([`fichas::notas`]): de ellas solo se guarda **cuántos alumnos** tienen
//! nota en cada ítem, para la marca discreta de Clases. La ponderación aún
//! no se lee (está en la tabla de Ítems, sin ficha): va como `None`.
//!
//! Se guarda por el `id` de la materia, no por el fichero: Clases cruza los
//! dos con `traidas.json` ([`super::traer::planear::Recordadas`]).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::cliente::{Cliente, Fallo};
use super::fichas::{self, Evaluacion, Puesta, Roto};
use crate::ajustes::guardar_json;

/// Dónde se guardan, en la carpeta `clickedu` de la app.
pub const FICHERO: &str = "items.json";

/// La página de importar de una materia y evaluación: sus ítems.
pub fn direccion_importar(materia: &str, evaluacion: &str) -> String {
    format!("/assignatures/importar_notes_items.php?assig={materia}&id_aval={evaluacion}")
}

/// Las notas ya puestas de una materia y evaluación, en JSON.
pub fn direccion_notas(materia: &str, evaluacion: &str) -> String {
    format!("/go/Index.php?ctr=Evaluate/EvaluateTermly&do=GetGrades&id={materia}&trm={evaluacion}")
}

/// `clickedu\items.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Guardados {
    /// Por `id` de materia.
    #[serde(default)]
    pub materias: BTreeMap<String, ItemsMateria>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemsMateria {
    /// Cuándo se trajeron (segundos Unix).
    pub traidos: u64,
    pub evaluaciones: Vec<EvaluacionItems>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluacionItems {
    pub id: String,
    pub nombre: String,
    /// `AAAA-MM-DD`, para saber cuál es la de hoy.
    pub desde: Option<String>,
    pub hasta: Option<String>,
    pub items: Vec<ItemGuardado>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemGuardado {
    pub id: String,
    pub nombre: String,
    /// En %; `None` mientras no se lea.
    pub ponderacion: Option<f64>,
    /// Cuántos alumnos tenían nota en Clickedu, al traer.
    pub en_clickedu: usize,
}

/// Los ítems de cada evaluación de `materia`, con cuántos alumnos tienen
/// ya nota en cada uno. Sin conexión, `Err`, como el resto de traer; una
/// página que no se entiende, `Ok(Err(roto))`: no para lo demás.
pub async fn bajar(
    cliente: &Cliente,
    materia: &str,
    evaluaciones: &[Evaluacion],
) -> Result<Result<Vec<EvaluacionItems>, Roto>, Fallo> {
    let mut res = Vec::with_capacity(evaluaciones.len());
    for e in evaluaciones {
        let pagina = cliente.abrir(&direccion_importar(materia, &e.id)).await?.texto;
        let items = match fichas::items(&pagina) {
            Ok(l) => l.dato,
            Err(roto) => return Ok(Err(roto)),
        };
        // Sin ítems no hay notas que contar: una petición menos.
        let puestas = if items.is_empty() {
            Vec::new()
        } else {
            let json = cliente.abrir(&direccion_notas(materia, &e.id)).await?.texto;
            match fichas::notas(&json) {
                Ok(l) => l.dato,
                Err(roto) => return Ok(Err(roto)),
            }
        };
        res.push(EvaluacionItems {
            id: e.id.clone(),
            nombre: e.nombre.clone(),
            desde: e.desde.clone(),
            hasta: e.hasta.clone(),
            items: items
                .into_iter()
                .map(|i| ItemGuardado {
                    en_clickedu: en_clickedu(&puestas, &i.id),
                    id: i.id,
                    nombre: i.nombre,
                    ponderacion: None,
                })
                .collect(),
        });
    }
    Ok(Ok(res))
}

/// Cuántos alumnos tienen nota en el ítem `item`: la puesta (`nota_mod`) o,
/// si no, la calculada (`nota`). Solo las filas de ítem (`type` 0): la nota
/// final lleva el id de la materia, que podría coincidir con el de un ítem.
pub fn en_clickedu(puestas: &[Puesta], item: &str) -> usize {
    let con_nota = |v: &Option<String>| v.as_deref().is_some_and(|t| !t.trim().is_empty());
    puestas
        .iter()
        .filter(|p| p.criterio == item && p.tipo.unwrap_or(0) == 0 && (con_nota(&p.nota_mod) || con_nota(&p.nota)))
        .map(|p| p.alumno.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

/// Lee `items.json`; si no está, vacío.
pub fn leer(ruta: &Path) -> Result<Guardados, String> {
    match std::fs::read_to_string(ruta) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{} está dañado: {e}", ruta.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Guardados::default()),
        Err(e) => Err(format!("no se puede leer {}: {e}", ruta.display())),
    }
}

/// Guarda los ítems de las materias de `nuevos`; las demás se quedan como
/// estaban (las que esta vez no se han podido leer, también).
pub fn guardar(ruta: &Path, nuevos: BTreeMap<String, ItemsMateria>) -> Result<(), String> {
    if nuevos.is_empty() {
        return Ok(());
    }
    // Si el de antes está dañado, se empieza de cero: es una copia de
    // Clickedu, no algo del profesor.
    let mut g = leer(ruta).unwrap_or_default();
    g.materias.extend(nuevos);
    guardar_json(ruta, &g).map_err(|e| format!("no se pueden guardar los ítems en {}: {e}", ruta.display()))
}

// ───────────────────────── Para Clases ─────────────────────────

/// Los ítems de una clase, como los pinta Clases
/// (`src/modules/clases/tipos.ts`, `ItemsClase`).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsClase {
    pub traidos: u64,
    pub evaluaciones: Vec<EvaluacionClase>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluacionClase {
    pub id: String,
    pub nombre: String,
    /// La de `hoy`, por sus fechas.
    pub en_curso: bool,
    pub items: Vec<ItemClase>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemClase {
    pub id: String,
    pub nombre: String,
    pub ponderacion: Option<f64>,
    pub en_clickedu: usize,
    /// Cuántos tienen nota en la sesión de este ítem.
    pub calificados: usize,
}

/// Los de `materia` para Clases: la evaluación de `hoy` (`AAAA-MM-DD`)
/// marcada, y `calificados(item)` contados en la sesión.
pub fn para_clases(materia: &ItemsMateria, hoy: &str, calificados: impl Fn(&str) -> usize) -> ItemsClase {
    ItemsClase {
        traidos: materia.traidos,
        evaluaciones: materia
            .evaluaciones
            .iter()
            .map(|e| EvaluacionClase {
                id: e.id.clone(),
                nombre: e.nombre.clone(),
                en_curso: match (&e.desde, &e.hasta) {
                    (Some(d), Some(h)) => d.as_str() <= hoy && hoy <= h.as_str(),
                    _ => false,
                },
                items: e
                    .items
                    .iter()
                    .map(|i| ItemClase {
                        id: i.id.clone(),
                        nombre: i.nombre.clone(),
                        ponderacion: i.ponderacion,
                        en_clickedu: i.en_clickedu,
                        calificados: calificados(&i.id),
                    })
                    .collect(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn puesta(alumno: &str, criterio: &str, tipo: i64, nota: Option<&str>, nota_mod: Option<&str>) -> Puesta {
        Puesta {
            alumno: alumno.into(),
            criterio: criterio.into(),
            tipo: Some(tipo),
            nota: nota.map(String::from),
            nota_mod: nota_mod.map(String::from),
        }
    }

    #[test]
    fn cuenta_los_alumnos_con_nota_en_el_item() {
        let puestas = [
            puesta("1", "9001", 0, None, Some("7.5")),
            puesta("2", "9001", 0, Some("6"), None),
            // Sin nada, o en blanco: no cuenta.
            puesta("3", "9001", 0, Some(""), Some(" ")),
            // Repetido: una vez.
            puesta("1", "9001", 0, Some("7.5"), Some("7.5")),
            // Otro ítem, y la nota final con un id que coincide.
            puesta("1", "9002", 0, None, Some("4")),
            puesta("4", "9001", 114, Some("8"), None),
        ];
        assert_eq!(en_clickedu(&puestas, "9001"), 2);
        assert_eq!(en_clickedu(&puestas, "9002"), 1);
        assert_eq!(en_clickedu(&puestas, "9003"), 0);
    }

    fn materia(traidos: u64, item: &str) -> ItemsMateria {
        let evaluacion = |id: &str, desde: &str, hasta: &str, items: Vec<ItemGuardado>| EvaluacionItems {
            id: id.into(),
            nombre: format!("{id}ª evaluación"),
            desde: Some(desde.into()),
            hasta: Some(hasta.into()),
            items,
        };
        ItemsMateria {
            traidos,
            evaluaciones: vec![
                evaluacion("1", "2026-09-04", "2026-11-20", vec![]),
                evaluacion(
                    "2",
                    "2026-11-21",
                    "2027-03-10",
                    vec![ItemGuardado { id: item.into(), nombre: "Parcial".into(), ponderacion: None, en_clickedu: 3 }],
                ),
            ],
        }
    }

    #[test]
    fn guardar_junta_con_lo_de_antes() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("clickedu").join("items.json");
        assert_eq!(leer(&ruta).unwrap(), Guardados::default());
        guardar(&ruta, BTreeMap::from([("101".into(), materia(1, "9001")), ("102".into(), materia(1, "9100"))]))
            .unwrap();
        guardar(&ruta, BTreeMap::from([("101".into(), materia(2, "9002"))])).unwrap();
        let g = leer(&ruta).unwrap();
        assert_eq!(g.materias["101"], materia(2, "9002"));
        assert_eq!(g.materias["102"], materia(1, "9100"));

        std::fs::write(&ruta, "{ medio").unwrap();
        assert!(leer(&ruta).is_err());
        guardar(&ruta, BTreeMap::from([("103".into(), materia(3, "9200"))])).unwrap();
        assert_eq!(leer(&ruta).unwrap().materias.len(), 1);
    }

    #[test]
    fn para_clases_marca_la_de_hoy_y_cuenta_la_sesion() {
        let c = para_clases(&materia(7, "9001"), "2026-11-23", |i| if i == "9001" { 5 } else { 0 });
        assert_eq!(c.traidos, 7);
        assert_eq!(c.evaluaciones.iter().map(|e| e.en_curso).collect::<Vec<_>>(), [false, true]);
        let i = &c.evaluaciones[1].items[0];
        assert_eq!((i.en_clickedu, i.calificados), (3, 5));
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["evaluaciones"][1]["enCurso"], true);
        assert_eq!(json["evaluaciones"][1]["items"][0]["enClickedu"], 3);
    }
}
