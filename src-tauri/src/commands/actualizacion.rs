//! Comandos de las **actualizaciones** (H7, D7): buscar, descargar e
//! instalar.
//!
//! Con `tauri-plugin-updater`, el oficial: pregunta por el `latest.json` de
//! la última versión publicada en GitHub (el repo es público: no hace falta
//! ningún token) y **comprueba la firma** del instalador con la clave pública
//! de `tauri.conf.json` al descargarlo y otra vez al instalarlo. Con
//! `requireSignedVersion`, la firma lleva además el número de versión: no se
//! puede hacer pasar una versión vieja por nueva.
//!
//! Lo hace todo Rust: la interfaz no tiene ningún permiso del actualizador.
//! La versión encontrada y lo descargado se quedan aquí ([`Pendiente`]) entre
//! un paso y otro. Instalar **cierra la app**: el instalador la sustituye y
//! la vuelve a abrir.

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Manager};
use tauri_plugin_updater::{Error, Update, UpdaterExt};

use crate::actualizacion::{leer_notas, Entrada};
use crate::commands::registro;
use crate::commands::voz::AvanceDescarga;

/// La versión encontrada y, si ya se ha descargado, su instalador.
#[derive(Default)]
pub struct Pendiente(Mutex<Option<(Update, Option<Arc<Vec<u8>>>)>>);

/// Lo que se enseña de una versión nueva.
#[derive(Debug, Serialize)]
pub struct VersionNueva {
    pub version: String,
    pub actual: String,
    /// `AAAA-MM-DD`, o vacía si `latest.json` no la trae.
    pub publicada: String,
    pub titular: String,
    pub entradas: Vec<Entrada>,
}

fn pendiente(app: &AppHandle) -> Result<tauri::State<'_, Pendiente>, String> {
    app.try_state::<Pendiente>().ok_or_else(|| "el actualizador no está".to_string())
}

/// Pregunta si hay una versión nueva. `None` si está al día. Un fallo de red
/// llega como error y **no** se apunta en el registro: sin conexión es lo
/// normal en un portátil.
#[tauri::command]
pub async fn buscar_actualizacion(app: AppHandle) -> Result<Option<VersionNueva>, String> {
    let update = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?;
    let Some(update) = update else {
        *pendiente(&app)?.0.lock().map_err(|e| e.to_string())? = None;
        return Ok(None);
    };
    let notas = leer_notas(update.body.as_deref().unwrap_or(""));
    let nueva = VersionNueva {
        version: update.version.clone(),
        actual: update.current_version.clone(),
        publicada: update.date.map(|d| d.date().to_string()).unwrap_or_default(),
        titular: notas.titular,
        entradas: notas.entradas,
    };
    *pendiente(&app)?.0.lock().map_err(|e| e.to_string())? = Some((update, None));
    Ok(Some(nueva))
}

/// Descarga la versión encontrada, con avance, y comprueba su firma. Si no
/// casa, no se guarda nada y se dice.
#[tauri::command]
pub async fn descargar_actualizacion(app: AppHandle, avance: Channel<AvanceDescarga>) -> Result<(), String> {
    let hecho = descargar(&app, &avance).await;
    if let (Err(e), Ok(r)) = (&hecho, registro(&app)) {
        r.fallo("descargar_actualizacion", e);
    }
    hecho
}

async fn descargar(app: &AppHandle, avance: &Channel<AvanceDescarga>) -> Result<(), String> {
    let update = {
        let p = pendiente(app)?;
        let cogido = p.0.lock().map_err(|e| e.to_string())?;
        cogido.as_ref().map(|(u, _)| u.clone()).ok_or("no hay ninguna versión nueva que descargar")?
    };
    // Un aviso cada 256 KB: con cada trozo serían miles.
    let (mut hecho, mut avisado) = (0u64, 0u64);
    let bytes = update
        .download(
            |trozo, total| {
                hecho += trozo as u64;
                if hecho - avisado >= 256 * 1024 || Some(hecho) == total {
                    avisado = hecho;
                    let _ = avance.send(AvanceDescarga { hecho, total: total.unwrap_or(0) });
                }
            },
            || {},
        )
        .await
        .map_err(|e| match e {
            Error::Minisign(_)
            | Error::SignatureUtf8(_)
            | Error::Base64(_)
            | Error::SignedVersionMismatch { .. }
            | Error::MissingSignedVersion => {
                format!("la firma de la v{} no es la nuestra: no se ha instalado nada ({e})", update.version)
            }
            otro => otro.to_string(),
        })?;
    let p = pendiente(app)?;
    let mut cogido = p.0.lock().map_err(|e| e.to_string())?;
    match cogido.as_mut() {
        Some((u, guardado)) if u.version == update.version => *guardado = Some(Arc::new(bytes)),
        _ => return Err("la versión nueva ha cambiado mientras se descargaba: vuelve a buscar".into()),
    }
    Ok(())
}

/// Instala lo descargado. La firma se vuelve a comprobar; si sale bien, **la
/// app se cierra** y el instalador la abre ya actualizada, así que esto no
/// llega a contestar.
#[tauri::command]
pub async fn instalar_actualizacion(app: AppHandle) -> Result<(), String> {
    let (update, bytes) = {
        let p = pendiente(&app)?;
        let cogido = p.0.lock().map_err(|e| e.to_string())?;
        match cogido.as_ref() {
            Some((u, Some(b))) => (u.clone(), b.clone()),
            _ => return Err("no hay nada descargado que instalar".into()),
        }
    };
    let hecho =
        tauri::async_runtime::spawn_blocking(move || update.install(bytes.as_slice()).map_err(|e| e.to_string()))
            .await
            .map_err(|e| format!("el trabajo en segundo plano se ha caído: {e}"))?;
    if let (Err(e), Ok(r)) = (&hecho, registro(&app)) {
        r.fallo("instalar_actualizacion", e);
    }
    hecho
}
