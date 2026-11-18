//! Lo que comparten los tests: escribir plantillas **inventadas** al vuelo.
//! Nunca alumnos reales en el repo.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use rust_xlsxwriter::Workbook;

/// Una celda de la plantilla de prueba.
pub enum C {
    T(&'static str),
    N(f64),
    /// Vacía.
    V,
}

/// Escribe `filas` en la primera hoja de `nombre`, desde la A1.
pub fn plantilla(dir: &Path, nombre: &str, filas: &[&[C]]) -> PathBuf {
    let mut libro = Workbook::new();
    let hoja = libro.add_worksheet();
    for (f, fila) in filas.iter().enumerate() {
        for (c, celda) in fila.iter().enumerate() {
            match celda {
                C::T(t) => {
                    hoja.write_string(f as u32, c as u16, *t).unwrap();
                }
                C::N(n) => {
                    hoja.write_number(f as u32, c as u16, *n).unwrap();
                }
                C::V => {}
            }
        }
    }
    let ruta = dir.join(nombre);
    libro.save(&ruta).unwrap();
    ruta
}

