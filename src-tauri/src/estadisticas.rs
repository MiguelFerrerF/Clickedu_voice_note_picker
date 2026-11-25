//! Las **Estadísticas** de las notas de una clase (H8): media, mediana,
//! aprobados y el reparto por puntos. Lo del dictado está en
//! [`crate::registro::estadisticas`].
//!
//! Cuenta la nota que vale (la de la sesión o, si no, la de la plantilla:
//! [`sesion::nota_de`]) y, en un ítem sin nota en la app, la que ya tenía
//! en Clickedu al traer (H6), como la enseña Calificar. Lo que no es nota
//! (`NP`) no entra en las cuentas: se dice aparte.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::plantillas::{Alumno, Nota};
use crate::sesion::{self, Puestas};

/// Desde aquí se aprueba.
pub const APROBADO: f64 = 5.0;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notas {
    pub alumnos: usize,
    pub con_nota: usize,
    pub sin_nota: usize,
    /// Lo que hay en el campo y no es una nota.
    pub no_son_nota: usize,
    pub media: Option<f64>,
    pub mediana: Option<f64>,
    pub aprobados: usize,
    /// Cuántas notas hay de cada punto: la 0 es de 0 a 0,99…; la 10, el 10.
    pub por_punto: [usize; 11],
    /// Quién tiene cada nota, de la más alta a la más baja (y, empatados,
    /// en el orden de la lista): al pulsar una columna o los aprobados, se
    /// ven. Los nombres solo van a la pantalla, nunca al registro (D8).
    pub quienes: Vec<Quien>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quien {
    pub nombre: String,
    pub nota: f64,
    /// Su columna, `por_punto[punto]`.
    pub punto: usize,
}

fn punto(v: f64) -> usize {
    v.floor().clamp(0.0, 10.0) as usize
}

/// `en_clickedu`: en un ítem, las notas de Clickedu por `Id` de alumno.
pub fn de_la_clase(alumnos: &[Alumno], puestas: &Puestas, en_clickedu: Option<&BTreeMap<String, String>>) -> Notas {
    let mut n = Notas { alumnos: alumnos.len(), ..Notas::default() };
    let mut valores = Vec::new();
    for a in alumnos {
        let nota = match sesion::nota_de(a, puestas) {
            Nota::Vacia => match en_clickedu.and_then(|c| c.get(&a.id)).map(|t| t.trim()).filter(|t| !t.is_empty()) {
                Some(t) => crate::clickedu::subir::numero(t).map_or_else(|| Nota::Invalida(t.to_string()), Nota::Valor),
                None => Nota::Vacia,
            },
            nota => nota,
        };
        match nota {
            Nota::Valor(v) => {
                valores.push(v);
                n.quienes.push(Quien { nombre: a.nombre.clone(), nota: v, punto: punto(v) });
            }
            Nota::Vacia => n.sin_nota += 1,
            Nota::Invalida(_) => n.no_son_nota += 1,
        }
    }
    n.con_nota = valores.len();
    n.aprobados = valores.iter().filter(|&&v| v >= APROBADO).count();
    for &v in &valores {
        n.por_punto[punto(v)] += 1;
    }
    if !valores.is_empty() {
        n.media = Some(valores.iter().sum::<f64>() / valores.len() as f64);
    }
    n.mediana = mediana(&mut valores);
    n.quienes.sort_by(|a, b| b.nota.total_cmp(&a.nota));
    n
}

/// La mediana de verdad: con un número par, la media de las dos del medio
/// (la v1 se quedaba con una, D6).
pub fn mediana(valores: &mut [f64]) -> Option<f64> {
    if valores.is_empty() {
        return None;
    }
    valores.sort_by(|a, b| a.total_cmp(b));
    let n = valores.len();
    Some(if n % 2 == 1 { valores[n / 2] } else { (valores[n / 2 - 1] + valores[n / 2]) / 2.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alumnos(notas: Vec<Nota>) -> Vec<Alumno> {
        notas
            .into_iter()
            .enumerate()
            .map(|(i, nota)| Alumno {
                fila: i as u32 + 2,
                id: format!("{}", 1000 + i),
                nombre: format!("Alumno {i}"),
                nota,
                clave: format!("{}", 1000 + i),
            })
            .collect()
    }

    #[test]
    fn la_mediana_con_un_numero_par() {
        assert_eq!(mediana(&mut []), None);
        assert_eq!(mediana(&mut [3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(mediana(&mut [4.0, 1.0, 3.0, 2.0]), Some(2.5));
    }

    #[test]
    fn cuenta_la_nota_que_vale() {
        let p = alumnos(vec![
            Nota::Valor(4.99),
            Nota::Valor(5.0),
            Nota::Vacia,
            Nota::Invalida("NP".into()),
            Nota::Vacia,
            Nota::Valor(10.0),
        ]);
        // La sesión manda: al quinto le pone un 9,5 y al primero se la quita.
        let puestas: Puestas = [("1004".to_string(), Some(9.5)), ("1000".to_string(), None)].into_iter().collect();
        let n = de_la_clase(&p, &puestas, None);
        assert_eq!((n.alumnos, n.con_nota, n.sin_nota, n.no_son_nota, n.aprobados), (6, 3, 2, 1, 3));
        assert_eq!(n.media, Some(24.5 / 3.0));
        assert_eq!(n.mediana, Some(9.5));
        assert_eq!(n.por_punto, [0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1]);
        let quienes: Vec<_> = n.quienes.iter().map(|q| (q.nombre.as_str(), q.punto)).collect();
        assert_eq!(quienes, [("Alumno 5", 10), ("Alumno 4", 9), ("Alumno 1", 5)]);

        let n = de_la_clase(&p, &Puestas::new(), None);
        assert_eq!(n.mediana, Some(5.0));
        assert_eq!((n.por_punto[4], n.aprobados), (1, 2));
    }

    #[test]
    fn sin_notas_no_hay_media() {
        let n = de_la_clase(&alumnos(vec![Nota::Vacia]), &Puestas::new(), None);
        assert_eq!((n.media, n.mediana, n.con_nota), (None, None, 0));
    }

    #[test]
    fn en_un_item_cuenta_lo_de_clickedu() {
        let p = alumnos(vec![Nota::Vacia, Nota::Vacia, Nota::Vacia, Nota::Vacia]);
        // La app manda sobre Clickedu; lo que allí no es nota, aparte.
        let puestas: Puestas = [("1000".to_string(), Some(3.0))].into_iter().collect();
        let alli: BTreeMap<String, String> =
            [("1000", "9"), ("1001", "7,5"), ("1002", "NP"), ("1003", " ")].map(|(a, b)| (a.into(), b.into())).into();
        let n = de_la_clase(&p, &puestas, Some(&alli));
        assert_eq!((n.con_nota, n.sin_nota, n.no_son_nota, n.aprobados), (2, 1, 1, 1));
        assert_eq!(n.media, Some(5.25));
    }
}
