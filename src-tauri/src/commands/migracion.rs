//! El paso desde la v1.3.4 (H7): lo que hace [`crate::migracion`], en dos
//! tiempos.
//!
//! 1. **Al abrir la app**, [`traer_de_la_v1`]: si la v1 está instalada, se
//!    copian sus plantillas y sus notas a `Documentos\Gestor de Notas\`, y la
//!    carpeta de plantillas de los ajustes pasa a la copia si apuntaba dentro
//!    de la v1. No pide nada a nadie, y la interfaz lo cuenta en un diálogo.
//! 2. **Con el botón del diálogo**, [`quitar_la_v1`]: su desinstalador, en
//!    silencio. Si se instaló para todo el equipo, Windows pide permiso
//!    (UAC); por eso no se hace sola al arrancar.
//!
//! Mientras no se quite, cada arranque vuelve a copiar (lo que ya está
//! igual cuenta como «ya estaba») y vuelve a ofrecer quitarla.
//!
//! **En desarrollo no se toca la v1 de verdad** (en el ordenador de
//! desarrollo está instalada): solo se mira la carpeta de `GESTORNOTAS_V1`,
//! si se da, con su `unins000.exe`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::ajustes;
use crate::commands::{carpeta_notas, en_segundo_plano, leer_ajustes};
use crate::migracion::{self, apunta_a_la_v1, Traido, V1};

/// Lo que ha pasado al traer, para el diálogo.
#[derive(Debug, Serialize)]
pub struct PasoV1 {
    pub traido: Traido,
    /// Dónde han quedado las plantillas y las notas.
    pub plantillas: String,
    pub notas: String,
    /// La carpeta de los ajustes apuntaba dentro de la v1 y ahora apunta a
    /// la copia.
    pub carpeta_cambiada: bool,
    /// Quitarla pedirá permiso a Windows.
    pub pide_permiso: bool,
}

/// Trae lo de la v1. `None` si no está instalada.
#[tauri::command]
pub async fn traer_de_la_v1(app: AppHandle) -> Result<Option<PasoV1>, String> {
    en_segundo_plano(app.clone(), "traer_de_la_v1", move || {
        let Some(v1) = buscar() else { return Ok(None) };
        let documentos = app.path().document_dir().map_err(|e| e.to_string())?;
        let plantillas = ajustes::por_defecto(&documentos);
        let notas = carpeta_notas(&app)?;
        let traido = migracion::copiar(&v1.carpeta, &plantillas, &notas)?;

        let mut carpeta_cambiada = false;
        let a = leer_ajustes(&app)?;
        if a.carpeta_plantillas.as_deref().is_some_and(|c| apunta_a_la_v1(c, &v1.carpeta)) {
            crate::commands::clases::cambiar_carpeta(&app, None)?;
            carpeta_cambiada = true;
        }
        Ok(Some(PasoV1 {
            traido,
            plantillas: plantillas.display().to_string(),
            notas: notas.display().to_string(),
            carpeta_cambiada,
            pide_permiso: v1.admin,
        }))
    })
    .await
}

/// Quita la v1 con su desinstalador y espera a que acabe. Antes vuelve a
/// copiar: así nunca se quita con algo sin traer.
#[tauri::command]
pub async fn quitar_la_v1(app: AppHandle) -> Result<(), String> {
    en_segundo_plano(app.clone(), "quitar_la_v1", move || {
        let v1 = buscar().ok_or("la versión anterior ya no está instalada")?;
        let documentos = app.path().document_dir().map_err(|e| e.to_string())?;
        migracion::copiar(&v1.carpeta, &ajustes::por_defecto(&documentos), &carpeta_notas(&app)?)?;
        desinstalar(&v1)
    })
    .await
}

fn buscar() -> Option<V1> {
    if cfg!(debug_assertions) {
        let carpeta = PathBuf::from(std::env::var_os("GESTORNOTAS_V1")?);
        let desinstalador = carpeta.join("unins000.exe");
        return Some(V1 { carpeta, desinstalador, admin: false });
    }
    migracion::buscar_v1()
}

/// El desinstalador de Inno Setup se copia al temporal y sigue desde allí,
/// así que lo que se espera es a que desaparezca `unins000.exe`.
fn desinstalar(v1: &V1) -> Result<(), String> {
    if !v1.desinstalador.is_file() {
        return Err(format!("no encuentro su desinstalador ({})", v1.desinstalador.display()));
    }
    if v1_abierta() {
        return Err("la versión anterior está abierta: ciérrala y vuelve a probar".into());
    }
    const ARGS: [&str; 3] = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"];
    let estado = if v1.admin {
        // Con permiso de administrador: `Start-Process -Verb RunAs`, que es
        // lo que hace el Explorador. La ruta va por el entorno, no dentro
        // del texto del script.
        sin_ventana(Command::new("powershell").args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "try { $p = Start-Process -FilePath $env:GESTORNOTAS_DESINSTALAR \
             -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -Verb RunAs -Wait -PassThru; \
             exit $p.ExitCode } catch { exit 1223 }",
        ]))
        .env("GESTORNOTAS_DESINSTALAR", &v1.desinstalador)
        .status()
    } else {
        sin_ventana(Command::new(&v1.desinstalador).args(ARGS)).status()
    }
    .map_err(|e| format!("no se puede abrir su desinstalador: {e}"))?;
    match estado.code() {
        Some(0) => {}
        Some(1223) => return Err("Windows no ha dado permiso para quitarla".into()),
        _ => return Err(format!("su desinstalador ha acabado con un error ({estado})")),
    }
    if esperar_a_que_no_este(&v1.desinstalador, Duration::from_secs(60)) {
        Ok(())
    } else {
        Err("su desinstalador no ha terminado en un minuto".into())
    }
}

fn esperar_a_que_no_este(f: &Path, tope: Duration) -> bool {
    let empieza = Instant::now();
    while f.exists() {
        if empieza.elapsed() > tope {
            return false;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    true
}

/// Si `GradingApp.exe` (la v1) está en marcha.
fn v1_abierta() -> bool {
    sin_ventana(Command::new("tasklist").args(["/FI", "IMAGENAME eq GradingApp.exe", "/NH"]))
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains("gradingapp.exe"))
        .unwrap_or(false)
}

/// Sin la consola negra que abriría Windows un momento.
fn sin_ventana(c: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000) // CREATE_NO_WINDOW
    }
    #[cfg(not(windows))]
    c
}
