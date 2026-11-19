//! Comandos de las **Clases** (H1): la lista de plantillas de la carpeta, leer
//! una, y elegir la carpeta. Lo que hacen está en [`crate::plantillas`] y
//! [`crate::ajustes`].

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::ajustes::{self, CarpetaPlantillas};
use crate::commands::{carpeta, en_segundo_plano, leer_ajustes, ruta_ajustes, ruta_sesion};
use crate::plantillas::{self, Clase, Plantilla};
use crate::sesion::{self, Puestas};
use crate::vigilante::{self, Vigilante};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListaClases {
    pub carpeta: CarpetaPlantillas,
    pub clases: Vec<Clase>,
}

/// Las clases de la carpeta de plantillas, con las notas de la sesión
/// contadas. Si la carpeta no existe, sin clases y `carpeta.existe` a
/// `false`.
#[tauri::command]
pub async fn listar_clases(app: AppHandle) -> Result<ListaClases, String> {
    en_segundo_plano(app.clone(), "listar_clases", move || {
        let carpeta = carpeta(&app)?;
        let sesion = sesion::leer(&ruta_sesion(&app)?)?;
        let clases = if carpeta.existe {
            plantillas::listar(Path::new(&carpeta.ruta), |f, p| sesion::calificados(p, &sesion.puestas(f)))
                .map_err(|e| format!("no se puede leer la carpeta {}: {e}", carpeta.ruta))?
        } else {
            Vec::new()
        };
        Ok(ListaClases { carpeta, clases })
    })
    .await
}

/// Una clase abierta para calificar: la plantilla tal cual y las notas que
/// lleva puestas la sesión (H2). La interfaz las junta para pintar.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaseAbierta {
    pub plantilla: Plantilla,
    pub puestas: Puestas,
}

/// Lee la plantilla `fichero` de la carpeta y sus notas de la sesión.
#[tauri::command]
pub async fn leer_clase(fichero: String, app: AppHandle) -> Result<ClaseAbierta, String> {
    en_segundo_plano(app.clone(), "leer_clase", move || {
        let plantilla = leer_plantilla(&app, &fichero)?;
        let puestas = sesion::leer(&ruta_sesion(&app)?)?.puestas(&fichero);
        Ok(ClaseAbierta { plantilla, puestas })
    })
    .await
}

/// Lee la plantilla `fichero`. Solo un nombre de fichero, nunca una ruta: no
/// se lee nada fuera de la carpeta.
pub(crate) fn leer_plantilla(app: &AppHandle, fichero: &str) -> Result<Plantilla, String> {
    let nombre = Path::new(fichero);
    if nombre.file_name() != Some(nombre.as_os_str()) || !plantillas::es_plantilla(fichero) {
        return Err(format!("«{fichero}» no es una plantilla de la carpeta"));
    }
    let ruta = PathBuf::from(carpeta(app)?.ruta).join(nombre);
    plantillas::leer(&ruta).map_err(|e| format!("{fichero}: {e}"))
}

/// Abre el selector de carpeta. Si se elige una, se guarda, se vigila y la
/// interfaz recibe `plantillas-cambiadas`. `None` si se cancela.
#[tauri::command]
pub async fn elegir_carpeta_plantillas(app: AppHandle) -> Result<Option<CarpetaPlantillas>, String> {
    en_segundo_plano(app.clone(), "elegir_carpeta_plantillas", move || {
        let elegida = app
            .dialog()
            .file()
            .set_title("Elige la carpeta de tus plantillas de Clickedu")
            .blocking_pick_folder()
            .and_then(|p| p.into_path().ok());
        match elegida {
            Some(ruta) => cambiar_carpeta(&app, Some(ruta)).map(Some),
            None => Ok(None),
        }
    })
    .await
}

/// Vuelve a `Documentos\Gestor de Notas\plantillas`.
#[tauri::command]
pub async fn restablecer_carpeta_plantillas(app: AppHandle) -> Result<CarpetaPlantillas, String> {
    en_segundo_plano(app.clone(), "restablecer_carpeta_plantillas", move || cambiar_carpeta(&app, None)).await
}

/// Abre la carpeta de plantillas en el Explorador.
#[tauri::command]
pub async fn abrir_carpeta_plantillas(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "abrir_carpeta_plantillas", move || {
        let carpeta = carpeta(&app)?;
        if !carpeta.existe {
            return Err(format!("la carpeta {} no existe", carpeta.ruta));
        }
        std::process::Command::new("explorer")
            .arg(&carpeta.ruta)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("no se puede abrir el Explorador: {e}"))
    })
    .await
}

/// Guarda la carpeta nueva, la vigila y avisa a la interfaz.
fn cambiar_carpeta(app: &AppHandle, ruta: Option<PathBuf>) -> Result<CarpetaPlantillas, String> {
    let mut a = leer_ajustes(app)?;
    a.carpeta_plantillas = ruta;
    ajustes::guardar(&ruta_ajustes(app)?, &a)?;
    let c = carpeta(app)?;
    empezar_a_vigilar(app, &c);
    let _ = app.emit(vigilante::EVENTO, ());
    Ok(c)
}

/// Vigila la carpeta si existe. Que no se pueda vigilar no es un error que
/// parar la app: la lista se pide igual al entrar en la pantalla.
pub(crate) fn empezar_a_vigilar(app: &AppHandle, carpeta: &CarpetaPlantillas) {
    if let Some(v) = app.try_state::<Vigilante>() {
        let _ = if carpeta.existe {
            vigilante::vigilar(app, &v, Path::new(&carpeta.ruta))
        } else {
            v.0.lock().map(|mut d| *d = None).map_err(|e| e.to_string())
        };
    }
}
