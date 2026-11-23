//! Clickedu (H4–H6). **No tiene API para esto** (la oficial no expone notas
//! y no nos dan credenciales): se lee su web, como hacía la v1
//! (`git show main:GradingApp/core/clickedu_client.py`).
//!
//! - [`cuenta`] — la cuenta guardada: usuario y contraseña en el
//!   Administrador de credenciales de Windows (D3), y la copia del archivo
//!   de seguridad en `%APPDATA%\es.gestornotas.app\clickedu\`.
//! - [`cliente`] — entrar (usuario y contraseña, y luego el archivo de
//!   seguridad), comprobar que se ha entrado y abrir páginas de dentro,
//!   **volviendo a entrar solo** si la sesión ha caducado.
//! - [`fichas`] — cada dato (materias, evaluaciones, ítems, el formulario
//!   de importar, las notas puestas): cómo se busca y qué tiene que
//!   cumplir. Bien, degradado o roto.
//! - [`huella`] — la forma de cada página, sin nombres, comparada con la
//!   última buena: qué ha cambiado y dónde.
//! - [`tapar`] — pasar textos y direcciones a su forma, sin nombres.
//! - [`traer`] — traer las plantillas (H5): el nombre de cada fichero.
//! - [`items`] — los ítems de cada materia, leídos al traer y guardados
//!   para calificar por ítem (H6).
//!
//! El cliente se prueba contra un Clickedu falso con páginas inventadas
//! ([`falso`]); las fichas y la huella, con páginas **de verdad,
//! anonimizadas** (`tests/datos/clickedu/`, `examples/anonimizar_clickedu`).
//! Clickedu no distingue un archivo de seguridad caducado de uno incorrecto
//! (probado el 2026-11-20).
//!
//! Una página que no se entiende se guarda **en local** ([`guardar_pagina`])
//! para diagnosticar: puede llevar nombres, así que no va en el informe de
//! D8 ni sale del ordenador.

pub mod cliente;
pub mod cuenta;
pub mod fichas;
pub mod huella;
pub mod items;
pub mod tapar;
pub mod traer;

#[cfg(test)]
pub(crate) mod falso;

use std::path::{Path, PathBuf};

/// Cuántas páginas no entendidas se guardan: las últimas.
pub const PAGINAS: usize = 10;

/// Guarda `html` en `carpeta\paginas\<segundos>-<paso>.html` y deja solo las
/// [`PAGINAS`] últimas. Devuelve dónde ha quedado.
pub fn guardar_pagina(carpeta: &Path, paso: &str, html: &str, ahora: u64) -> Result<PathBuf, String> {
    let paginas = carpeta.join("paginas");
    std::fs::create_dir_all(&paginas).map_err(|e| format!("no se puede crear {}: {e}", paginas.display()))?;
    let ruta = paginas.join(format!("{ahora}-{paso}.html"));
    std::fs::write(&ruta, html).map_err(|e| format!("no se puede guardar la página en {}: {e}", ruta.display()))?;

    // Los nombres empiezan por los segundos, todos con las mismas cifras
    // hasta el año 2286: el orden de los nombres es el de las fechas.
    let mut todas: Vec<PathBuf> = std::fs::read_dir(&paginas)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "html"))
        .collect();
    todas.sort();
    let sobran = todas.len().saturating_sub(PAGINAS);
    for vieja in &todas[..sobran] {
        let _ = std::fs::remove_file(vieja);
    }
    Ok(ruta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn se_quedan_las_ultimas() {
        let dir = tempfile::tempdir().unwrap();
        for s in 0..(PAGINAS as u64 + 3) {
            guardar_pagina(dir.path(), "entrar", "<html></html>", 1_800_000_000 + s).unwrap();
        }
        let mut quedan: Vec<String> = std::fs::read_dir(dir.path().join("paginas"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        quedan.sort();
        assert_eq!(quedan.len(), PAGINAS);
        assert_eq!(quedan[0], "1800000003-entrar.html");
    }
}
