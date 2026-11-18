//! Los comandos que la interfaz puede llamar.
//!
//! Llegan con la pantalla que los necesita, no antes. Hoy están los de las
//! **Clases** y la carpeta de plantillas (H1), y los de poner notas y
//! exportarlas (H2).
//!
//! Regla que no se relaja (library-app): **ningún comando síncrono hace
//! trabajo**. Todos son `async` y lo que toca disco pasa por
//! [`en_segundo_plano`].

pub mod clases;
pub mod notas;

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::ajustes::{self, Ajustes, CarpetaPlantillas};

/// Ejecuta `f` en un hilo de trabajo y espera sin bloquear la ventana.
///
/// Un `fn` síncrono corre en el hilo de la ventana, y la ventana se queda en
/// «No responde».
pub(crate) async fn en_segundo_plano<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("el trabajo en segundo plano se ha caído: {e}"))?
}

/// `ajustes.json`, en la carpeta de la app (D4).
pub(crate) fn ruta_ajustes(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("ajustes.json"))
}

/// `sesion.json`, al lado: las notas puestas (H2).
pub(crate) fn ruta_sesion(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("sesion.json"))
}

/// `Documentos\Gestor de Notas\notas`.
pub(crate) fn carpeta_notas(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(ajustes::carpeta_notas(&app.path().document_dir().map_err(|e| e.to_string())?))
}

pub(crate) fn leer_ajustes(app: &AppHandle) -> Result<Ajustes, String> {
    ajustes::leer(&ruta_ajustes(app)?)
}

/// La carpeta de plantillas según los ajustes de ahora.
pub(crate) fn carpeta(app: &AppHandle) -> Result<CarpetaPlantillas, String> {
    let documentos = app.path().document_dir().map_err(|e| e.to_string())?;
    Ok(ajustes::carpeta_plantillas(&leer_ajustes(app)?, &documentos))
}
