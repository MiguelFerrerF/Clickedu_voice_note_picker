//! Vigila la carpeta de plantillas (H1): si se añade, cambia o quita un
//! Excel, la interfaz recibe `plantillas-cambiadas` y vuelve a pedir la lista.
//!
//! Solo la carpeta, sin subcarpetas, y solo los ficheros que son plantilla
//! ([`crate::plantillas::es_plantilla`]): el `~$…` que deja Excel al abrir un
//! libro no cuenta. Con *debounce* de medio segundo, porque copiar o guardar
//! un Excel toca el fichero varias veces.

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use tauri::{AppHandle, Emitter};

use crate::plantillas::es_plantilla;

pub const EVENTO: &str = "plantillas-cambiadas";

/// El vigilante de ahora. Al cambiar de carpeta se suelta el viejo, y con él
/// deja de vigilar.
#[derive(Default)]
pub struct Vigilante(pub Mutex<Option<Debouncer<RecommendedWatcher>>>);

/// Empieza a vigilar `carpeta`, dejando la de antes. Si no se puede (la
/// carpeta no existe), se queda sin vigilar: la lista se sigue pidiendo al
/// entrar en la pantalla.
pub fn vigilar(app: &AppHandle, vigilante: &Vigilante, carpeta: &Path) -> Result<(), String> {
    let mut actual = vigilante.0.lock().map_err(|e| e.to_string())?;
    *actual = None;
    let avisar = app.clone();
    let mut d = new_debouncer(Duration::from_millis(500), move |r: DebounceEventResult| {
        let toca = match r {
            Ok(eventos) => eventos.iter().any(|e| {
                e.path.file_name().map(|n| es_plantilla(&n.to_string_lossy())).unwrap_or(false)
            }),
            // Si el vigilante falla, mejor una lista de más que una de menos.
            Err(_) => true,
        };
        if toca {
            let _ = avisar.emit(EVENTO, ());
        }
    })
    .map_err(|e| e.to_string())?;
    d.watcher().watch(carpeta, RecursiveMode::NonRecursive).map_err(|e| e.to_string())?;
    *actual = Some(d);
    Ok(())
}
