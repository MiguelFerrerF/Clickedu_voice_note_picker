//! La ayuda (H9): lo que ya se ha visto, para no enseñarlo otra vez. Vive en
//! `ajustes.json` (`ayudaVista`), no en el `localStorage` de la ventana:
//! así sobrevive a reinstalar y a actualizar.

use std::sync::Mutex;

use tauri::AppHandle;

use super::{en_segundo_plano, leer_ajustes, ruta_ajustes};
use crate::ajustes;

/// Dos avisos cerrados a la vez leen y escriben `ajustes.json` por turnos:
/// si no, el segundo pisaría al primero.
static POR_TURNOS: Mutex<()> = Mutex::new(());

/// Lo ya visto: `bienvenida` y las pantallas cuyo aviso se ha cerrado.
#[tauri::command]
pub async fn ayuda_vista(app: AppHandle) -> Result<Vec<String>, String> {
    en_segundo_plano(app.clone(), "ayuda_vista", move || Ok(leer_ajustes(&app)?.ayuda_vista)).await
}

/// Da por visto `clave`. Si ya lo estaba, no escribe.
#[tauri::command]
pub async fn marcar_ayuda(clave: String, app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "marcar_ayuda", move || {
        let _turno = POR_TURNOS.lock().unwrap_or_else(|e| e.into_inner());
        let mut a = leer_ajustes(&app)?;
        if a.ayuda_vista.contains(&clave) {
            return Ok(());
        }
        a.ayuda_vista.push(clave);
        ajustes::guardar(&ruta_ajustes(&app)?, &a)
    })
    .await
}
