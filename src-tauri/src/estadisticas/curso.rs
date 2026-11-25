//! **La evaluación y el alumno** (H8): con todos los ítems de una clase
//! traída de Clickedu, cómo va cada evaluación y cada alumno.
//!
//! No se lee nada nuevo de Clickedu: los ítems, su ponderación y las notas
//! de cada alumno al traer están en `items.json` (con la «Nota final» de
//! cada evaluación); las de la app, en la sesión de cada ítem. La nota de
//! un alumno en un ítem es la que enseña Calificar
//! ([`super::nota_efectiva`]).
//!
//! **Dos notas de la evaluación** (decidido por el usuario, 2026-11-25):
//!
//! - **La calculada**: la media ponderada de sus ítems, con lo de la app.
//!   Es provisional (la interfaz la pinta con borde discontinuo, D6). Un
//!   ítem con apartados (los que «cuelgan» de él) vale lo que la media
//!   ponderada de los apartados que tenga el alumno, y si no tiene
//!   ninguno, su propia nota. Nada en silencio: si ningún ítem tiene
//!   ponderación, es la media simple y se dice; si unos sí y otros no, los
//!   que no la tienen no cuentan (en Clickedu suman 100 sin ellos) y se
//!   dice cuáles; a quien le falta nota en algún ítem se le calcula con
//!   los que tiene y se dice cuántos le faltan.
//! - **La «Nota final» de Clickedu**, tal cual estaba al traer.
//!
//! Los nombres solo van a la pantalla, nunca al registro (D8).

use serde::Serialize;

