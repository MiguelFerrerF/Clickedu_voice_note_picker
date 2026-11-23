//! Comandos de las **Clases** (H1): la lista de plantillas de la carpeta, leer
//! una, y elegir la carpeta. Lo que hacen está en [`crate::plantillas`] y
//! [`crate::ajustes`].

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::ajustes::{self, CarpetaPlantillas};
use crate::clickedu::items::{self, ItemsClase, ItemsMateria};
use crate::clickedu::traer::aplicar;
use crate::commands::clickedu::carpeta_clickedu;
use crate::commands::{carpeta, en_segundo_plano, leer_ajustes, registro, ruta_ajustes, ruta_sesion};
use crate::plantillas::{self, Clase, Plantilla};
use crate::registro::{ahora, fecha};
use crate::sesion::{self, Juego, Puestas};
use crate::vigilante::{self, Vigilante};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListaClases {
    pub carpeta: CarpetaPlantillas,
    pub clases: Vec<ClaseConItems>,
}

/// Una clase y sus ítems de Clickedu (H6).
#[derive(Debug, Serialize)]
pub struct ClaseConItems {
    #[serde(flatten)]
    pub clase: Clase,
    /// Los guardados al traer, si la clase es de una materia traída;
    /// `None` si no (quien no se conecta, o una plantilla puesta a mano):
    /// se califica sin ítem, como en la v1.
    pub items: Option<ItemsClase>,
}

/// Las clases de la carpeta de plantillas, con las notas de la sesión
/// contadas (sin ítem y en cada ítem). Si la carpeta no existe, sin clases
/// y `carpeta.existe` a `false`.
#[tauri::command]
pub async fn listar_clases(app: AppHandle) -> Result<ListaClases, String> {
    en_segundo_plano(app.clone(), "listar_clases", move || {
        let carpeta = carpeta(&app)?;
        let sesion = sesion::leer(&ruta_sesion(&app)?)?;
        let de_materias = items_por_fichero(&app);
        // Las notas de cada ítem (en la sesión y en Clickedu) se cuentan con
        // la plantilla, que solo se lee una vez: dentro de `listar`.
        let contados: RefCell<HashMap<String, HashMap<String, (usize, usize)>>> = RefCell::default();
        let clases = if carpeta.existe {
            plantillas::listar(Path::new(&carpeta.ruta), |f, p| {
                if let Some(m) = de_materias.get(&f.to_lowercase()) {
                    let por_item = m
                        .evaluaciones
                        .iter()
                        .flat_map(|e| &e.items)
                        .map(|i| {
                            let calificados = sesion::calificados(p, &sesion.puestas_de(Juego::Item(&i.id)));
                            let en_clickedu = items::en_la_clase(i, p.alumnos.iter().map(|a| a.id.as_str()));
                            (i.id.clone(), (calificados, en_clickedu))
                        })
                        .collect();
                    contados.borrow_mut().insert(f.to_string(), por_item);
                }
                sesion::calificados(p, &sesion.puestas(f))
            })
            .map_err(|e| format!("no se puede leer la carpeta {}: {e}", carpeta.ruta))?
        } else {
            Vec::new()
        };
        let (a, m, d) = fecha(ahora());
        let hoy = format!("{a:04}-{m:02}-{d:02}");
        let contados = contados.into_inner();
        let clases = clases
            .into_iter()
            .map(|clase| {
                let items = de_materias.get(&clase.fichero.to_lowercase()).map(|m| {
                    let de_la_clase = contados.get(&clase.fichero);
                    items::para_clases(m, &hoy, |i| de_la_clase.and_then(|c| c.get(&i.id)).copied().unwrap_or((0, 0)))
                });
                ClaseConItems { clase, items }
            })
            .collect();
        Ok(ListaClases { carpeta, clases })
    })
    .await
}

/// Los ítems guardados al traer, por fichero (en minúsculas: Windows no
/// las distingue), cruzando `items.json` con `traidas.json`. Si no se
/// pueden leer, sin ítems y apuntado en el registro: la lista de clases
/// sale igual.
fn items_por_fichero(app: &AppHandle) -> HashMap<String, ItemsMateria> {
    let leidos = carpeta_clickedu(app).and_then(|c| {
        let recordadas = aplicar::leer_recordadas(&c.join("traidas.json"))?;
        let guardados = items::leer(&c.join(items::FICHERO))?;
        Ok((recordadas, guardados))
    });
    match leidos {
        Ok((recordadas, mut guardados)) => recordadas
            .materias
            .into_iter()
            .filter_map(|(id, fichero)| guardados.materias.remove(&id).map(|m| (fichero.to_lowercase(), m)))
            .collect(),
        Err(e) => {
            if let Ok(r) = registro(app) {
                r.fallo("listar_clases", &e);
            }
            HashMap::new()
        }
    }
}

/// Una clase abierta para calificar: la plantilla tal cual y las notas que
/// lleva puestas la sesión (H2). La interfaz las junta para pintar.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaseAbierta {
    pub plantilla: Plantilla,
    pub puestas: Puestas,
    /// En un ítem (H6), las notas que tenían en Clickedu al traer, por `Id`
    /// de alumno: se enseñan junto a las de la app y se avisa de las que se
    /// pisarían al subir. `None` sin ítem, o si no se encuentra.
    pub en_clickedu: Option<BTreeMap<String, String>>,
}

/// Lee la plantilla `fichero` de la carpeta y sus notas de la sesión: las
/// del ítem `item` si lo hay (H6), o las de sin ítem.
#[tauri::command]
pub async fn leer_clase(fichero: String, item: Option<String>, app: AppHandle) -> Result<ClaseAbierta, String> {
    en_segundo_plano(app.clone(), "leer_clase", move || {
        let plantilla = leer_plantilla(&app, &fichero)?;
        let puestas = sesion::leer(&ruta_sesion(&app)?)?.puestas_de(Juego::de(&fichero, item.as_deref()));
        let en_clickedu = item.as_deref().and_then(|item| {
            let mut m = items_por_fichero(&app).remove(&fichero.to_lowercase())?;
            let i = m.evaluaciones.iter_mut().flat_map(|e| e.items.iter_mut()).find(|i| i.id == item)?;
            Some(std::mem::take(&mut i.notas))
        });
        Ok(ClaseAbierta { plantilla, puestas, en_clickedu })
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
pub(crate) fn cambiar_carpeta(app: &AppHandle, ruta: Option<PathBuf>) -> Result<CarpetaPlantillas, String> {
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
