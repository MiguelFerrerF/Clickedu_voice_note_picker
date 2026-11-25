//! Comandos de **poner notas y exportarlas** (H2). Lo que hacen está en
//! [`crate::sesion`] y [`crate::exportar`].

use tauri::{AppHandle, Emitter, Manager};

use crate::commands::clases::leer_plantilla;
use crate::commands::{carpeta_notas, en_segundo_plano, registro, ruta_sesion};
use crate::exportar::{self, Exportado};
use crate::plantillas;
use crate::registro::Que;
use crate::sesion::{self, Cerrojo, Juego};

/// Lo que recibe la interfaz cuando cambia una nota: la lista de Clases
/// vuelve a contar las notas.
pub const EVENTO: &str = "notas-cambiadas";

/// Pone la nota del alumno `clave` de la clase `fichero` (en el ítem `item`
/// si lo hay, H6), o la quita con `None`, y la guarda al momento. Devuelve
/// la nota tal como ha quedado.
#[tauri::command]
pub async fn poner_nota(
    fichero: String,
    item: Option<String>,
    clave: String,
    nota: Option<f64>,
    app: AppHandle,
) -> Result<Option<f64>, String> {
    en_segundo_plano(app.clone(), "poner_nota", move || {
        if !plantillas::es_plantilla(&fichero) {
            return Err(format!("«{fichero}» no es una plantilla"));
        }
        let ruta = ruta_sesion(&app)?;
        let cerrojo = app.state::<Cerrojo>();
        let _cogido = cerrojo.0.lock().map_err(|e| e.to_string())?;
        let puesta = sesion::poner(&ruta, Juego::de(&fichero, item.as_deref()), &clave, nota)?;
        let _ = app.emit(EVENTO, ());
        Ok(puesta)
    })
    .await
}

/// Devuelve al alumno `clave` a la nota de su plantilla (deshacer un
/// dictado sobre un alumno que no se había tocado).
#[tauri::command]
pub async fn devolver_nota(fichero: String, item: Option<String>, clave: String, app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "devolver_nota", move || {
        let ruta = ruta_sesion(&app)?;
        let cerrojo = app.state::<Cerrojo>();
        let _cogido = cerrojo.0.lock().map_err(|e| e.to_string())?;
        sesion::devolver(&ruta, Juego::de(&fichero, item.as_deref()), &clave)?;
        let _ = app.emit(EVENTO, ());
        Ok(())
    })
    .await
}

/// Exporta la clase `fichero`, con las notas del ítem `item` si lo hay (H6),
/// a `Documentos\Gestor de Notas\notas`.
#[tauri::command]
pub async fn exportar_clase(fichero: String, item: Option<String>, app: AppHandle) -> Result<Exportado, String> {
    en_segundo_plano(app.clone(), "exportar_clase", move || {
        let plantilla = leer_plantilla(&app, &fichero)?;
        let puestas = {
            let cerrojo = app.state::<Cerrojo>();
            let _cogido = cerrojo.0.lock().map_err(|e| e.to_string())?;
            sesion::leer(&ruta_sesion(&app)?)?.puestas_de(Juego::de(&fichero, item.as_deref()))
        };
        let hecho =
            exportar::exportar(&plantilla, &puestas, &fichero, &carpeta_notas(&app)?).map_err(|e| e.to_string())?;
        // Para el tiempo ahorrado de las Estadísticas (H8): sin nombres.
        registro(&app)?.apuntar(Que::Exportado { fichero, item, notas: hecho.con_nota });
        Ok(hecho)
    })
    .await
}

/// Abre la carpeta de las notas exportadas en el Explorador (la crea si
/// falta: es la nuestra, D4).
#[tauri::command]
pub async fn abrir_carpeta_notas(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "abrir_carpeta_notas", move || {
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
