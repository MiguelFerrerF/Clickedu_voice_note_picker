//! El comando de las **Estadísticas** (H8). Lo que cuentan está en
//! [`crate::estadisticas`] (las notas) y [`crate::registro::estadisticas`]
//! (el dictado).

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::clases::{items_por_fichero, leer_plantilla};
use crate::commands::{en_segundo_plano, registro, ruta_sesion};
use crate::estadisticas::{self, Notas};
use crate::registro::estadisticas::{self as dictado, EstadisticasDictado};
use crate::sesion::{self, Cerrojo, Juego};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estadisticas {
    /// Las de la clase y el ítem.
    pub notas: Notas,
    /// Lo del dictado: de la clase (en todos sus ítems) y de todas.
    pub dictado: EstadisticasDictado,
}

/// Las Estadísticas de la clase `fichero`, con las notas del ítem `item`
/// si lo hay (H6).
#[tauri::command]
pub async fn leer_estadisticas(fichero: String, item: Option<String>, app: AppHandle) -> Result<Estadisticas, String> {
    en_segundo_plano(app.clone(), "leer_estadisticas", move || {
        let plantilla = leer_plantilla(&app, &fichero)?;
        let puestas = {
            let cerrojo = app.state::<Cerrojo>();
            let _cogido = cerrojo.0.lock().map_err(|e| e.to_string())?;
            sesion::leer(&ruta_sesion(&app)?)?.puestas_de(Juego::de(&fichero, item.as_deref()))
        };
        // En un ítem, lo que ya tenía en Clickedu al traer (H6).
        let en_clickedu = item.as_deref().and_then(|item| {
            let mut m = items_por_fichero(&app).remove(&fichero.to_lowercase())?;
            let i = m.evaluaciones.iter_mut().flat_map(|e| e.items.iter_mut()).find(|i| i.id == item)?;
            Some(std::mem::take(&mut i.notas))
        });
        Ok(Estadisticas {
            notas: estadisticas::de_la_clase(&plantilla.alumnos, &puestas, en_clickedu.as_ref()),
            dictado: dictado::estadisticas(&registro(&app)?.leer()?, &fichero),
        })
    })
    .await
}
