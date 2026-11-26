//! Comandos de **refrescar lo de Clickedu solo** (H11). Lo que hace está en
//! [`crate::clickedu::refrescar`]; aquí, cuándo y qué se enseña.
//!
//! - [`refrescar_clickedu`]: lo llama la ventana cada minuto mientras se ve
//!   (`useRefrescoClickedu`) y al volver a ella. **Aquí se decide si toca**:
//!   encendido en Ajustes, sin parar por la cuenta y pasados los minutos
//!   desde el último intento. Sin cuenta guardada, ni se intenta.
//! - [`estado_refresco`] y [`poner_refresco`]: para Inicio, Conexión y
//!   Ajustes.
//!
//! Los fallos, sin ruido: sin red o con Clickedu caído, solo «el último
//! falló», y se vuelve a probar al siguiente. Si Clickedu no acepta la
//! cuenta, **se para** hasta volver a entrar en Conexión ([`entrado`]).
//!
//! Traer y subir a mano cuentan como refresco ([`contar`]) y, mientras
//! esperan la sesión ([`Esperando`]), el refresco se para entre materia y
//! materia.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::clickedu::{bloqueante, carpeta_clickedu, entrar_con_la_guardada, guardar_pagina_de, Sesion};
use super::{carpeta, leer_ajustes, registro, ruta_ajustes, ruta_sesion, AJUSTES_POR_TURNOS};
use crate::ajustes::{self, Refresco};
use crate::clickedu::cliente::Fallo;
use crate::clickedu::refrescar::{self, PorTraer, Refrescado, Rutas};
use crate::clickedu::traer::FalloTraer;
use crate::registro::ahora;

/// Lo leído de Clickedu ha cambiado (`items.json`): las pantallas vuelven a
/// leer, sin perder lo que se está haciendo.
pub const EVENTO: &str = "clickedu-refrescado";

/// Ha cambiado el estado del refresco (cuándo, si falla, lo que hay por
/// traer): Inicio, Conexión y Ajustes lo vuelven a pedir.
pub const EVENTO_ESTADO: &str = "refresco-estado";

/// Cuántos esperan la sesión para traer o subir a mano.
#[derive(Default)]
pub struct QuierenLaSesion(AtomicUsize);

/// Mientras vive, alguien espera la sesión: el refresco se para.
pub(crate) struct Esperando<'a>(&'a AtomicUsize);

impl<'a> Esperando<'a> {
    pub(crate) fn nuevo(app: &'a AppHandle) -> Self {
        let quieren = &app.state::<QuierenLaSesion>().inner().0;
        quieren.fetch_add(1, Ordering::SeqCst);
        Esperando(quieren)
    }
}

impl Drop for Esperando<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Lo que se sabe del refresco mientras la app está abierta. Al cerrarla se
/// olvida: al abrirla se refresca otra vez.
#[derive(Default)]
pub struct Refrescos(Mutex<Interno>);

#[derive(Default, Clone)]
struct Interno {
    /// El último intento, bueno o no: de aquí se cuentan los minutos.
    intento: Option<u64>,
    /// El último que fue bien (o traer o subir a mano).
    ultimo: Option<u64>,
    fallo: Option<String>,
    /// Clickedu no acepta la cuenta: no se insiste hasta volver a entrar.
    parado: bool,
    refrescando: bool,
    por_traer: Vec<PorTraer>,
    avisos: Vec<String>,
}

/// Lo que enseñan Inicio, Conexión y Ajustes.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoRefresco {
    pub encendido: bool,
    pub minutos: u32,
    /// Segundos Unix.
    pub ultimo: Option<u64>,
    pub fallo: Option<String>,
    pub parado: bool,
    pub refrescando: bool,
    /// Las plantillas que han cambiado en Clickedu y esperan a traerse a mano.
    pub por_traer: Vec<PorTraer>,
    pub avisos: Vec<String>,
}

fn con_interno<T>(app: &AppHandle, f: impl FnOnce(&mut Interno) -> T) -> T {
    let r = app.state::<Refrescos>();
    let mut i = r.0.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut i)
}

fn avisar(app: &AppHandle) {
    let _ = app.emit(EVENTO_ESTADO, ());
}

/// Traer o subir a mano han leído de Clickedu: cuenta como refresco (los
/// minutos vuelven a empezar). Con `por_traer`, lo que queda por traer
/// ahora (vacío, si se ha traído todo).
pub(crate) fn contar(app: &AppHandle, por_traer: Option<Vec<PorTraer>>) {
    let t = ahora();
    con_interno(app, |i| {
        i.intento = Some(t);
        i.ultimo = Some(t);
        i.fallo = None;
        i.parado = false;
        if let Some(p) = por_traer {
            i.por_traer = p;
        }
    });
    avisar(app);
}

/// Se ha entrado a mano en Conexión: si estaba parado, vuelve, y refresca
/// en cuanto la ventana lo pida.
pub(crate) fn entrado(app: &AppHandle) {
    con_interno(app, |i| {
        i.parado = false;
        i.fallo = None;
        i.intento = None;
    });
    avisar(app);
}

