//! Comandos del **registro** (D8): apuntar lo que hizo el profesor con un
//! dictado, ver cuánto hay, preparar el informe y borrarlo. Lo que hacen
//! está en [`crate::registro`].

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::ajustes;
use crate::commands::{carpeta, en_segundo_plano, registro};
use crate::plantillas;
use crate::registro::informe::{self, Tapar};
use crate::registro::{self as reg, Que, Respuesta, Resumen};

/// Lo que hizo el profesor con el dictado `id`: la respuesta buena, que es
/// lo que hace útil el registro.
#[tauri::command]
pub async fn responder_dictado(id: String, respuesta: Respuesta, app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "responder_dictado", move || {
        registro(&app)?.apuntar(Que::Respuesta { id, respuesta });
        Ok(())
    })
    .await
}

/// Cuánto hay en el registro, para Ajustes.
#[tauri::command]
pub async fn resumen_registro(app: AppHandle) -> Result<Resumen, String> {
    en_segundo_plano(app.clone(), "resumen_registro", move || registro(&app)?.resumen()).await
}

/// Prepara el informe en `Documentos\Gestor de Notas\informes`, sin nombres
/// ni voz, y devuelve su ruta. Uno por día: el de hoy se rehace con todo.
#[tauri::command]
pub async fn preparar_informe(app: AppHandle) -> Result<String, String> {
    en_segundo_plano(app.clone(), "preparar_informe", move || {
        let casos = registro(&app)?.leer()?;
        let tapar = Tapar::new(&nombres_que_tapar(&app, &casos), rutas_que_tapar(&app));
        let ahora = reg::ahora();
        let texto =
            serde_json::to_string_pretty(&informe::generar(&casos, &tapar, ahora)).map_err(|e| e.to_string())?;
        let carpeta = carpeta_informes(&app)?;
        std::fs::create_dir_all(&carpeta).map_err(|e| format!("no se puede crear {}: {e}", carpeta.display()))?;
        let ruta = carpeta.join(format!("informe-{}.json", reg::dia(ahora)));
        std::fs::write(&ruta, texto).map_err(|e| format!("no se puede escribir {}: {e}", ruta.display()))?;
        Ok(ruta.to_string_lossy().into_owned())
    })
    .await
}

/// Enseña el informe `ruta` en el Explorador, seleccionado, para
/// adjuntarlo. Solo uno de la carpeta de informes.
#[tauri::command]
pub async fn mostrar_informe(ruta: String, app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "mostrar_informe", move || {
        let ruta = PathBuf::from(ruta);
        let es_informe = ruta.parent() == Some(carpeta_informes(&app)?.as_path())
            && ruta.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("informe-"));
        if !es_informe || !ruta.exists() {
            return Err(format!("{} no es un informe", ruta.display()));
        }
        std::process::Command::new("explorer")
            .raw_arg(format!("/select,\"{}\"", ruta.display()))
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("no se puede abrir el Explorador: {e}"))
    })
    .await
}

/// Borra todo el registro.
#[tauri::command]
pub async fn borrar_registro(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "borrar_registro", move || registro(&app)?.borrar()).await
}

fn carpeta_informes(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(ajustes::carpeta_informes(&app.path().document_dir().map_err(|e| e.to_string())?))
}

/// Los nombres de los alumnos de las plantillas de la carpeta, y lo que se
/// tomó por nombre en cada dictado: lo que el informe tapa en los errores.
fn nombres_que_tapar(app: &AppHandle, casos: &[reg::Caso]) -> Vec<String> {
    let mut nombres = Vec::new();
    if let Ok(c) = carpeta(app) {
        let entradas = std::fs::read_dir(&c.ruta).into_iter().flatten().filter_map(|e| e.ok());
        for ruta in entradas.map(|e| e.path()) {
            let es = ruta.file_name().and_then(|n| n.to_str()).is_some_and(plantillas::es_plantilla);
            if let Some(p) = es.then(|| plantillas::leer(&ruta).ok()).flatten() {
                nombres.extend(p.alumnos.into_iter().map(|a| a.nombre));
            }
        }
    }
    for caso in casos {
        if let Que::Dictado { nombre, .. } = &caso.que {
            nombres.extend(nombre.iter().cloned());
        }
    }
    nombres
}

/// Las rutas de este ordenador, que dicen quién es el profesor y dónde
/// trabaja.
fn rutas_que_tapar(app: &AppHandle) -> Vec<(String, String)> {
    let texto = |p: &Path| p.to_string_lossy().into_owned();
    let mut rutas = Vec::new();
    if let Ok(c) = carpeta(app) {
        rutas.push((c.ruta, "%PLANTILLAS%".to_string()));
    }
    if let Ok(d) = app.path().document_dir() {
        rutas.push((texto(&d), "%DOCUMENTOS%".to_string()));
    }
    if let Ok(h) = app.path().home_dir() {
        rutas.push((texto(&h), "~".to_string()));
    }
    rutas
}
