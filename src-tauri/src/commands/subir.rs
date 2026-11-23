//! Comandos de **subir las notas** a Clickedu (H6). Lo que hacen está en
//! [`crate::clickedu::subir`].
//!
//! - [`preparar_subida`]: exporta el Excel (es también el plan B), entra si
//!   hace falta, **vuelve a leer** el ítem en Clickedu y cuenta qué notas
//!   son nuevas, iguales o **se pisarían**. Solo lee; lo preparado espera a
//!   [`confirmar_subida`] o a [`cancelar_subida`].
//! - [`confirmar_subida`]: sube, importa y comprueba, con el avance por
//!   pasos. **Escribe en Clickedu.**
//! - [`abrir_importar_en_clickedu`]: el plan B, la página de importar en el
//!   navegador.

use std::collections::BTreeMap;

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Emitter, Manager};

use crate::clickedu::cliente::{Cliente, BASE};
use crate::clickedu::guardar_pagina;
use crate::clickedu::items::{self, direccion_importar};
use crate::clickedu::subir::{self, Antes, Destino, FalloSubir, Paso};
use crate::clickedu::traer::{aplicar, FalloTraer};
use crate::commands::clases::leer_plantilla;
use crate::commands::clickedu::{bloqueante, carpeta_clickedu, entrar_con_la_guardada, Sesion};
use crate::commands::{carpeta_notas, registro, ruta_sesion};
use crate::exportar;
use crate::plantillas::Nota;
use crate::registro::ahora;
use crate::sesion::{self, Cerrojo, Juego};

/// Lo preparado, que espera a que se confirme.
pub struct Pendiente {
    destino: Destino,
    antes: Antes,
    /// El Excel: nombre y contenido.
    excel: (String, Vec<u8>),
    /// Las notas que se suben, por `Id` de alumno.
    esperadas: BTreeMap<String, f64>,
    /// El nombre de cada alumno, por `Id`: para decir cuáles no entraron.
    nombres: BTreeMap<String, String>,
}

#[derive(Default)]
pub struct Subidas(pub std::sync::Mutex<Option<Pendiente>>);

/// Una nota que se pisaría.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cambio {
    pub nombre: String,
    /// La de Clickedu, tal cual.
    pub antes: String,
    pub nueva: f64,
}

/// Lo que se va a subir, para preguntar.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Previa {
    pub item: String,
    pub evaluacion: String,
    pub alumnos: usize,
    pub con_nota: usize,
    /// Sin nota en Clickedu.
    pub nuevas: usize,
    /// La misma nota en Clickedu: subirlas no cambia nada.
    pub iguales: usize,
    /// **Se pisarían.**
    pub cambiarian: Vec<Cambio>,
    /// El Excel exportado (el plan B).
    pub excel: String,
}

/// Cómo ha ido.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subido {
    /// Cuántas están ya en Clickedu, comprobadas.
    pub entraron: usize,
    /// Los alumnos cuya nota no está en Clickedu tras importar.
    pub no_entraron: Vec<String>,
    /// Lo que Clickedu no ha dicho como siempre: el aviso temprano.
    pub avisos: Vec<String>,
}

/// Prepara la subida de las notas de la clase `fichero` (las del ítem
/// `item`, o las de sin ítem) al ítem `destino` de Clickedu. Solo lee.
#[tauri::command]
pub async fn preparar_subida(
    fichero: String,
    item: Option<String>,
    destino: String,
    app: AppHandle,
) -> Result<Previa, FalloSubir> {
    let hecho = preparar(&app, fichero, item, destino).await;
    apuntar(&app, "preparar_subida", &hecho).await;
    hecho
}

/// Lo de Clickedu del ítem: a dónde se sube y cómo se llaman el ítem y su
/// evaluación.
struct Donde {
    destino: Destino,
    item: String,
    evaluacion: String,
}