/// Se ha olvidado la cuenta: lo sabido ya no vale.
pub(crate) fn olvidado(app: &AppHandle) {
    con_interno(app, |i| *i = Interno { refrescando: i.refrescando, ..Interno::default() });
    avisar(app);
}

fn estado(app: &AppHandle, r: Refresco) -> EstadoRefresco {
    let i = con_interno(app, |i| i.clone());
    EstadoRefresco {
        encendido: r.encendido,
        minutos: r.minutos,
        ultimo: i.ultimo,
        fallo: i.fallo,
        parado: i.parado,
        refrescando: i.refrescando,
        por_traer: i.por_traer,
        avisos: i.avisos,
    }
}

async fn leer_refresco(app: &AppHandle) -> Result<Refresco, String> {
    let a = app.clone();
    bloqueante(move || Ok(leer_ajustes(&a)?.refresco.unwrap_or_default())).await
}

#[tauri::command]
pub async fn estado_refresco(app: AppHandle) -> Result<EstadoRefresco, String> {
    let r = leer_refresco(&app).await?;
    Ok(estado(&app, r))
}

/// Enciende o apaga el refresco y cada cuántos minutos (15, 30 o 60).
#[tauri::command]
pub async fn poner_refresco(encendido: bool, minutos: u32, app: AppHandle) -> Result<EstadoRefresco, String> {
    if !Refresco::MINUTOS.contains(&minutos) {
        return Err(format!("{minutos} minutos no es de los que se pueden elegir"));
    }
    let a = app.clone();
    let r = bloqueante(move || {
        let _turno = AJUSTES_POR_TURNOS.lock().unwrap_or_else(|e| e.into_inner());
        let mut ajustes = leer_ajustes(&a)?;
        let r = Refresco { encendido, minutos };
        ajustes.refresco = Some(r);
        ajustes::guardar(&ruta_ajustes(&a)?, &ajustes)?;
        Ok(r)
    })
    .await?;
    avisar(&app);
    Ok(estado(&app, r))
}

/// Refresca si toca. Vuelve enseguida si no toca; si toca, cuando acaba.
/// Lo que pase se cuenta con [`EVENTO_ESTADO`], no como error: un fallo de
/// red no es para la ventana.
#[tauri::command]
pub async fn refrescar_clickedu(app: AppHandle) -> Result<(), String> {
    let r = leer_refresco(&app).await?;
    let t = ahora();
    let toca = con_interno(&app, |i| {
        let pronto = i.intento.is_some_and(|antes| t < antes + u64::from(r.minutos) * 60);
        if !r.encendido || i.parado || i.refrescando || pronto {
            return false;
        }
        i.intento = Some(t);
        i.refrescando = true;
        true
    });
    if !toca {
        return Ok(());
    }
    avisar(&app);

    let hecho = refrescar(&app, t).await;
    let datos = matches!(hecho, Ok(Some(_)));
    let apuntar = con_interno(&app, |i| {
        i.refrescando = false;
        match hecho {
            Ok(Some(Refrescado { cuando, por_traer, avisos })) => {
                i.ultimo = Some(cuando);
                i.fallo = None;
                i.por_traer = por_traer;
                i.avisos = avisos;
                None
            }
            // Un traer o una subida tienen la sesión, o la esperan: ellos
            // cuentan como refresco. Si no, se prueba al minuto.
            Ok(None) | Err(FalloTraer::Parado) => {
                i.intento = None;
                None
            }
            // Sin cuenta guardada: no es un fallo.
            Err(FalloTraer::SinEntrar) => None,
            Err(e) => {
                if matches!(e, FalloTraer::Conexion { fallo: Fallo::Usuario | Fallo::Archivo { .. } }) {
                    i.parado = true;
                }
                i.fallo = Some(e.to_string());
                Some(e.to_string())
            }
        }
    });
    if let (Some(e), Ok(r)) = (apuntar, registro(&app)) {
        r.fallo("refrescar_clickedu", &e);
    }
    if datos {
        let _ = app.emit(EVENTO, ());
    }
    avisar(&app);
    Ok(())
}

async fn refrescar(app: &AppHandle, t: u64) -> Result<Option<Refrescado>, FalloTraer> {
    let carpeta_ce = carpeta_clickedu(app)?;
    let a = app.clone();
    let (plantillas, sesion) = bloqueante(move || Ok((PathBuf::from(carpeta(&a)?.ruta), ruta_sesion(&a)?))).await?;
    let rutas = Rutas { clickedu: carpeta_ce.clone(), plantillas, sesion };
    let quieren = &app.state::<QuierenLaSesion>().inner().0;
    let parar = || quieren.load(Ordering::SeqCst) > 0;
    let entrar = || entrar_con_la_guardada(&carpeta_ce);
    let hecho = refrescar::refrescar(&app.state::<Sesion>().inner().0, entrar, rutas, parar, t).await;
    if let Err(e) = &hecho {
        guardar_pagina_de(&carpeta_ce, e).await;
    }
    hecho
}
