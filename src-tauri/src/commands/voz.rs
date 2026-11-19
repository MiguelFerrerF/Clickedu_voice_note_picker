//! Comandos del **dictado** (H3). Lo que hacen está en [`crate::voz`].
//!
//! - **El push-to-talk** (H3a): la interfaz abre el micrófono mientras
//!   Calificar está a la vista y la ventana delante
//!   ([`abrir_microfono`]), y lo cierra al salir ([`cerrar_microfono`]):
//!   abrirlo tarda 0,2–0,4 s y, abierto, guarda los últimos 0,3 s para no
//!   perder el principio del nombre. Al pulsar ESPACIO,
//!   [`empezar_dictado`]; al soltarlo, [`terminar_dictado`] pasa lo grabado
//!   por Whisper y lo entiende. Nada se escribe en disco: **la voz no sale
//!   de la memoria**.
//! - **El modelo** se descarga la primera vez ([`descargar_modelo`], con
//!   avance) y se carga una vez, al abrir el micrófono, para que la primera
//!   frase no espere.
//! - [`entender_frase`] entiende una frase ya escrita: lo que usa Calificar
//!   en desarrollo para probar sin hablar.
//! - **El micrófono** se elige en Ajustes (HUI: [`listar_microfonos`],
//!   [`elegir_microfono`]); si no se elige, el de Windows.
//!
//! Cada frase entendida se apunta en el registro (D8), con lo que hizo
//! falta para decidir.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Manager};

use crate::ajustes::{self, MicrofonoElegido};
use crate::commands::clases::leer_plantilla;
use crate::commands::{en_segundo_plano, leer_ajustes, registro, ruta_ajustes};
use crate::registro::{Candidato, Dictado, Que};
use crate::voz::entender::entender_con_detalle;
use crate::voz::interprete::una_vez;
use crate::voz::microfono::{self, hay_voz, Microfono};
use crate::voz::modelo::{self, Comprobar};
use crate::voz::whisper::Whisper;

/// La voz de la app: el micrófono, si está abierto, y Whisper, si ya se ha
/// cargado. Cada uno con su cerrojo: se puede grabar la frase siguiente
/// mientras Whisper entiende la de antes.
#[derive(Default)]
pub struct Voz {
    microfono: Mutex<Option<Microfono>>,
    whisper: Mutex<Option<Whisper>>,
    descargando: AtomicBool,
}

fn coger<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

fn voz(app: &AppHandle) -> Result<Arc<Voz>, String> {
    app.try_state::<Arc<Voz>>().map(|v| v.inner().clone()).ok_or_else(|| "la voz no está".to_string())
}

/// `%APPDATA%\es.gestornotas.app\modelos`.
fn carpeta_modelos(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("modelos"))
}

/// Si el modelo ya está descargado.
#[tauri::command]
pub async fn hay_modelo(app: AppHandle) -> Result<bool, String> {
    en_segundo_plano(app.clone(), "hay_modelo", move || Ok(modelo::esta(&carpeta_modelos(&app)?))).await
}

/// El avance de la descarga, en bytes.
#[derive(Clone, Serialize)]
pub struct AvanceDescarga {
    pub hecho: u64,
    pub total: u64,
}

/// Descarga el modelo, comprobándolo mientras llega. Se escribe como
/// `.parcial` y solo se deja en su sitio si ha llegado entero y es el que
/// es. Como la descarga de las actualizaciones de library-app.
#[tauri::command]
pub async fn descargar_modelo(app: AppHandle, avance: Channel<AvanceDescarga>) -> Result<(), String> {
    let voz = voz(&app)?;
    if voz.descargando.swap(true, Ordering::SeqCst) {
        return Err("el modelo ya se está descargando".into());
    }
    let hecho = descargar(&app, &avance).await;
    voz.descargando.store(false, Ordering::SeqCst);
    if let (Err(e), Ok(r)) = (&hecho, registro(&app)) {
        r.fallo("descargar_modelo", e);
    }
    hecho
}

