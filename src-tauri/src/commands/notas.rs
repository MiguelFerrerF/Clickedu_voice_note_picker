//! Comandos de **poner notas y exportarlas** (H2). Lo que hacen está en
//! [`crate::sesion`] y [`crate::exportar`].

use tauri::{AppHandle, Emitter, Manager};

use crate::commands::clases::leer_plantilla;
use crate::commands::{carpeta_notas, en_segundo_plano, ruta_sesion};
use crate::exportar::{self, Exportado};
use crate::plantillas;
use crate::sesion::{self, Cerrojo};

/// Lo que recibe la interfaz cuando cambia una nota: la lista de Clases
/// vuelve a contar las notas.
pub const EVENTO: &str = "notas-cambiadas";

/// Pone la nota del alumno `clave` de la clase `fichero`, o la quita con
/// `None`, y la guarda al momento. Devuelve la nota tal como ha quedado.
#[tauri::command]
pub async fn poner_nota(fichero: String, clave: String, nota: Option<f64>, app: AppHandle) -> Result<Option<f64>, String> {
    en_segundo_plano(move || {
        if !plantillas::es_plantilla(&fichero) {
            return Err(format!("«{fichero}» no es una plantilla"));
        }
        let ruta = ruta_sesion(&app)?;
        let cerrojo = app.state::<Cerrojo>();
        let _cogido = cerrojo.0.lock().map_err(|e| e.to_string())?;
        let puesta = sesion::poner(&ruta, &fichero, &clave, nota)?;
        let _ = app.emit(EVENTO, ());
        Ok(puesta)
    })
    .await
}

/// Exporta la clase `fichero` a `Documentos\Gestor de Notas\notas`.
#[tauri::command]
pub async fn exportar_clase(fichero: String, app: AppHandle) -> Result<Exportado, String> {
    en_segundo_plano(move || {
        let plantilla = leer_plantilla(&app, &fichero)?;
        let puestas = {
            let cerrojo = app.state::<Cerrojo>();
            let _cogido = cerrojo.0.lock().map_err(|e| e.to_string())?;
            sesion::leer(&ruta_sesion(&app)?)?.puestas(&fichero)
        };
        exportar::exportar(&plantilla, &puestas, &fichero, &carpeta_notas(&app)?).map_err(|e| e.to_string())
    })
    .await
}

/// Abre la carpeta de las notas exportadas en el Explorador (la crea si
/// falta: es la nuestra, D4).
#[tauri::command]
pub async fn abrir_carpeta_notas(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(move || {
        let carpeta = carpeta_notas(&app)?;
        std::fs::create_dir_all(&carpeta).map_err(|e| format!("no se puede crear {}: {e}", carpeta.display()))?;
        std::process::Command::new("explorer")
            .arg(&carpeta)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("no se puede abrir el Explorador: {e}"))
    })
    .await
}