/// El ítem `destino` de la materia traída como `fichero`, de
/// `traidas.json` e `items.json`.
fn donde(app: &AppHandle, fichero: &str, destino: &str) -> Result<Donde, String> {
    let c = carpeta_clickedu(app)?;
    let recordadas = aplicar::leer_recordadas(&c.join("traidas.json"))?;
    let materia = recordadas
        .materias
        .iter()
        .find(|(_, f)| f.eq_ignore_ascii_case(fichero))
        .map(|(id, _)| id.clone())
        .ok_or_else(|| format!("«{fichero}» no es de una materia traída de Clickedu: trae las plantillas"))?;
    let guardados = items::leer(&c.join(items::FICHERO))?;
    let m =
        guardados.materias.get(&materia).ok_or("los ítems de esta materia no se han traído: trae las plantillas")?;
    m.evaluaciones
        .iter()
        .find_map(|e| {
            let i = e.items.iter().find(|i| i.id == destino)?;
            Some(Donde {
                destino: Destino { materia: materia.clone(), evaluacion: e.id.clone(), item: i.id.clone() },
                item: i.nombre.clone(),
                evaluacion: e.nombre.clone(),
            })
        })
        .ok_or_else(|| "ese ítem no está entre los traídos de esta materia: vuelve a traer las plantillas".to_string())
}

async fn preparar(
    app: &AppHandle,
    fichero: String,
    item: Option<String>,
    destino: String,
) -> Result<Previa, FalloSubir> {
    // Primero lo local: si algo falla aquí, no se llega a Clickedu.
    let a = app.clone();
    let (donde, excel, ruta_excel, esperadas, nombres, alumnos) = bloqueante(move || {
        let donde = donde(&a, &fichero, &destino)?;
        let plantilla = leer_plantilla(&a, &fichero)?;
        let puestas = {
            let cerrojo = a.state::<Cerrojo>();
            let _cogido = cerrojo.0.lock().map_err(|e| e.to_string())?;
            sesion::leer(&ruta_sesion(&a)?)?.puestas_de(Juego::de(&fichero, item.as_deref()))
        };
        let mut esperadas = BTreeMap::new();
        let mut sin_id = 0;
        for al in &plantilla.alumnos {
            if let Nota::Valor(v) = sesion::nota_de(al, &puestas) {
                let id_bueno = !al.id.is_empty() && al.id.bytes().all(|b| b.is_ascii_digit());
                if !id_bueno || esperadas.insert(al.id.clone(), v).is_some() {
                    sin_id += 1;
                }
            }
        }
        if sin_id > 0 {
            return Err(format!(
                "{sin_id} alumnos con nota no tienen un Id de Clickedu (o está repetido): Clickedu no sabría de quién es"
            ));
        }
        if esperadas.is_empty() {
            return Err("no hay ninguna nota que subir".to_string());
        }
        // El Excel, el mismo que «Exportar»: si la subida falla, es el plan B.
        let e = exportar::exportar(&plantilla, &puestas, &fichero, &carpeta_notas(&a)?).map_err(|e| e.to_string())?;
        let contenido = std::fs::read(&e.ruta).map_err(|err| format!("no se puede leer {}: {err}", e.ruta))?;
        let nombre = exportar::nombre_exportado(&fichero);
        let nombres: BTreeMap<String, String> =
            plantilla.alumnos.iter().map(|al| (al.id.clone(), al.nombre.clone())).collect();
        Ok((donde, (nombre, contenido), e.ruta, esperadas, nombres, plantilla.alumnos.len()))
    })
    .await?;

    let carpeta_ce = carpeta_clickedu(app)?;
    let antes = {
        let sesion = app.state::<Sesion>();
        let mut cogida = sesion.0.lock().await;
        let cliente = con_sesion(&mut cogida, &carpeta_ce).await?;
        subir::leer_antes(cliente, &donde.destino).await?
    };

    // Lo recién leído, a `items.json`: Calificar enseña lo de ahora.
    let (ruta_items, d, notas) = (carpeta_ce.join(items::FICHERO), donde.destino.clone(), antes.notas.clone());
    bloqueante(move || items::poner_notas(&ruta_items, &d.materia, &d.item, notas)).await?;
    let _ = app.emit(crate::commands::notas::EVENTO, ());

    let c = subir::comparar(&esperadas, &antes.notas);
    let cambiarian = c
        .cambiarian
        .iter()
        .map(|(id, antes)| Cambio {
            nombre: nombres.get(id).cloned().unwrap_or_default(),
            antes: antes.clone(),
            nueva: esperadas[id],
        })
        .collect();
    let previa = Previa {
        item: donde.item,
        evaluacion: donde.evaluacion,
        alumnos,
        con_nota: esperadas.len(),
        nuevas: c.nuevas.len(),
        iguales: c.iguales.len(),
        cambiarian,
        excel: ruta_excel,
    };
    let pendiente = Pendiente { destino: donde.destino, antes, excel, esperadas, nombres };
    *app.state::<Subidas>().0.lock().map_err(|e| e.to_string())? = Some(pendiente);
    Ok(previa)
}