use super::{contar, de_clickedu, nota_efectiva, Notas};
use crate::clickedu::items::{self, EvaluacionItems, ItemGuardado, ItemsMateria};
use crate::plantillas::{Alumno, Nota};
use crate::sesion::{Juego, Puestas, Sesion};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Curso {
    /// Cuándo se trajeron las notas de Clickedu (segundos Unix).
    pub traidos: u64,
    pub evaluaciones: Vec<EvaluacionCurso>,
    /// En el orden de la plantilla.
    pub alumnos: Vec<AlumnoCurso>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluacionCurso {
    pub id: String,
    pub nombre: String,
    /// La de hoy, por sus fechas.
    pub en_curso: bool,
    pub items: Vec<ItemCurso>,
    /// Ningún ítem tiene ponderación: la calculada es la media simple.
    pub media_simple: bool,
    /// Los ítems que no cuentan en la calculada: sin ponderación, cuando
    /// los demás la tienen.
    pub no_cuentan: Vec<String>,
    /// Las cuentas de la nota calculada de cada alumno.
    pub calculada: Notas,
    /// Las de la «Nota final» de Clickedu.
    pub final_clickedu: Notas,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCurso {
    pub id: String,
    pub nombre: String,
    /// En % de la evaluación ([`items::ponderacion_en_la_evaluacion`]).
    pub ponderacion: Option<f64>,
    /// El nombre del ítem del que cuelga, si cuelga de otro.
    pub dentro_de: Option<String>,
    pub con_nota: usize,
    pub media: Option<f64>,
    pub aprobados: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlumnoCurso {
    pub clave: String,
    pub nombre: String,
    /// Una por evaluación, en el orden de [`Curso::evaluaciones`].
    pub evaluaciones: Vec<DelAlumno>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DelAlumno {
    /// Su nota en cada ítem, en el orden de [`EvaluacionCurso::items`]:
    /// `None` si no tiene o no es un número.
    pub notas: Vec<Option<f64>>,
    pub calculada: Option<f64>,
    /// En cuántos de los ítems que cuentan no tiene nota.
    pub faltan: usize,
    pub final_clickedu: Option<f64>,
}

/// Lo de la clase de `alumnos`, con sus ítems de `materia` y las notas de
/// la app en `sesion`. `hoy`, `AAAA-MM-DD`.
pub fn del_curso(alumnos: &[Alumno], materia: &ItemsMateria, sesion: &Sesion, hoy: &str) -> Curso {
    let mut del_alumno: Vec<Vec<DelAlumno>> = vec![Vec::new(); alumnos.len()];
    let evaluaciones = materia
        .evaluaciones
        .iter()
        .map(|e| {
            let (ev, suyas) = de_la_evaluacion(alumnos, e, sesion, hoy);
            for (d, s) in del_alumno.iter_mut().zip(suyas) {
                d.push(s);
            }
            ev
        })
        .collect();
    Curso {
        traidos: materia.traidos,
        evaluaciones,
        alumnos: alumnos
            .iter()
            .zip(del_alumno)
            .map(|(a, evaluaciones)| AlumnoCurso { clave: a.clave.clone(), nombre: a.nombre.clone(), evaluaciones })
            .collect(),
    }
}

fn de_la_evaluacion(
    alumnos: &[Alumno],
    e: &EvaluacionItems,
    sesion: &Sesion,
    hoy: &str,
) -> (EvaluacionCurso, Vec<DelAlumno>) {
    // La nota de cada alumno (filas) en cada ítem (columnas).
    let por_item: Vec<Vec<Nota>> = e
        .items
        .iter()
        .map(|i| {
            let puestas: Puestas = sesion.puestas_de(Juego::Item(&i.id));
            alumnos.iter().map(|a| nota_efectiva(a, &puestas, Some(&i.notas))).collect()
        })
        .collect();
    let valor = |n: &Nota| match n {
        Nota::Valor(v) => Some(*v),
        _ => None,
    };

    let items: Vec<ItemCurso> = e
        .items
        .iter()
        .zip(&por_item)
        .map(|(i, notas)| {
            let n = contar(alumnos.iter().zip(notas.iter().cloned()));
            ItemCurso {
                id: i.id.clone(),
                nombre: i.nombre.clone(),
                ponderacion: items::ponderacion_en_la_evaluacion(i, &e.items),
                dentro_de: padre(i, &e.items).map(|p| p.nombre.clone()),
                con_nota: n.con_nota,
                media: n.media,
                aprobados: n.aprobados,
            }
        })
        .collect();

    // Los de arriba (los que no cuelgan de otro) y su peso.
    let arriba: Vec<usize> = (0..e.items.len()).filter(|&k| padre(&e.items[k], &e.items).is_none()).collect();
    let pesos_arriba = pesos(arriba.iter().map(|&k| e.items[k].ponderacion));
    let media_simple = pesos_arriba.simple;
    let mut no_cuentan: Vec<String> = arriba
        .iter()
        .zip(&pesos_arriba.de)
        .filter(|(_, &w)| w == 0.0)
        .map(|(&k, _)| e.items[k].nombre.clone())
        .collect();
    // Los apartados de cada uno de arriba y su peso dentro de él.
    let apartados: Vec<(Vec<usize>, Pesos)> = arriba
        .iter()
        .map(|&k| {
            let hijos: Vec<usize> =
                (0..e.items.len()).filter(|&h| e.items[h].dentro_de.as_deref() == Some(&e.items[k].id)).collect();
            let p = pesos(hijos.iter().map(|&h| e.items[h].ponderacion));
            if !hijos.is_empty() {
                no_cuentan.extend(
                    hijos.iter().zip(&p.de).filter(|(_, &w)| w == 0.0).map(|(&h, _)| e.items[h].nombre.clone()),
                );
            }
            (hijos, p)
        })
        .collect();

    let suyas: Vec<DelAlumno> = alumnos
        .iter()
        .enumerate()
        .map(|(a, alumno)| {
            let nota = |k: usize| valor(&por_item[k][a]);
            // Lo que vale cada uno de arriba: con sus apartados, si tiene
            // nota en alguno; si no, la suya.
            let de_arriba: Vec<Option<f64>> = arriba
                .iter()
                .zip(&apartados)
                .map(|(&k, (hijos, p))| ponderada(hijos.iter().map(|&h| nota(h)).zip(p.de.iter().copied())).or(nota(k)))
                .collect();
            let faltan = de_arriba.iter().zip(&pesos_arriba.de).filter(|(n, &w)| w > 0.0 && n.is_none()).count();
            DelAlumno {
                notas: (0..e.items.len()).map(nota).collect(),
                calculada: ponderada(de_arriba.into_iter().zip(pesos_arriba.de.iter().copied()))
                    .map(|v| (v * 100.0).round() / 100.0),
                faltan,
                final_clickedu: valor(&de_clickedu(e.finales.get(&alumno.id))),
            }
        })
        .collect();

    let calculada = contar(alumnos.iter().zip(&suyas).map(|(a, s)| (a, s.calculada.map_or(Nota::Vacia, Nota::Valor))));
    let final_clickedu = contar(alumnos.iter().map(|a| (a, de_clickedu(e.finales.get(&a.id)))));
    let ev = EvaluacionCurso {
        id: e.id.clone(),
        nombre: e.nombre.clone(),
        en_curso: items::en_curso(e, hoy),
        items,
        media_simple,
        no_cuentan,
        calculada,
        final_clickedu,
    };
    (ev, suyas)
}

/// El ítem del que cuelga `i`, si está entre `items`.
fn padre<'a>(i: &ItemGuardado, items: &'a [ItemGuardado]) -> Option<&'a ItemGuardado> {
    let p = i.dentro_de.as_ref()?;
    items.iter().find(|x| &x.id == p)
}

/// El peso con que cuenta cada uno: si alguno tiene ponderación, la suya
/// (0 si no tiene: no cuenta); si ninguno, todos igual (la media simple).
struct Pesos {
    de: Vec<f64>,
    simple: bool,
}

fn pesos(ponderaciones: impl Iterator<Item = Option<f64>>) -> Pesos {
    let p: Vec<Option<f64>> = ponderaciones.collect();
    if p.iter().any(|w| w.is_some_and(|w| w > 0.0)) {
        Pesos { de: p.iter().map(|w| w.filter(|&w| w > 0.0).unwrap_or(0.0)).collect(), simple: false }
    } else {
        Pesos { de: vec![1.0; p.len()], simple: true }
    }
}

/// La media de las notas que hay, cada una con su peso (los de peso 0 no
/// cuentan). Sin ninguna, `None`.
fn ponderada(notas: impl Iterator<Item = (Option<f64>, f64)>) -> Option<f64> {
    let (suma, pesos) = notas
        .filter_map(|(n, w)| n.filter(|_| w > 0.0).map(|n| (n * w, w)))
        .fold((0.0, 0.0), |(s, p), (nw, w)| (s + nw, p + w));
    (pesos > 0.0).then(|| suma / pesos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alumnos(n: usize) -> Vec<Alumno> {
        (0..n)
            .map(|i| Alumno {
                fila: i as u32 + 2,
                id: format!("{}", 1000 + i),
                nombre: format!("Alumno {i}"),
                nota: Nota::Vacia,
                clave: format!("{}", 1000 + i),
            })
            .collect()
    }

    fn item(id: &str, ponderacion: Option<f64>, dentro_de: Option<&str>, notas: &[(&str, &str)]) -> ItemGuardado {
        ItemGuardado {
            id: id.into(),
            nombre: format!("Ítem {id}"),
            ponderacion,
            dentro_de: dentro_de.map(String::from),
            en_clickedu: notas.len(),
            notas: notas.iter().map(|(a, n)| (a.to_string(), n.to_string())).collect(),
        }
    }

    fn materia(items: Vec<ItemGuardado>, finales: &[(&str, &str)]) -> ItemsMateria {
        ItemsMateria {
            traidos: 7,
            evaluaciones: vec![EvaluacionItems {
                id: "902".into(),
                nombre: "1ª evaluación".into(),
                desde: Some("2026-09-04".into()),
                hasta: Some("2026-12-20".into()),
                items,
                finales: finales.iter().map(|(a, n)| (a.to_string(), n.to_string())).collect(),
            }],
        }
    }

    fn sesion(items: &[(&str, &[(&str, Option<f64>)])]) -> Sesion {
        Sesion {
            items: items
                .iter()
                .map(|(i, p)| (i.to_string(), p.iter().map(|(a, n)| (a.to_string(), *n)).collect()))
                .collect(),
            ..Sesion::default()
        }
    }

    #[test]
    fn la_calculada_es_la_media_ponderada_con_lo_de_la_app() {
        let m = materia(
            vec![
                item("1", Some(40.0), None, &[("1000", "5"), ("1001", "8")]),
                item("2", Some(60.0), None, &[("1000", "10")]),
            ],
            &[("1000", "8"), ("1001", "NP")],
        );
        // La app manda: al segundo le pone un 6 en el ítem 2.
        let s = sesion(&[("2", &[("1001", Some(6.0))])]);
        let c = del_curso(&alumnos(3), &m, &s, "2026-11-25");
        let e = &c.evaluaciones[0];
        assert!(e.en_curso);
        assert!(!e.media_simple);
        assert!(e.no_cuentan.is_empty());
        let a = |k: usize| &c.alumnos[k].evaluaciones[0];
        assert_eq!(a(0).calculada, Some(8.0));
        assert_eq!(a(1).calculada, Some(6.8));
        assert_eq!(a(1).notas, [Some(8.0), Some(6.0)]);
        assert_eq!((a(2).calculada, a(2).faltan), (None, 2));
        assert_eq!(a(0).final_clickedu, Some(8.0));
        assert_eq!(a(1).final_clickedu, None, "NP no es una nota");
        assert_eq!((e.final_clickedu.con_nota, e.final_clickedu.no_son_nota), (1, 1));
        assert_eq!(e.calculada.con_nota, 2);
        assert!((e.calculada.media.unwrap() - 7.4).abs() < 1e-9);
        assert_eq!(e.items[1].con_nota, 2);
        assert_eq!(e.items[1].media, Some(8.0));
        assert_eq!(e.calculada.quienes[0].clave, "1000");
    }

    #[test]
    fn a_quien_le_falta_un_item_se_le_calcula_con_los_que_tiene() {
        let m = materia(vec![item("1", Some(30.0), None, &[("1000", "4")]), item("2", Some(70.0), None, &[])], &[]);
        let c = del_curso(&alumnos(1), &m, &Sesion::default(), "2027-01-10");
        let a = &c.alumnos[0].evaluaciones[0];
        assert_eq!((a.calculada, a.faltan), (Some(4.0), 1));
        assert!(!c.evaluaciones[0].en_curso);
    }

    #[test]
    fn sin_ninguna_ponderacion_es_la_media_simple() {
        let m = materia(vec![item("1", None, None, &[("1000", "4")]), item("2", None, None, &[("1000", "7")])], &[]);
        let c = del_curso(&alumnos(1), &m, &Sesion::default(), "2026-11-25");
        assert!(c.evaluaciones[0].media_simple);
        assert_eq!(c.alumnos[0].evaluaciones[0].calculada, Some(5.5));
    }

    #[test]
    fn lo_que_no_tiene_ponderacion_no_cuenta_y_se_dice() {
        let m =
            materia(vec![item("1", Some(100.0), None, &[("1000", "6")]), item("2", None, None, &[("1000", "1")])], &[]);
        let c = del_curso(&alumnos(1), &m, &Sesion::default(), "2026-11-25");
        let e = &c.evaluaciones[0];
        assert!(!e.media_simple);
        assert_eq!(e.no_cuentan, ["Ítem 2"]);
        assert_eq!((c.alumnos[0].evaluaciones[0].calculada, c.alumnos[0].evaluaciones[0].faltan), (Some(6.0), 0));
    }

    #[test]
    fn un_item_con_apartados_vale_lo_de_sus_apartados() {
        // Como en la página de verdad: el 2 vale el 50 %, y de él cuelgan
        // el 3 (70 %) y el 4 (30 %).
        let m = materia(
            vec![
                item("1", Some(50.0), None, &[("1000", "10"), ("1001", "10")]),
                item("2", Some(50.0), None, &[("1000", "1"), ("1001", "2")]),
                item("3", Some(70.0), Some("2"), &[("1000", "5")]),
                item("4", Some(30.0), Some("2"), &[("1000", "10")]),
            ],
            &[],
        );
        let c = del_curso(&alumnos(2), &m, &Sesion::default(), "2026-11-25");
        let e = &c.evaluaciones[0];
        assert_eq!(e.items[2].ponderacion, Some(35.0));
        assert_eq!(e.items[2].dentro_de.as_deref(), Some("Ítem 2"));
        // El primero: 10 × 0,5 + (5 × 0,7 + 10 × 0,3) × 0,5 = 8,25.
        assert_eq!(c.alumnos[0].evaluaciones[0].calculada, Some(8.25));
        // El segundo, sin apartados: la del 2.
        assert_eq!(c.alumnos[1].evaluaciones[0].calculada, Some(6.0));
    }

    #[test]
    fn sin_items_ni_finales_no_hay_nada() {
        let m = materia(vec![], &[]);
        let c = del_curso(&alumnos(2), &m, &Sesion::default(), "2026-11-25");
        let e = &c.evaluaciones[0];
        assert_eq!((e.calculada.con_nota, e.calculada.media, e.final_clickedu.media), (0, None, None));
        assert_eq!(c.alumnos[1].evaluaciones[0].notas, Vec::<Option<f64>>::new());
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["evaluaciones"][0]["finalClickedu"]["conNota"], 0);
        assert_eq!(json["alumnos"][0]["evaluaciones"][0]["finalClickedu"], serde_json::Value::Null);
    }
}