async fn descargar(app: &AppHandle, avance: &Channel<AvanceDescarga>) -> Result<(), String> {
    let carpeta = carpeta_modelos(app)?;
    if modelo::esta(&carpeta) {
        return Ok(());
    }
    std::fs::create_dir_all(&carpeta).map_err(|e| format!("no se puede crear {}: {e}", carpeta.display()))?;
    let destino = modelo::ruta(&carpeta);
    let parcial = destino.with_extension("parcial");

    let cliente = reqwest::Client::builder()
        .user_agent(concat!("GestorDeNotas/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("no se puede preparar la conexión: {e}"))?;
    let mut r = cliente
        .get(modelo::URL)
        .send()
        .await
        .map_err(|e| format!("no se ha podido descargar el modelo (¿hay conexión?): {e}"))?;
    if !r.status().is_success() {
        return Err(format!("no se ha podido descargar el modelo: el servidor ha contestado {}", r.status().as_u16()));
    }

    let resultado = async {
        let escribir = |e: std::io::Error| format!("no se puede escribir el modelo en {}: {e}", carpeta.display());
        let mut fichero = std::fs::File::create(&parcial).map_err(escribir)?;
        let mut comprobar = Comprobar::modelo();
        let mut hecho = 0u64;
        let mut ultimo_pct = u64::MAX;
        while let Some(trozo) = r.chunk().await.map_err(|e| format!("la descarga se ha cortado: {e}"))? {
            fichero.write_all(&trozo).map_err(escribir)?;
            comprobar.anadir(&trozo);
            hecho += trozo.len() as u64;
            // Un mensaje por punto porcentual: son miles de trozos.
            let pct = hecho * 100 / modelo::BYTES;
            if pct != ultimo_pct {
                ultimo_pct = pct;
                let _ = avance.send(AvanceDescarga { hecho, total: modelo::BYTES });
            }
        }
        fichero.sync_all().map_err(escribir)?;
        comprobar.terminar().map_err(|e| format!("el modelo {e}"))
    }
    .await;

    if let Err(e) = resultado {
        let _ = std::fs::remove_file(&parcial);
        return Err(e);
    }
    std::fs::rename(&parcial, &destino).map_err(|e| format!("no se puede dejar el modelo en su sitio: {e}"))
}

/// Los micrófonos conectados, cuál es el de Windows y el elegido en Ajustes
/// (que puede no estar conectado).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Microfonos {
    disponibles: Vec<MicrofonoElegido>,
    de_windows: Option<String>,
    elegido: Option<MicrofonoElegido>,
}

#[tauri::command]
pub async fn listar_microfonos(app: AppHandle) -> Result<Microfonos, String> {
    en_segundo_plano(app.clone(), "listar_microfonos", move || {
        let (disponibles, de_windows) = microfono::disponibles()?;
        Ok(Microfonos {
            disponibles: disponibles.into_iter().map(|d| MicrofonoElegido { id: d.id, nombre: d.nombre }).collect(),
            de_windows,
            elegido: leer_ajustes(&app)?.microfono,
        })
    })
    .await
}

/// Guarda el micrófono elegido (`None`: el de Windows) y cierra el abierto:
/// el siguiente dictado lo abre ya con el nuevo.
#[tauri::command]
pub async fn elegir_microfono(microfono: Option<MicrofonoElegido>, app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "elegir_microfono", move || {
        let mut a = leer_ajustes(&app)?;
        a.microfono = microfono;
        ajustes::guardar(&ruta_ajustes(&app)?, &a)?;
        drop(coger(&voz(&app)?.microfono).take());
        Ok(())
    })
    .await
}

/// El micrófono elegido en Ajustes, por su id.
fn elegido(app: &AppHandle) -> Result<Option<String>, String> {
    Ok(leer_ajustes(app)?.microfono.map(|m| m.id))
}

/// El micrófono abierto: si el elegido en Ajustes no se ha podido abrir, su
/// nombre (`falta`), y se escucha por el de Windows.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrofonoAbierto {
    nombre: String,
    falta: Option<String>,
}

/// Abre el micrófono, si no lo está. Si Whisper no está cargado, lo carga
/// después, sin esperar: así la primera frase no tarda más que las otras.
#[tauri::command]
pub async fn abrir_microfono(app: AppHandle) -> Result<MicrofonoAbierto, String> {
    en_segundo_plano(app.clone(), "abrir_microfono", move || {
        let voz = voz(&app)?;
        let pedido = leer_ajustes(&app)?.microfono;
        let abierto = {
            let mut m = coger(&voz.microfono);
            if m.is_none() {
                *m = Some(Microfono::abrir(pedido.as_ref().map(|p| p.id.clone()))?);
            }
            let m = m.as_ref().ok_or("el micrófono no está abierto")?;
            MicrofonoAbierto { nombre: m.nombre.clone(), falta: pedido.filter(|_| m.falta).map(|p| p.nombre) }
        };
        let carpeta = carpeta_modelos(&app)?;
        if modelo::esta(&carpeta) {
            std::thread::spawn(move || {
                if let (Err(e), Ok(r)) = (cargar_whisper(&voz, &carpeta).map(drop), registro(&app)) {
                    r.fallo("cargar_whisper", &e);
                }
            });
        }
        Ok(abierto)
    })
    .await
}

