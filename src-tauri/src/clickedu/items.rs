//! **Los ítems de cada materia** (H6, `docs/01`): a qué se pueden subir las
//! notas. Se leen al traer las plantillas ([`bajar`]) y se guardan en
//! `clickedu\items.json` ([`guardar`]), para que Clases los enseñe y se
//! califique por ítem también sin conexión.
//!
//! De cada evaluación, la página de importar da sus ítems
//! ([`fichas::items`]) y `GetGrades`, las notas que ya tienen en Clickedu
//! ([`fichas::notas`]): se guarda **la nota de cada alumno** en cada ítem
//! ([`notas_de`]), para que Clases diga cuántos la tienen y Calificar la
//! enseñe junto a la de la app y avise de la que se pisaría al subir. Es
//! una copia de cuando se trajo: antes de subir se vuelve a leer. **La
//! ponderación**, de la tabla de «Ítems» de esa evaluación
//! ([`fichas::ponderaciones`]); si esa página no se entiende, los ítems
//! se guardan igual, sin ella (no es lo básico).
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

/// La página de «Ítems» de una materia y evaluación: su ponderación.
pub fn direccion_ponderaciones(materia: &str, evaluacion: &str) -> String {
    format!("{}&trm={evaluacion}", super::traer::direccion_items(materia))
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
    /// En %; `None` si en Clickedu está vacía o no se ha podido leer. Si
    /// cuelga de otro ítem (`dentro_de`), es su parte dentro de él.
    pub ponderacion: Option<f64>,
    /// El `id` del ítem ponderado del que cuelga.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dentro_de: Option<String>,
    /// Cuántos alumnos tenían nota en Clickedu, al traer: `notas.len()`.
    /// Solo cuenta en los `items.json` de antes de guardar las notas.
    #[serde(default)]
    pub en_clickedu: usize,
    /// La nota de cada alumno en Clickedu, al traer, por su `Id`: tal cual
    /// la da (un número con punto, o un texto).
    #[serde(default)]
    pub notas: BTreeMap<String, String>,
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
        // Sin ítems no hay notas ni ponderaciones: dos peticiones menos.
        let (puestas, ponderaciones) = if items.is_empty() {
            (Vec::new(), BTreeMap::new())
        } else {
            let json = cliente.abrir(&direccion_notas(materia, &e.id)).await?.texto;
            let puestas = match fichas::notas(&json) {
                Ok(l) => l.dato,
                Err(roto) => return Ok(Err(roto)),
            };
            let pagina = cliente.abrir(&direccion_ponderaciones(materia, &e.id)).await?.texto;
            (puestas, fichas::ponderaciones(&pagina).map(|l| l.dato).unwrap_or_default())
        };
        res.push(EvaluacionItems {
            id: e.id.clone(),
            nombre: e.nombre.clone(),
            desde: e.desde.clone(),
            hasta: e.hasta.clone(),
            items: items
                .into_iter()
                .map(|i| {
                    let notas = notas_de(&puestas, &i.id);
                    let p = ponderaciones.get(&i.id);
                    ItemGuardado {
                        en_clickedu: notas.len(),
                        notas,
                        ponderacion: p.and_then(|p| p.propia),
                        dentro_de: p.and_then(|p| p.dentro_de.clone()),
                        id: i.id,
                        nombre: i.nombre,
                    }
                })
                .collect(),
        });
    }
    Ok(Ok(res))
}

/// La nota de cada alumno en el ítem `item`, por su `Id`: la puesta
/// (`nota_mod`) o, si no, la calculada (`nota`). Sin ninguna, o en blanco,
/// no está. Solo las filas de ítem (`type` 0): la nota final lleva el id de
/// la materia, que podría coincidir con el de un ítem. Si un alumno sale
/// dos veces, vale la primera.
pub fn notas_de(puestas: &[Puesta], item: &str) -> BTreeMap<String, String> {
    let con_nota = |v: &Option<String>| v.as_deref().map(str::trim).filter(|t| !t.is_empty()).map(String::from);
    let mut res = BTreeMap::new();
    for p in puestas.iter().filter(|p| p.criterio == item && p.tipo.unwrap_or(0) == 0) {
        if let Some(n) = con_nota(&p.nota_mod).or_else(|| con_nota(&p.nota)) {
            res.entry(p.alumno.clone()).or_insert(n);
        }
    }
    res
}

