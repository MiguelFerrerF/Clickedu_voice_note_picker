//! Escribe la clase **inventada** del corpus de voz (`tests/datos/voz/
//! clase.txt`) como una plantilla de Clickedu, para probar el dictado en la
//! app sin alumnos reales:
//!
//!     cargo run --example escribir_clase_inventada [-- <carpeta>]
//!
//! Por defecto, en `Documentos\Gestor de Notas\plantillas` (la de la v2, no
//! la de la v1). Con la forma de las plantillas reales (H1): hoja
//! `Worksheet`, cabecera `Id | Alumnos | Nota` en la fila 1, el ID como
//! número y el nombre como «Apellidos, Nombre». Sin notas.

#[path = "comun/corpus.rs"]
mod corpus;

use std::path::PathBuf;

use gestor_de_notas_lib::plantillas;
use rust_xlsxwriter::Workbook;

const FICHERO: &str = "prueba_voz_inventada.xlsx";

fn main() {
    let carpeta = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        dirs::document_dir().expect("no encuentro Documentos").join("Gestor de Notas").join("plantillas")
    });
    std::fs::create_dir_all(&carpeta).expect("no se puede crear la carpeta");

    let mut libro = Workbook::new();
    let hoja = libro.add_worksheet();
    hoja.set_name("Worksheet").unwrap();
    for (c, cabecera) in ["Id", "Alumnos", "Nota"].iter().enumerate() {
        hoja.write_string(0, c as u16, *cabecera).unwrap();
    }
    let clase = corpus::clase();
    for (i, nombre) in clase.iter().enumerate() {
        // «Nombre Apellidos» → «Apellidos, Nombre». Los nombres compuestos
        // del corpus («María José», «Juan Pablo») van enteros delante.
        let palabras: Vec<&str> = nombre.split_whitespace().collect();
        let de_nombre = match palabras.as_slice() {
            ["María", "José", ..] | ["Juan", "Pablo", ..] => 2,
            _ => 1,
        };
        let (n, a) = palabras.split_at(de_nombre);
        let fila = i as u32 + 1;
        hoja.write_number(fila, 0, 900_001.0 + i as f64).unwrap();
        hoja.write_string(fila, 1, format!("{}, {}", a.join(" "), n.join(" "))).unwrap();
    }
    let ruta = carpeta.join(FICHERO);
    libro.save(&ruta).expect("no se puede guardar");

    // Que se lea como una de verdad, sin avisos.
    let p = plantillas::leer(&ruta).expect("no se lee");
    assert_eq!(p.alumnos.len(), clase.len());
    assert!(p.avisos.is_empty(), "avisos: {:?}", p.avisos);
    let sin_ext = FICHERO.trim_end_matches(".xlsx");
    println!("{} alumnos en {}", p.alumnos.len(), ruta.display());
    println!("En la app: «{}»", plantillas::nombre_legible(sin_ext));
}
