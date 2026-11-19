//! Comandos del **dictado** (H3). Lo que hacen está en [`crate::voz`].
//!
//! Hoy, entender una frase ya escrita (H3b): es lo que hará el
//! push-to-talk de H3a con lo que oiga Whisper, y lo que usa Calificar en
//! desarrollo para probar la confirmación sin micrófono.

use serde::Serialize;
use tauri::AppHandle;

use crate::commands::clases::leer_plantilla;
use crate::commands::en_segundo_plano;
use crate::voz::entender::{entender, Entendido, Motivo};

/// Lo entendido de una frase, con cada alumno por su **clave** (la de la
/// sesión, H2), no por su posición en la lista: es con lo que se pone la
/// nota. `oido`: lo que se entendió, tal cual, para enseñarlo.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Dictado {
    /// Se apunta sin preguntar.
    Seguro { oido: String, clave: String, nota: f64 },
    /// Se pregunta: los alumnos y las notas posibles, lo más probable primero.
    Duda { oido: String, claves: Vec<String>, notas: Vec<f64>, motivos: Vec<Motivo> },
    /// No se apunta nada.
    NoEntendido { oido: String, motivo: Motivo },
}

/// Entiende `texto` con los alumnos de la clase `fichero`.
#[tauri::command]
pub async fn entender_frase(fichero: String, texto: String, app: AppHandle) -> Result<Dictado, String> {
    en_segundo_plano(move || {
        let plantilla = leer_plantilla(&app, &fichero)?;
        let nombres: Vec<&str> = plantilla.alumnos.iter().map(|a| a.nombre.as_str()).collect();
        let clave = |i: usize| plantilla.alumnos[i].clave.clone();
        Ok(match entender(&texto, &nombres) {
            Entendido::Seguro { alumno, nota } => Dictado::Seguro { oido: texto, clave: clave(alumno), nota },
            Entendido::Duda { alumnos, notas, motivos } => {
                Dictado::Duda { oido: texto, claves: alumnos.into_iter().map(clave).collect(), notas, motivos }
            }
            Entendido::NoEntendido { motivo } => Dictado::NoEntendido { oido: texto, motivo },
        })
    })
    .await
}
