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
//! - [`traer_plantillas`] (H5): baja una plantilla por materia y planea qué
//!   hacer con ellas y con lo que ya hay en la carpeta. La primera vez, o
//!   si hay dudas, **espera** a [`confirmar_traer`] (o a
//!   [`cancelar_traer`]); si no, lo hace y lo cuenta. Sin sesión, entra
//!   con la cuenta guardada. [`replanear_traer`] enseña lo que haría con
//!   las respuestas de hasta ahora, sin hacerlo.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::clickedu::cliente::{self, Cliente, Fallo, BASE};
use crate::clickedu::cuenta::{self, LlaveroWindows, Vista};
use crate::clickedu::fichas::Estado;
use crate::clickedu::guardar_pagina;
use crate::clickedu::huella::{self, Huella};
use crate::clickedu::items::{self, ItemsMateria};
use crate::clickedu::tapar::Tapar;
use crate::clickedu::traer::aplicar::{self, Hecho, Rutas};
use crate::clickedu::traer::planear::{self, Contenido, EnLaRaiz, Plan, Respuesta, Traida};
use crate::clickedu::traer::{self, Bajado, FalloBajada, FalloTraer};
use crate::commands::{carpeta, en_segundo_plano, registro, ruta_sesion};
use crate::registro::{ahora, fecha};
use crate::{sesion, vigilante};

/// La sesión con Clickedu, si se ha entrado.
#[derive(Default)]
pub struct Sesion(pub tauri::async_runtime::Mutex<Option<Cliente>>);

/// `%APPDATA%\es.gestornotas.app\clickedu`.
pub(crate) fn carpeta_clickedu(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("clickedu"))
}

/// Lo que toca disco o el Administrador de credenciales, fuera del hilo de
/// la ventana.
pub(crate) async fn bloqueante<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
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

    let cliente = conectar(&carpeta, &usuario, &contrasena, (nombre.clone(), bytes.clone())).await?;
    let vista = bloqueante(move || {
        let archivo = nuevo.then_some((nombre.as_str(), bytes.as_slice()));
        cuenta::guardar(&carpeta, &LlaveroWindows, &usuario, &contrasena, archivo, ahora()).map(|g| g.vista())
    })
    .await?;
    *app.state::<Sesion>().0.lock().await = Some(cliente);
    Ok(vista)
}

/// Un cliente nuevo que entra en Clickedu. Si no entra, la página va a
/// local: la que no se entiende, y la del archivo que no vale, para
/// distinguir un día «caducado» de «incorrecto».
async fn conectar(
    carpeta: &Path,
    usuario: &str,
    contrasena: &str,
    (nombre, bytes): (String, Vec<u8>),
) -> Result<Cliente, Fallo> {
    let cliente = Cliente::new(BASE)?;
    if let Err(e) = cliente.entrar(usuario, contrasena, (&nombre, &bytes)).await {
        let pagina = match &e {
            Fallo::NoEntiendo { paso, html } => Some((paso.clone(), html.clone())),
            Fallo::Archivo { html } => Some(("archivo".to_string(), html.clone())),
            _ => None,
        };
        if let Some((paso, html)) = pagina {
            let c = carpeta.to_path_buf();
            let _ = bloqueante(move || guardar_pagina(&c, &paso, &html, ahora())).await;
        }
        return Err(e);
    }
    Ok(cliente)
}

