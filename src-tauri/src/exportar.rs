//! Exportar las notas de una clase (H2) al Excel que se sube a Clickedu.
//!
//! **Replica el exportado de la v1**, que Clickedu aceptaba
//! (`git show main:GradingApp/core/excel_manager.py` y `gui/app_window.py`):
//!
//! - Un `.xlsx` **nuevo**, no una copia de la plantilla: la plantilla no se
//!   escribe nunca (D4). Una hoja `Sheet1` (la que ponía `pandas`), con las
//!   **mismas cabeceras** que la plantilla en la fila 1 y un alumno por fila.
//! - El ID como número si es un número (`pandas` lo leía así y así lo
//!   escribía), y como texto si no, o si empieza por cero: `007` no es `7`.
//! - La nota **como texto con punto**, sin ceros de más: `"7.5"`, `"7"`,
//!   `"6.25"` (el `{:g}` de la v1). Como texto para que Excel no la pinte
//!   con la coma del sistema. Sin nota, la celda vacía.
//! - En `Documentos\Gestor de Notas\notas\notas_<plantilla>.xlsx`. Si ya
//!   había uno de esa clase, el viejo pasa a `notas\notas_viejas\`.
//!
//! **Lo que cambia respecto a la v1**, que lo hacía en silencio:
//!
//! - Si alguna nota no es una nota de 0 a 10, **no se exporta** y se dice en
//!   qué filas: la v1 la dejaba vacía sin avisar.
//! - Al exportar una clase solo se aparta el exportado viejo **de esa
//!   clase**; la v1 apartaba todos los de `notas\`. Y en `notas_viejas\` se
//!   numeran (`notas_x (2).xlsx`) en vez de llevar la hora.
//! - Se escribe primero a un temporal: si falla a mitad, el exportado de
//!   antes sigue donde estaba.

use std::fmt;
use std::path::{Path, PathBuf};

use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};
use serde::Serialize;

use crate::plantillas::{Nota, Plantilla};
use crate::sesion::{nota_de, Puestas};

/// Lo que ha salido bien, para decirlo.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exportado {
    pub ruta: String,
    pub con_nota: usize,
    pub alumnos: usize,
    /// Dónde ha ido a parar el exportado de antes, si lo había.
    pub viejo: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorExportar {
    /// Hay valores que no son nota, en estas filas de la plantilla.
    NotasInvalidas(Vec<u32>),
    /// El exportado de antes no se puede apartar: casi siempre, porque está
    /// abierto en Excel.
    ViejoBloqueado(String),
    Escribir(String),
}

impl fmt::Display for ErrorExportar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotasInvalidas(filas) => {
                let filas: Vec<String> = filas.iter().map(u32::to_string).collect();
                write!(f, "hay valores que no son una nota de 0 a 10 (filas {})", filas.join(", "))
            }
            Self::ViejoBloqueado(e) => {
                write!(f, "el exportado anterior no se puede apartar; si está abierto en Excel, ciérralo ({e})")
            }
            Self::Escribir(e) => write!(f, "no se puede escribir el Excel: {e}"),
        }
    }
}

impl std::error::Error for ErrorExportar {}

/// `notas_<plantilla sin extensión>.xlsx`, como la v1 (que dejaba también la
/// extensión de la plantilla: aquí siempre es `.xlsx`, que es lo que es).
pub fn nombre_exportado(fichero: &str) -> String {
    let base = Path::new(fichero).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    format!("notas_{base}.xlsx")
}

/// La nota como la escribe la v1: con punto y sin ceros de más.
pub fn nota_como_texto(v: f64) -> String {
    // `Display` de `f64` ya da lo más corto que vuelve al mismo número:
    // `7.0` → `7`, `7.50` → `7.5`.
    format!("{v}")
}