/// El cliente de la sesión; si no hay, entra con la cuenta guardada.
async fn con_sesion<'a>(
    cogida: &'a mut Option<Cliente>,
    carpeta_ce: &std::path::Path,
) -> Result<&'a Cliente, FalloSubir> {
    if cogida.is_none() {
        *cogida = Some(entrar_con_la_guardada(carpeta_ce).await.map_err(|e| match e {
            FalloTraer::SinEntrar => FalloSubir::SinEntrar,
            FalloTraer::Conexion { fallo } => FalloSubir::Conexion { paso: Paso::Leer, fallo },
            otro => FalloSubir::Otro { detalle: otro.to_string() },
        })?);
    }
    cogida.as_ref().ok_or(FalloSubir::SinEntrar)
}

/// Sube lo preparado, lo importa y comprueba que entró. **Escribe en
/// Clickedu.**
#[tauri::command]
pub async fn confirmar_subida(app: AppHandle, avance: Channel<Paso>) -> Result<Subido, FalloSubir> {
    let hecho = confirmar(&app, &avance).await;
    apuntar(&app, "confirmar_subida", &hecho).await;
    hecho
}

async fn confirmar(app: &AppHandle, avance: &Channel<Paso>) -> Result<Subido, FalloSubir> {
    let p = app
        .state::<Subidas>()
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or_else(|| "no hay nada preparado para subir: vuelve a empezar".to_string())?;
    let carpeta_ce = carpeta_clickedu(app)?;
    let subida = {
        let sesion = app.state::<Sesion>();
        let mut cogida = sesion.0.lock().await;
        let cliente = con_sesion(&mut cogida, &carpeta_ce).await?;
        let (nombre, contenido) = p.excel;
        subir::subir(cliente, &p.destino, &p.antes, (&nombre, contenido), |paso| {
            let _ = avance.send(paso);
        })
        .await?
    };

    let (ruta_items, d, notas) = (carpeta_ce.join(items::FICHERO), p.destino.clone(), subida.despues.clone());
    bloqueante(move || items::poner_notas(&ruta_items, &d.materia, &d.item, notas)).await?;
    let _ = app.emit(crate::commands::notas::EVENTO, ());

    let c = subir::comparar(&p.esperadas, &subida.despues);
    let no_entraron = c
        .nuevas
        .iter()
        .chain(c.cambiarian.iter().map(|(id, _)| id))
        .map(|id| p.nombres.get(id).cloned().unwrap_or_else(|| id.clone()))
        .collect();
    Ok(Subido { entraron: c.iguales.len(), no_entraron, avisos: subida.avisos })
}

/// Olvida lo preparado.
#[tauri::command]
pub async fn cancelar_subida(app: AppHandle) -> Result<(), String> {
    app.state::<Subidas>().0.lock().map_err(|e| e.to_string())?.take();
    Ok(())
}

/// El plan B: la página de importar del ítem `destino` de la clase
/// `fichero`, en el navegador. Hay que entrar allí: la sesión de la app no
/// se comparte.
#[tauri::command]
pub async fn abrir_importar_en_clickedu(fichero: String, destino: String, app: AppHandle) -> Result<(), String> {
    crate::commands::en_segundo_plano(app.clone(), "abrir_importar_en_clickedu", move || {
        let d = donde(&app, &fichero, &destino)?.destino;
        let url = format!("{BASE}{}", direccion_importar(&d.materia, &d.evaluacion));
        std::process::Command::new("explorer")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("no se puede abrir el navegador: {e}"))
    })
    .await
}

/// Apunta un fallo en el registro (D8, sin páginas ni nombres) y guarda la
/// página que no se entiende en local, para mirarla.
async fn apuntar<T>(app: &AppHandle, donde: &str, hecho: &Result<T, FalloSubir>) {
    let Err(e) = hecho else { return };
    if let Ok(r) = registro(app) {
        r.fallo(donde, &e.to_string());
    }
    if let (FalloSubir::Cambiado { paso, pagina, .. }, Ok(c)) = (e, carpeta_clickedu(app)) {
        let (paso, pagina) = (format!("subir-{paso:?}").to_lowercase(), pagina.clone());
        let _ = bloqueante(move || guardar_pagina(&c, &paso, &pagina, ahora())).await;
    }
}
