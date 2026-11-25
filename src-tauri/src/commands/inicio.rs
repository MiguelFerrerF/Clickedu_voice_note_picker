//! Lo del **Inicio** (HUI, 2026-11-25) que no sale de otros comandos: la
//! última clase abierta en Calificar, para seguir donde se dejó, y el
//! dictado de todas las clases en dos cifras. Cómo van las clases y lo
//! pendiente con Clickedu salen de `listar_clases`.

use tauri::AppHandle;

use super::{en_segundo_plano, leer_ajustes, registro, ruta_ajustes, AJUSTES_POR_TURNOS};
use crate::ajustes::{self, Ultima};
use crate::registro::estadisticas::{self as dictado, Dictados};

/// La última clase abierta en Calificar, si la hay.
#[tauri::command]
pub async fn ultima_clase(app: AppHandle) -> Result<Option<Ultima>, String> {
    en_segundo_plano(app.clone(), "ultima_clase", move || Ok(leer_ajustes(&app)?.ultima)).await
}

/// Recuerda `ultima` como la última clase abierta. Si ya lo era, no escribe.
#[tauri::command]
pub async fn recordar_clase(ultima: Option<Ultima>, app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "recordar_clase", move || {
        let _turno = AJUSTES_POR_TURNOS.lock().unwrap_or_else(|e| e.into_inner());
        let mut a = leer_ajustes(&app)?;
        if a.ultima == ultima {
            return Ok(());
        }
        a.ultima = ultima;
        ajustes::guardar(&ruta_ajustes(&app)?, &a)
    })
    .await
}

/// Lo del dictado de todas las clases: lo mismo que «todas» en Estadísticas.
#[tauri::command]
pub async fn resumen_dictado(app: AppHandle) -> Result<Dictados, String> {
    en_segundo_plano(app.clone(), "resumen_dictado", move || {
        Ok(dictado::estadisticas(&registro(&app)?.leer()?, "").todas)
    })
    .await
}