/// Exporta la clase `fichero` (la plantilla leída y las notas puestas) a
/// `carpeta`.
pub fn exportar(plantilla: &Plantilla, puestas: &Puestas, fichero: &str, carpeta: &Path) -> Result<Exportado, ErrorExportar> {
    let notas: Vec<Nota> = plantilla.alumnos.iter().map(|a| nota_de(a, puestas)).collect();
    let invalidas: Vec<u32> = plantilla
        .alumnos
        .iter()
        .zip(&notas)
        .filter(|(_, n)| matches!(n, Nota::Invalida(_)))
        .map(|(a, _)| a.fila)
        .collect();
    if !invalidas.is_empty() {
        return Err(ErrorExportar::NotasInvalidas(invalidas));
    }

    let escribir = |e: &dyn fmt::Display| ErrorExportar::Escribir(e.to_string());
    std::fs::create_dir_all(carpeta).map_err(|e| escribir(&e))?;
    let destino = carpeta.join(nombre_exportado(fichero));
    let temporal = destino.with_extension("xlsx.tmp");
    escribir_libro(plantilla, &notas, &temporal).map_err(|e| {
        let _ = std::fs::remove_file(&temporal);
        escribir(&e)
    })?;

    let viejo = if destino.exists() {
        let viejas = carpeta.join("notas_viejas");
        let apartado = std::fs::create_dir_all(&viejas)
            .map(|_| hueco_libre(&viejas, &nombre_exportado(fichero)))
            .and_then(|a| std::fs::rename(&destino, &a).map(|_| a));
        match apartado {
            Ok(a) => Some(a.display().to_string()),
            Err(e) => {
                let _ = std::fs::remove_file(&temporal);
                return Err(ErrorExportar::ViejoBloqueado(e.to_string()));
            }
        }
    } else {
        None
    };
    std::fs::rename(&temporal, &destino).map_err(|e| escribir(&e))?;

    Ok(Exportado {
        ruta: destino.display().to_string(),
        con_nota: notas.iter().filter(|n| matches!(n, Nota::Valor(_))).count(),
        alumnos: notas.len(),
        viejo,
    })
}

fn escribir_libro(plantilla: &Plantilla, notas: &[Nota], ruta: &Path) -> Result<(), rust_xlsxwriter::XlsxError> {
    let mut libro = Workbook::new();
    let hoja = libro.add_worksheet();
    hoja.set_name("Sheet1")?;

    // La cabecera como la ponía `pandas`: negrita, centrada y con borde.
    let cabecera = Format::new()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::Top);
    let c = &plantilla.columnas;
    for (col, texto) in [&c.id.cabecera, &c.nombre.cabecera, &c.nota.cabecera].into_iter().enumerate() {
        hoja.write_string_with_format(0, col as u16, texto, &cabecera)?;
    }

    for (i, (a, nota)) in plantilla.alumnos.iter().zip(notas).enumerate() {
        let fila = i as u32 + 1;
        match id_como_numero(&a.id) {
            Some(n) => hoja.write_number(fila, 0, n)?,
            None => hoja.write_string(fila, 0, &a.id)?,
        };
        hoja.write_string(fila, 1, &a.nombre)?;
        if let Nota::Valor(v) = nota {
            hoja.write_string(fila, 2, nota_como_texto(*v))?;
        }
    }
    hoja.set_column_width(1, 40)?;
    libro.save(ruta)
}

/// El ID como número si son solo cifras, sin ceros delante y cabe sin perder
/// nada en un `f64`.
fn id_como_numero(id: &str) -> Option<f64> {
    let cifras = !id.is_empty() && id.len() <= 15 && id.bytes().all(|b| b.is_ascii_digit());
    let sin_cero_delante = id == "0" || !id.starts_with('0');
    (cifras && sin_cero_delante).then(|| id.parse().ok()).flatten()
}

/// `nombre` en `carpeta`, o `nombre (2)`, `(3)`… si ya está.
fn hueco_libre(carpeta: &Path, nombre: &str) -> PathBuf {
    let ruta = carpeta.join(nombre);
    if !ruta.exists() {
        return ruta;
    }
    let p = Path::new(nombre);
    let base = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = p.extension().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    (2..)
        .map(|n| carpeta.join(format!("{base} ({n}).{ext}")))
        .find(|r| !r.exists())
        .expect("siempre hay un número libre")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notas_como_la_v1() {
        assert_eq!(nota_como_texto(7.5), "7.5");
        assert_eq!(nota_como_texto(7.0), "7");
        assert_eq!(nota_como_texto(10.0), "10");
        assert_eq!(nota_como_texto(0.0), "0");
        assert_eq!(nota_como_texto(6.25), "6.25");
        assert_eq!(nota_como_texto(6.8), "6.8");
    }

    #[test]
    fn ids() {
        assert_eq!(id_como_numero("20231"), Some(20231.0));
        assert_eq!(id_como_numero("0"), Some(0.0));
        assert_eq!(id_como_numero("007"), None);
        assert_eq!(id_como_numero("A-12"), None);
        assert_eq!(id_como_numero(""), None);
    }

    #[test]
    fn nombres() {
        assert_eq!(nombre_exportado("biologia_1_ab_bach.xlsx"), "notas_biologia_1_ab_bach.xlsx");
        assert_eq!(nombre_exportado("VIEJA.xls"), "notas_VIEJA.xlsx");
    }
}