/// Cuántos de `ids` (los alumnos de una clase) tienen nota en Clickedu en
/// el ítem `i`: una materia puede tener alumnos de otros grupos. En un
/// `items.json` de antes de guardar las notas, lo contado entonces.
pub fn en_la_clase<'a>(i: &ItemGuardado, ids: impl IntoIterator<Item = &'a str>) -> usize {
    if i.notas.is_empty() {
        return i.en_clickedu;
    }
    ids.into_iter().collect::<BTreeSet<_>>().into_iter().filter(|id| i.notas.contains_key(*id)).count()
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

/// Pone las notas de Clickedu del ítem `item` de `materia`, recién leídas
/// (al subir). Si no está guardado, nada: se verá al traer.
pub fn poner_notas(ruta: &Path, materia: &str, item: &str, notas: BTreeMap<String, String>) -> Result<(), String> {
    let mut g = leer(ruta).unwrap_or_default();
    let Some(i) = g
        .materias
        .get_mut(materia)
        .and_then(|m| m.evaluaciones.iter_mut().flat_map(|e| e.items.iter_mut()).find(|i| i.id == item))
    else {
        return Ok(());
    };
    i.en_clickedu = notas.len();
    i.notas = notas;
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
    /// En % de la evaluación: la de un ítem que cuelga de otro, ya
    /// multiplicada por la de este ([`ponderacion_en_la_evaluacion`]).
    pub ponderacion: Option<f64>,
    /// Si cuelga de un ítem ponderado: de cuál y su parte dentro de él.
    pub dentro_de: Option<Dentro>,
    /// Cuántos de la clase tienen nota en Clickedu, al traer.
    pub en_clickedu: usize,
    /// Cuántos tienen nota en la sesión de este ítem.
    pub calificados: usize,
    /// Cuántos están publicados: con nota en Clickedu y, en la app, ninguna
    /// o la misma. La barra los pinta aparte, en turquesa apagado.
    pub publicadas: usize,
    /// Cuántos tienen en la app una nota que no está en Clickedu (nueva o
    /// distinta): lo que queda por subir, en turquesa.
    pub por_subir: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dentro {
    pub nombre: String,
    pub ponderacion: Option<f64>,
}

/// La ponderación de `item` en su evaluación: la suya, o, si cuelga de
/// otro ítem de `items`, la de ese por la suya (un 30 % de un ítem que
/// vale el 10 %, un 3 %). Redondeada a centésimas. Si falta alguna, `None`.
pub fn ponderacion_en_la_evaluacion(item: &ItemGuardado, items: &[ItemGuardado]) -> Option<f64> {
    let propia = item.ponderacion?;
    let Some(padre) = &item.dentro_de else { return Some(propia) };
    let del_padre = items.iter().find(|i| &i.id == padre)?.ponderacion?;
    Some((del_padre * propia).round() / 100.0)
}

/// Lo contado de un ítem en una clase.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Contados {
    pub calificados: usize,
    pub en_clickedu: usize,
    pub publicadas: usize,
    pub por_subir: usize,
}

/// Los de `materia` para Clases: la evaluación de `hoy` (`AAAA-MM-DD`)
/// marcada, y de cada ítem, `contar(item)`: cuántos de la clase tienen nota
/// en la sesión y en Clickedu.
pub fn para_clases(materia: &ItemsMateria, hoy: &str, contar: impl Fn(&ItemGuardado) -> Contados) -> ItemsClase {
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
                    .map(|i| {
                        let c = contar(i);
                        ItemClase {
                            id: i.id.clone(),
                            nombre: i.nombre.clone(),
                            ponderacion: ponderacion_en_la_evaluacion(i, &e.items),
                            dentro_de: i.dentro_de.as_ref().map(|padre| Dentro {
                                nombre: e
                                    .items
                                    .iter()
                                    .find(|p| &p.id == padre)
                                    .map_or_else(|| padre.clone(), |p| p.nombre.clone()),
                                ponderacion: i.ponderacion,
                            }),
                            en_clickedu: c.en_clickedu,
                            calificados: c.calificados,
                            publicadas: c.publicadas,
                            por_subir: c.por_subir,
                        }
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
    fn guarda_la_nota_de_cada_alumno_en_el_item() {
        let puestas = [
            puesta("1", "9001", 0, None, Some("7.5")),
            puesta("2", "9001", 0, Some("6"), None),
            // Sin nada, o en blanco: no cuenta.
            puesta("3", "9001", 0, Some(""), Some(" ")),
            // Repetido: la primera.
            puesta("1", "9001", 0, Some("2"), Some("3")),
            // Otro ítem, y la nota final con un id que coincide.
            puesta("1", "9002", 0, None, Some("4")),
            puesta("4", "9001", 114, Some("8"), None),
        ];
        let n = |a: &str, b: &str| (a.to_string(), b.to_string());
        assert_eq!(notas_de(&puestas, "9001"), BTreeMap::from([n("1", "7.5"), n("2", "6")]));
        assert_eq!(notas_de(&puestas, "9002"), BTreeMap::from([n("1", "4")]));
        assert!(notas_de(&puestas, "9003").is_empty());
    }

    #[test]
    fn en_la_clase_cuenta_solo_sus_alumnos() {
        let notas = BTreeMap::from([("1".to_string(), "7".to_string()), ("9".to_string(), "5".to_string())]);
        let i = ItemGuardado {
            id: "9001".into(),
            nombre: "P".into(),
            ponderacion: None,
            dentro_de: None,
            en_clickedu: 2,
            notas,
        };
        assert_eq!(en_la_clase(&i, ["1", "2", "1"]), 1);
        // Un `items.json` de antes, sin notas: lo contado al traer.
        let viejo: ItemGuardado =
            serde_json::from_str(r#"{"id":"9","nombre":"P","ponderacion":null,"en_clickedu":4}"#).unwrap();
        assert_eq!(en_la_clase(&viejo, ["1"]), 4);
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
                    vec![ItemGuardado {
                        id: item.into(),
                        nombre: "Parcial".into(),
                        ponderacion: None,
                        dentro_de: None,
                        en_clickedu: 3,
                        notas: BTreeMap::new(),
                    }],
                ),
            ],
        }
    }

    #[test]
    fn la_ponderacion_de_lo_que_cuelga_de_otro_se_multiplica() {
        let item = |id: &str, ponderacion: Option<f64>, dentro_de: Option<&str>| ItemGuardado {
            id: id.into(),
            nombre: format!("Ítem {id}"),
            ponderacion,
            dentro_de: dentro_de.map(String::from),
            en_clickedu: 0,
            notas: BTreeMap::new(),
        };
        let items = vec![
            item("1", Some(10.0), None),
            item("2", Some(30.0), Some("1")),
            item("3", Some(33.33), Some("1")),
            item("4", None, Some("1")),
            item("5", Some(50.0), Some("9")),
            item("6", Some(20.0), Some("7")),
            item("7", None, None),
        ];
        let p = |i: usize| ponderacion_en_la_evaluacion(&items[i], &items);
        assert_eq!(p(0), Some(10.0));
        assert_eq!(p(1), Some(3.0));
        assert_eq!(p(2), Some(3.33));
        assert_eq!(p(3), None);
        assert_eq!(p(4), None, "el padre no está");
        assert_eq!(p(5), None, "el padre no tiene ponderación");
        let m = ItemsMateria {
            traidos: 1,
            evaluaciones: vec![EvaluacionItems {
                id: "1".into(),
                nombre: "1ª".into(),
                desde: None,
                hasta: None,
                items,
            }],
        };
        let c = para_clases(&m, "2026-11-24", |_| Contados::default());
        let hijo = &c.evaluaciones[0].items[1];
        assert_eq!(hijo.ponderacion, Some(3.0));
        assert_eq!(hijo.dentro_de, Some(Dentro { nombre: "Ítem 1".into(), ponderacion: Some(30.0) }));
        assert_eq!(c.evaluaciones[0].items[0].dentro_de, None);
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
        let c = para_clases(&materia(7, "9001"), "2026-11-23", |i| {
            if i.id == "9001" {
                Contados { calificados: 5, en_clickedu: 3, publicadas: 1, por_subir: 4 }
            } else {
                Contados::default()
            }
        });
        assert_eq!(c.traidos, 7);
        assert_eq!(c.evaluaciones.iter().map(|e| e.en_curso).collect::<Vec<_>>(), [false, true]);
        let i = &c.evaluaciones[1].items[0];
        assert_eq!((i.en_clickedu, i.calificados, i.publicadas, i.por_subir), (3, 5, 1, 4));
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["evaluaciones"][1]["enCurso"], true);
        assert_eq!(json["evaluaciones"][1]["items"][0]["enClickedu"], 3);
    }
}