/// Cierra el micrófono: Windows deja de enseñar que se está usando.
#[tauri::command]
pub async fn cerrar_microfono(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "cerrar_microfono", move || {
        let cerrado = coger(&voz(&app)?.microfono).take();
        drop(cerrado);
        Ok(())
    })
    .await
}

/// Se ha pulsado ESPACIO: empieza a grabar, con los últimos 0,3 s de antes,
/// y manda por `nivel` cómo se oye, de 0 a 1, 20 veces por segundo (el
/// medidor de la interfaz). Si el micrófono no estaba abierto, se abre ahora
/// (y se pierde el principio).
#[tauri::command]
pub async fn empezar_dictado(app: AppHandle, nivel: Channel<f32>) -> Result<(), String> {
    en_segundo_plano(app.clone(), "empezar_dictado", move || {
        let voz = voz(&app)?;
        let mut m = coger(&voz.microfono);
        if m.is_none() {
            *m = Some(Microfono::abrir(elegido(&app)?)?);
        }
        if let Some(m) = m.as_ref() {
            m.empezar(move |n| {
                let _ = nivel.send(n);
            });
        }
        Ok(())
    })
    .await
}

/// Se ha soltado ESPACIO sin querer dictar (pulsación corta, la ventana ha
/// dejado de estar delante): lo grabado se tira.
#[tauri::command]
pub async fn cancelar_dictado(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "cancelar_dictado", move || {
        if let Some(m) = coger(&voz(&app)?.microfono).as_ref() {
            drop(m.terminar());
        }
        Ok(())
    })
    .await
}

/// Se ha soltado ESPACIO: lo grabado, por Whisper, y lo oído se entiende con
/// los alumnos de la clase `fichero`. `None` si no se ha oído nada: silencio,
/// o Whisper no ha escrito nada.
#[tauri::command]
pub async fn terminar_dictado(fichero: String, app: AppHandle) -> Result<Option<Dictado>, String> {
    en_segundo_plano(app.clone(), "terminar_dictado", move || {
        let voz = voz(&app)?;
        let audio = match coger(&voz.microfono).as_ref() {
            Some(m) => m.terminar(),
            None => return Err("el micrófono no está abierto".into()),
        };
        if !hay_voz(&audio) {
            return Ok(None);
        }
        let oido = cargar_whisper(&voz, &carpeta_modelos(&app)?)?
            .as_mut()
            .ok_or("Whisper no está cargado")?
            .transcribir(&audio, None)?;
        if oido.sin_palabras() {
            return Ok(None);
        }
        entender_y_apuntar(&app, fichero, oido.texto, true).map(Some)
    })
    .await
}

/// Whisper, cargándolo si no lo está. Se queda con el cerrojo: quien llegue
/// mientras se carga, espera.
fn cargar_whisper<'a>(voz: &'a Voz, carpeta: &std::path::Path) -> Result<MutexGuard<'a, Option<Whisper>>, String> {
    let mut w = coger(&voz.whisper);
    if w.is_none() {
        if !modelo::esta(carpeta) {
            return Err("el modelo de voz no está descargado".into());
        }
        *w = Some(Whisper::cargar(&modelo::ruta(carpeta))?);
    }
    Ok(w)
}

/// Entiende `texto` con los alumnos de la clase `fichero`. Cada alumno va
/// por su **clave** de la sesión, no por su posición: es con lo que se pone
/// la nota.
#[tauri::command]
pub async fn entender_frase(fichero: String, texto: String, app: AppHandle) -> Result<Dictado, String> {
    en_segundo_plano(app.clone(), "entender_frase", move || entender_y_apuntar(&app, fichero, texto, false)).await
}

/// Entiende lo oído (`voz`) o escrito y lo apunta en el registro (D8).
fn entender_y_apuntar(app: &AppHandle, fichero: String, texto: String, voz: bool) -> Result<Dictado, String> {
    let plantilla = leer_plantilla(app, &fichero)?;
    let nombres: Vec<&str> = plantilla.alumnos.iter().map(|a| a.nombre.as_str()).collect();
    let clave = |i: usize| plantilla.alumnos[i].clave.clone();
    let (entendido, detalle) = entender_con_detalle(&texto, &nombres);
    let registro = registro(app)?;
    let dictado =
        Dictado { id: registro.nuevo_id(), repetida: una_vez(&texto), oido: texto, entendido: entendido.con(clave) };
    registro.apuntar(Que::Dictado {
        dictado: dictado.clone(),
        fichero,
        nombre: detalle.nombre,
        candidatos: detalle
            .parecidos
            .iter()
            .map(|p| Candidato { clave: clave(p.alumno), parecido: p.parecido, distancia: p.distancia })
            .collect(),
        voz,
    });
    Ok(dictado)
}
