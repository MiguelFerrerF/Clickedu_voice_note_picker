//! Comandos del **dictado** (H3). Lo que hacen está en [`crate::voz`].
//!
//! Hoy, entender una frase ya escrita (H3b): es lo que hará el
//! push-to-talk de H3a con lo que oiga Whisper, y lo que usa Calificar en
//! desarrollo para probar la confirmación sin micrófono. Cada frase
//! entendida se apunta en el registro (D8), con lo que hizo falta para
//! decidir.

use tauri::AppHandle;

use crate::commands::clases::leer_plantilla;
use crate::commands::{en_segundo_plano, registro};
use crate::registro::{Candidato, Dictado, Que};
use crate::voz::entender::entender_con_detalle;

/// Entiende `texto` con los alumnos de la clase `fichero`. Cada alumno va
/// por su **clave** de la sesión, no por su posición: es con lo que se pone
/// la nota.
#[tauri::command]
pub async fn entender_frase(fichero: String, texto: String, app: AppHandle) -> Result<Dictado, String> {
    en_segundo_plano(app.clone(), "entender_frase", move || {
        let plantilla = leer_plantilla(&app, &fichero)?;
        let nombres: Vec<&str> = plantilla.alumnos.iter().map(|a| a.nombre.as_str()).collect();
        let clave = |i: usize| plantilla.alumnos[i].clave.clone();
        let (entendido, detalle) = entender_con_detalle(&texto, &nombres);
        let registro = registro(&app)?;
        let dictado = Dictado { id: registro.nuevo_id(), oido: texto, entendido: entendido.con(clave) };
        registro.apuntar(Que::Dictado {
            dictado: dictado.clone(),
            fichero,
            nombre: detalle.nombre,
            candidatos: detalle
                .parecidos
                .iter()
                .map(|p| Candidato { clave: clave(p.alumno), parecido: p.parecido, distancia: p.distancia })
                .collect(),
        });
        Ok(dictado)
    })
    .await
}