/// Entra con la cuenta guardada, sin volver a guardarla: la app entra sola
/// cuando hace falta (H4). Sin cuenta, [`FalloTraer::SinEntrar`].
pub(crate) async fn entrar_con_la_guardada(carpeta: &Path) -> Result<Cliente, FalloTraer> {
    let c = carpeta.to_path_buf();
    let Some(g) = bloqueante(move || cuenta::leer(&c, &LlaveroWindows)).await? else {
        return Err(FalloTraer::SinEntrar);
    };
    let ruta = g.archivo.ok_or(FalloTraer::SinEntrar)?;
    let archivo = bloqueante(move || cuenta::leer_archivo(&ruta)).await?;
    Ok(conectar(carpeta, &g.usuario, &g.contrasena, archivo).await?)
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

// ───────────────────────── Traer las plantillas (H5) ─────────────────────────

/// Lo traído que espera a que se confirme: la primera vez, o con dudas.
#[derive(Default)]
pub struct Traidas(pub std::sync::Mutex<Option<Vec<Traida>>>);

/// El avance, por materias.
#[derive(Clone, Serialize)]
pub struct AvanceTraer {
    pub hechas: usize,
    pub total: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Traer {
    pub plan: Plan,
    /// `None`: espera a que se confirme.
    pub hecho: Option<Hecho>,
    /// Es la primera vez: se enseña antes de hacerlo.
    pub primera_vez: bool,
    /// Cómo se han leído las materias; degradado, el aviso temprano.
    pub estado_materias: Option<Estado>,
    /// Lo que ha cambiado en la forma de las páginas desde la última buena,
    /// sin nombres (`clickedu::huella`). También es aviso temprano.
    pub cambios: Vec<String>,
}

/// Baja las plantillas de Clickedu y planea qué hacer. Si es la primera vez
/// o hay dudas, espera a [`confirmar_traer`]; si no, lo hace.
#[tauri::command]
pub async fn traer_plantillas(app: AppHandle, avance: Channel<AvanceTraer>) -> Result<Traer, FalloTraer> {
    let hecho = traer_y_planear(&app, &avance).await;
    if let (Err(e), Ok(r)) = (&hecho, registro(&app)) {
        r.fallo("traer_plantillas", &e.to_string());
    }
    hecho
}

async fn traer_y_planear(app: &AppHandle, avance: &Channel<AvanceTraer>) -> Result<Traer, FalloTraer> {
    let carpeta_ce = carpeta_clickedu(app)?;
    let bajado = {
        let sesion = app.state::<Sesion>();
        let mut cogida = sesion.0.lock().await;
        // Recién abierta la app no hay sesión: se entra con la guardada.
        if cogida.is_none() {
            *cogida = Some(entrar_con_la_guardada(&carpeta_ce).await?);
        }
        let Some(cliente) = cogida.as_ref() else { return Err(FalloTraer::SinEntrar) };
        traer::bajar(cliente, |hechas, total| {
            let _ = avance.send(AvanceTraer { hechas, total });
        })
        .await
    };
    let bajado = match bajado {
        Ok(b) => b,
        Err(e) => {
            // La página que no se entiende, en local, para mirarla.
            let pagina = match &e {
                FalloTraer::Materias { pagina, .. } => Some(("materias".to_string(), pagina.clone())),
                FalloTraer::Conexion { fallo: Fallo::NoEntiendo { paso, html } } => Some((paso.clone(), html.clone())),
                _ => None,
            };
            if let Some((paso, html)) = pagina {
                let c = carpeta_ce.clone();
                let _ = bloqueante(move || guardar_pagina(&c, &paso, &html, ahora())).await;
            }
            return Err(e);
        }
    };

    let app = app.clone();
    bloqueante(move || {
        let Bajado { materias, pagina_materias, bajadas } = bajado;
        let estado_materias = materias.estado.clone();
        // Para la huella de las evaluaciones, la página de Ítems de la
        // primera materia: siempre la misma, para no comparar materias
        // distintas entre sí.
        let items = bajadas.first().map(|b| {
            let buena = match &b.resultado {
                Ok(p) => p.estado_evaluaciones == Estado::Bien,
                Err(FalloBajada::Evaluaciones { .. }) => false,
                Err(FalloBajada::NoEsExcel { .. }) => true,
            };
            (traer::direccion_items(&b.materia.id), b.pagina_items.clone(), buena)
        });

        // Los ítems (H6), guardados ya: son una copia de Clickedu y no
        // tocan nada del profesor, así que no esperan a confirmar. Los que
        // no se entienden se cuentan con los cambios, el aviso temprano.
        let traidos = ahora();
        let mut nuevos = BTreeMap::new();
        let mut items_rotos = Vec::new();
        for b in &bajadas {
            match b.resultado.as_ref().map(|p| &p.items) {
                Ok(Ok(evaluaciones)) => {
                    let m = ItemsMateria { traidos, evaluaciones: evaluaciones.clone() };
                    nuevos.insert(b.materia.id.clone(), m);
                }
                Ok(Err(roto)) => items_rotos.push(format!("{} {}: {roto}", b.materia.nombre, b.materia.grupo)),
                Err(_) => {}
            }
        }
        items::guardar(&carpeta_ce.join(items::FICHERO), nuevos)?;

        let traidas = traer::preparar(bajadas, &carpeta_ce.join("bajadas"))?;
        let plantillas = PathBuf::from(carpeta(&app)?.ruta);
        let raiz = traer::leer_raiz(&plantillas)?;

        // La huella, con los nombres de las plantillas tapados.
        let tapar = Tapar::new(nombres_de(&traidas, &raiz));
        let url_materias = format!("{BASE}{}", cliente::ASIGNATURAS);
        let buena = estado_materias == Estado::Bien;
        let mut cambios =
            comparar(&carpeta_ce, "materias", &huella::huella(&pagina_materias, &url_materias, &tapar), buena);
        if let Some((direccion, html, buena)) = items {
            let h = huella::huella(&html, &format!("{BASE}{direccion}"), &tapar);
            cambios.extend(comparar(&carpeta_ce, "evaluaciones", &h, buena));
        }
        cambios.extend(items_rotos);

        let recordadas = aplicar::leer_recordadas(&carpeta_ce.join("traidas.json"))?;
        let s = sesion::leer(&ruta_sesion(&app)?)?;
        let plan = planear::planear(&traidas, &raiz, &s, &recordadas, &HashMap::new());
        let primera_vez = !recordadas.confirmada;
        let hecho = if primera_vez || !plan.dudas.is_empty() {
            *app.state::<Traidas>().0.lock().map_err(|e| e.to_string())? = Some(traidas);
            None
        } else {
            Some(hacer(&app, &plan, &plantillas, &carpeta_ce)?)
        };
        Ok(Traer { plan, hecho, primera_vez, estado_materias: Some(estado_materias), cambios })
    })
    .await
    .map_err(FalloTraer::from)
}

/// Lo que espera, con las respuestas (fichero viejo → a qué nueva pasan sus
/// notas, o a ninguna): se vuelve a planear con la carpeta y la sesión de
/// ahora, y se hace.
#[tauri::command]
pub async fn confirmar_traer(respuestas: HashMap<String, Respuesta>, app: AppHandle) -> Result<Traer, String> {
    en_segundo_plano(app.clone(), "confirmar_traer", move || {
        let traidas = app
            .state::<Traidas>()
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .take()
            .ok_or("no hay nada traído que confirmar: vuelve a traer las plantillas")?;
        let carpeta_ce = carpeta_clickedu(&app)?;
        let plantillas = PathBuf::from(carpeta(&app)?.ruta);
        let raiz = traer::leer_raiz(&plantillas)?;
        let recordadas = aplicar::leer_recordadas(&carpeta_ce.join("traidas.json"))?;
        let s = sesion::leer(&ruta_sesion(&app)?)?;
        let plan = planear::planear(&traidas, &raiz, &s, &recordadas, &respuestas);
        let hecho = hacer(&app, &plan, &plantillas, &carpeta_ce)?;
        let primera_vez = !recordadas.confirmada;
        Ok(Traer { plan, hecho: Some(hecho), primera_vez, estado_materias: None, cambios: vec![] })
    })
    .await
}

/// Lo que espera, planeado otra vez con las respuestas de hasta ahora, sin
/// hacerlo: para enseñar a dónde pasan las notas antes de confirmar.
#[tauri::command]
pub async fn replanear_traer(respuestas: HashMap<String, Respuesta>, app: AppHandle) -> Result<Plan, String> {
    en_segundo_plano(app.clone(), "replanear_traer", move || {
        let cogidas = app.state::<Traidas>();
        let cogidas = cogidas.0.lock().map_err(|e| e.to_string())?;
        let traidas = cogidas.as_ref().ok_or("no hay nada traído que confirmar: vuelve a traer las plantillas")?;
        let carpeta_ce = carpeta_clickedu(&app)?;
        let raiz = traer::leer_raiz(&PathBuf::from(carpeta(&app)?.ruta))?;
        let recordadas = aplicar::leer_recordadas(&carpeta_ce.join("traidas.json"))?;
        let s = sesion::leer(&ruta_sesion(&app)?)?;
        Ok(planear::planear(traidas, &raiz, &s, &recordadas, &respuestas))
    })
    .await
}

/// Olvida lo que esperaba a que se confirmara.
#[tauri::command]
pub async fn cancelar_traer(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "cancelar_traer", move || {
        app.state::<Traidas>().0.lock().map_err(|e| e.to_string())?.take();
        let _ = std::fs::remove_dir_all(carpeta_clickedu(&app)?.join("bajadas"));
        Ok(())
    })
    .await
}

/// Aplica `plan` y avisa a la interfaz: las clases y sus notas han cambiado.
fn hacer(app: &AppHandle, plan: &Plan, plantillas: &Path, carpeta_ce: &Path) -> Result<Hecho, String> {
    let (a, m, d) = fecha(ahora());
    let (sesion, recordadas, bajadas) =
        (ruta_sesion(app)?, carpeta_ce.join("traidas.json"), carpeta_ce.join("bajadas"));
    let rutas = Rutas { plantillas, sesion: &sesion, recordadas: &recordadas, bajadas: &bajadas };
    let hoy = format!("{a:04}-{m:02}-{d:02}");
    let hecho = aplicar::aplicar(plan, &rutas, &hoy, &app.state::<sesion::Cerrojo>());
    let _ = app.emit(vigilante::EVENTO, ());
    let _ = app.emit(crate::commands::notas::EVENTO, ());
    Ok(hecho)
}

/// Los nombres de los alumnos de lo traído y de lo que hay en la carpeta:
/// lo que la huella ha de tapar.
fn nombres_de(traidas: &[Traida], raiz: &[EnLaRaiz]) -> Vec<String> {
    let de_traidas = traidas.iter().filter_map(|t| match &t.contenido {
        Contenido::Plantilla { plantilla, .. } => Some(plantilla),
        _ => None,
    });
    let de_raiz = raiz.iter().filter_map(|r| r.lectura.as_ref().ok());
    de_traidas.chain(de_raiz).flat_map(|p| p.alumnos.iter().map(|a| a.nombre.clone())).collect()
}

/// Compara la huella con la última buena; si no se puede, sin cambios.
fn comparar(carpeta: &Path, ficha: &str, ahora: &Huella, buena: bool) -> Vec<String> {
    huella::comparar_y_guardar(carpeta, ficha, ahora, buena).unwrap_or_default()
}
