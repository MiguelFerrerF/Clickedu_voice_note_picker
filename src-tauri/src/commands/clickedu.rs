//! Comandos de **Clickedu** (H4): la cuenta de la pantalla Conexión. Lo que
//! hacen está en [`crate::clickedu`].
//!
//! - [`cuenta_clickedu`]: la cuenta guardada, **sin la contraseña**.
//! - [`elegir_archivo_seguridad`]: el selector de archivo. Solo dice cuál se
//!   ha elegido: no se copia hasta haber entrado con él.
//! - [`entrar_clickedu`]: entra con lo que se le da y, lo que falte, lo
//!   toma de la cuenta guardada. **Solo guarda si ha entrado.** La sesión
//!   queda en [`Sesion`] para traer plantillas (H5) y subir notas (H6).
//! - [`olvidar_cuenta_clickedu`]: borra la cuenta y cierra la sesión.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::clickedu::cliente::{Cliente, Fallo, BASE};
use crate::clickedu::cuenta::{self, LlaveroWindows, Vista};
use crate::clickedu::guardar_pagina;
use crate::commands::{en_segundo_plano, registro};
use crate::registro::ahora;

/// La sesión con Clickedu, si se ha entrado.
#[derive(Default)]
pub struct Sesion(pub tauri::async_runtime::Mutex<Option<Cliente>>);

/// `%APPDATA%\es.gestornotas.app\clickedu`.
fn carpeta_clickedu(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("clickedu"))
}

/// Lo que toca disco o el Administrador de credenciales, fuera del hilo de
/// la ventana.
async fn bloqueante<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("el trabajo en segundo plano se ha caído: {e}"))?
}

/// La cuenta guardada, o `None`.
#[tauri::command]
pub async fn cuenta_clickedu(app: AppHandle) -> Result<Option<Vista>, String> {
    en_segundo_plano(app.clone(), "cuenta_clickedu", move || {
        Ok(cuenta::leer(&carpeta_clickedu(&app)?, &LlaveroWindows)?.map(|g| g.vista()))
    })
    .await
}

/// Abre el selector para el archivo de seguridad. `None` si se cancela.
#[tauri::command]
pub async fn elegir_archivo_seguridad(app: AppHandle) -> Result<Option<String>, String> {
    en_segundo_plano(app.clone(), "elegir_archivo_seguridad", move || {
        Ok(app
            .dialog()
            .file()
            .set_title("Elige tu archivo de seguridad de Clickedu")
            .add_filter("Archivo de seguridad", &["txt"])
            .add_filter("Todos los archivos", &["*"])
            .blocking_pick_file()
            .and_then(|p| p.into_path().ok())
            .map(|p| p.display().to_string()))
    })
    .await
}

/// Entra en Clickedu y, si entra, guarda la cuenta. `usuario`,
/// `contrasena` y `archivo` (la ruta del elegido) son lo nuevo; lo que
/// venga a `None`, de la cuenta guardada.
#[tauri::command]
pub async fn entrar_clickedu(
    usuario: Option<String>,
    contrasena: Option<String>,
    archivo: Option<String>,
    app: AppHandle,
) -> Result<Vista, Fallo> {
    let hecho = entrar(&app, usuario, contrasena, archivo).await;
    if let (Err(e), Ok(r)) = (&hecho, registro(&app)) {
        r.fallo("entrar_clickedu", &e.to_string());
    }
    hecho
}

async fn entrar(
    app: &AppHandle,
    usuario: Option<String>,
    contrasena: Option<String>,
    archivo: Option<String>,
) -> Result<Vista, Fallo> {
    let carpeta = carpeta_clickedu(app)?;
    let c = carpeta.clone();
    let guardada = bloqueante(move || cuenta::leer(&c, &LlaveroWindows)).await?;

    let usuario = usuario
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .or_else(|| guardada.as_ref().map(|g| g.usuario.clone()))
        .ok_or_else(|| "falta el usuario".to_string())?;
    let contrasena = contrasena
        .filter(|c| !c.is_empty())
        .or_else(|| guardada.as_ref().map(|g| g.contrasena.clone()))
        .ok_or_else(|| "falta la contraseña".to_string())?;
    let nuevo = archivo.is_some();
    let ruta = archivo
        .map(PathBuf::from)
        .or_else(|| guardada.as_ref().and_then(|g| g.archivo.clone()))
        .ok_or_else(|| "falta el archivo de seguridad: elígelo".to_string())?;
    let (nombre, bytes) = bloqueante(move || cuenta::leer_archivo(&ruta)).await?;

    let cliente = Cliente::new(BASE)?;
    if let Err(e) = cliente.entrar(&usuario, &contrasena, (&nombre, &bytes)).await {
        // La página, en local: la que no se entiende, y la del archivo que
        // no vale, para distinguir un día «caducado» de «incorrecto».
        let pagina = match &e {
            Fallo::NoEntiendo { paso, html } => Some((paso.clone(), html.clone())),
            Fallo::Archivo { html } => Some(("archivo".to_string(), html.clone())),
            _ => None,
        };
        if let Some((paso, html)) = pagina {
            let c = carpeta.clone();
            let _ = bloqueante(move || guardar_pagina(&c, &paso, &html, ahora())).await;
        }
        return Err(e);
    }

    let vista = bloqueante(move || {
        let archivo = nuevo.then_some((nombre.as_str(), bytes.as_slice()));
        cuenta::guardar(&carpeta, &LlaveroWindows, &usuario, &contrasena, archivo, ahora()).map(|g| g.vista())
    })
    .await?;
    *app.state::<Sesion>().0.lock().await = Some(cliente);
    Ok(vista)
}

/// Olvida la cuenta (la contraseña del Administrador de credenciales y la
/// copia del archivo) y cierra la sesión.
#[tauri::command]
pub async fn olvidar_cuenta_clickedu(app: AppHandle) -> Result<(), String> {
    *app.state::<Sesion>().0.lock().await = None;
    en_segundo_plano(app.clone(), "olvidar_cuenta_clickedu", move || {
        cuenta::olvidar(&carpeta_clickedu(&app)?, &LlaveroWindows)
    })
    .await
}
