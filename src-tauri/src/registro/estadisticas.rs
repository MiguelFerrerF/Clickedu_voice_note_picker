//! Lo del dictado para las **Estadísticas** (H8), sacado del registro: de
//! una clase y de todas. Solo cuentas: ni nombres ni frases (D8).
//!
//! - **A la primera**: dictados por voz que la app apuntó sin preguntar y
//!   que nadie deshizo ni corrigió.
//! - **Segundos por nota**: la mediana de lo que pasa entre un dictado por
//!   voz y el siguiente de la misma clase, si es menos de [`HUECO`]: lo de
//!   más es una pausa, no dictar.
//! - **Tiempo ahorrado**, frente a poner cada nota a mano en Clickedu
//!   ([`A_MANO`] por nota). De cada clase e ítem cuenta **cómo llegaron las
//!   notas la última vez**: subidas desde la app (H6), sin más; exportadas,
//!   menos lo que cuesta importar el Excel a mano ([`IMPORTAR`]). Y se resta
//!   el tiempo dictando. Los números los decidió el usuario (2026-11-25).

use std::collections::HashMap;

use serde::Serialize;

use super::{respuestas, Caso, Que, Respuesta};
use crate::estadisticas::mediana;
use crate::voz::entender::Entendido;

/// Poner una nota a mano en Clickedu, en segundos (como la v1).
pub const A_MANO: u64 = 15;
/// Importar un Excel a mano en Clickedu: entrar, ir al ítem, subirlo.
pub const IMPORTAR: u64 = 180;
/// Entre dos dictados, lo que pasa de esto es una pausa.
pub const HUECO: u64 = 60;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dictados {
    /// Dictados por voz (lo escrito en el campo de desarrollo no cuenta).
    pub por_voz: usize,
    pub a_la_primera: usize,
    pub preguntados: usize,
    pub no_entendidos: usize,
    /// Apuntados sin preguntar que luego se deshicieron o corrigieron.
    pub corregidos: usize,
    pub segundos_por_nota: Option<f64>,
    /// Las notas que llegaron a Clickedu, la última vez de cada clase e ítem.
    pub notas_llevadas: usize,
    /// Las clases e ítems cuya última vez fue subirlas desde la app.
    pub subidas: usize,
    /// Las clases e ítems cuya última vez fue exportar el Excel.
    pub exportadas: usize,
    /// En segundos; sin nada llevado a Clickedu, no hay.
    pub ahorro: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadisticasDictado {
    pub clase: Dictados,
    pub todas: Dictados,
}

pub fn estadisticas(casos: &[Caso], fichero: &str) -> EstadisticasDictado {
    EstadisticasDictado {
        clase: contar(casos, |f| f.eq_ignore_ascii_case(fichero)),
        todas: contar(casos, |_| true),
    }
}

fn contar(casos: &[Caso], de: impl Fn(&str) -> bool) -> Dictados {
    let contestadas = respuestas(casos);
    let mut d = Dictados::default();
    // El último dictado por voz de cada clase, para los huecos.
    let mut ultimo: HashMap<&str, u64> = HashMap::new();
    let mut huecos = Vec::new();
    // De cada clase e ítem, la última vez: (notas, ¿exportadas?).
    let mut llevadas: HashMap<(&str, Option<&str>), (usize, bool)> = HashMap::new();
    for c in casos {
        match &c.que {
            Que::Dictado { dictado, fichero, voz: true, .. } if de(fichero) => {
                d.por_voz += 1;
                match dictado.entendido {
                    Entendido::Seguro { .. } => {
                        let deshecho = matches!(
                            contestadas.get(dictado.id.as_str()),
                            Some(Respuesta::Deshecha | Respuesta::Corregida { .. })
                        );
                        if deshecho {
                            d.corregidos += 1;
                        } else {
                            d.a_la_primera += 1;
                        }
                    }
                    Entendido::Duda { .. } => d.preguntados += 1,
                    Entendido::NoEntendido { .. } => d.no_entendidos += 1,
                }
                let clave = fichero.as_str();
                if let Some(antes) = ultimo.insert(clave, c.cuando) {
                    let hueco = c.cuando.saturating_sub(antes);
                    if hueco <= HUECO {
                        huecos.push(hueco as f64);
                    }
                }
            }
            Que::Exportado { fichero, item, notas } if de(fichero) => {
                llevadas.insert((fichero.as_str(), item.as_deref()), (*notas, true));
            }
            Que::Subido { fichero, item, notas } if de(fichero) => {
                llevadas.insert((fichero.as_str(), item.as_deref()), (*notas, false));
            }
            _ => {}
        }
    }
    d.segundos_por_nota = mediana(&mut huecos);
    d.notas_llevadas = llevadas.values().map(|(n, _)| n).sum();
    d.exportadas = llevadas.values().filter(|(_, e)| *e).count();
    d.subidas = llevadas.len() - d.exportadas;
    if !llevadas.is_empty() {
        let sin_app: i64 = llevadas
            .values()
            .map(|&(n, exportadas)| (n as u64 * A_MANO) as i64 - if exportadas { IMPORTAR as i64 } else { 0 })
            .sum();
        let dictando = d.segundos_por_nota.map_or(0.0, |s| s * d.por_voz as f64).round() as i64;
        d.ahorro = Some((sin_app - dictando).max(0) as u64);
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registro::{Dictado, VERSION};
    use crate::voz::entender::Motivo;

    fn caso(cuando: u64, que: Que) -> Caso {
        Caso { cuando, version: VERSION.into(), que }
    }

    fn dictado(cuando: u64, id: &str, fichero: &str, voz: bool, entendido: Entendido<String>) -> Caso {
        caso(
            cuando,
            Que::Dictado {
                dictado: Dictado { id: id.into(), oido: "x".into(), repetida: None, entendido },
                fichero: fichero.into(),
                nombre: vec![],
                candidatos: vec![],
                voz,
            },
        )
    }

    fn seguro() -> Entendido<String> {
        Entendido::Seguro { alumno: "1".into(), nota: 7.0 }
    }

    #[test]
    fn cuenta_los_dictados_por_voz_de_la_clase() {
        let duda = Entendido::Duda { alumnos: vec!["1".into(), "2".into()], notas: vec![7.0], motivos: vec![] };
        let casos = vec![
            dictado(100, "a", "mates.xlsx", true, seguro()),
            dictado(104, "b", "mates.xlsx", true, seguro()),
            caso(105, Que::Respuesta { id: "b".into(), respuesta: Respuesta::Deshecha }),
            dictado(110, "c", "mates.xlsx", true, duda),
            dictado(500, "d", "mates.xlsx", true, Entendido::NoEntendido { motivo: Motivo::SinNota }),
            // Escrito en el campo de desarrollo: no cuenta.
            dictado(501, "e", "mates.xlsx", false, seguro()),
            dictado(502, "f", "lengua.xlsx", true, seguro()),
        ];
        let e = estadisticas(&casos, "MATES.xlsx");
        let c = &e.clase;
        assert_eq!((c.por_voz, c.a_la_primera, c.corregidos, c.preguntados, c.no_entendidos), (4, 1, 1, 1, 1));
        // Huecos de 4 y 6 s; el de 390 s es una pausa.
        assert_eq!(c.segundos_por_nota, Some(5.0));
        assert_eq!(c.ahorro, None);
        assert_eq!(e.todas.por_voz, 5);
        // El de otra clase no hace hueco con los de esta.
        assert_eq!(e.todas.segundos_por_nota, Some(5.0));
    }

    #[test]
    fn el_ahorro_cuenta_como_llegaron_la_ultima_vez() {
        let casos = vec![
            dictado(100, "a", "mates.xlsx", true, seguro()),
            dictado(110, "b", "mates.xlsx", true, seguro()),
            // Exportada y luego subida: cuenta la subida.
            caso(200, Que::Exportado { fichero: "mates.xlsx".into(), item: Some("42".into()), notas: 10 }),
            caso(300, Que::Subido { fichero: "mates.xlsx".into(), item: Some("42".into()), notas: 20 }),
            caso(400, Que::Exportado { fichero: "mates.xlsx".into(), item: None, notas: 30 }),
            caso(500, Que::Subido { fichero: "lengua.xlsx".into(), item: Some("7".into()), notas: 4 }),
        ];
        let c = estadisticas(&casos, "mates.xlsx").clase;
        assert_eq!((c.notas_llevadas, c.subidas, c.exportadas), (50, 1, 1));
        // 20·15 + (30·15 − 180) − 2 dictados · 10 s.
        assert_eq!(c.ahorro, Some(300 + 270 - 20));
        let t = estadisticas(&casos, "mates.xlsx").todas;
        assert_eq!((t.notas_llevadas, t.subidas), (54, 2));
        // Nunca menos de nada.
        let poco = vec![caso(1, Que::Exportado { fichero: "m.xlsx".into(), item: None, notas: 2 })];
        assert_eq!(estadisticas(&poco, "m.xlsx").clase.ahorro, Some(0));
    }
}
