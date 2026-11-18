//! Pasa el lector de plantillas por una carpeta de plantillas **reales** y
//! enseña cómo las entiende, **sin ningún dato de alumnos**: la cabecera, las
//! columnas elegidas, cuántos alumnos y los avisos (con sus filas).
//!
//!     cargo run --example revisar_plantillas -- "C:\ruta\a\plantillas"
//!
//! Para comprobar el formato de Clickedu sin copiar las plantillas al repo.
//! Lo que hay encima de la cabecera se enseña como forma (`texto(12)`), no
//! como texto: puede llevar el nombre de un profesor o de un grupo.

use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use gestor_de_notas_lib::plantillas::{self, Aviso};

fn main() {
    let carpeta = std::env::args().nth(1).expect("falta la carpeta");
    let mut ficheros: Vec<_> = std::fs::read_dir(&carpeta)
        .expect("no se puede leer la carpeta")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| plantillas::es_plantilla(&p.file_name().unwrap().to_string_lossy()))
        .collect();
    ficheros.sort();

    for ruta in ficheros {
        let nombre = ruta.file_name().unwrap().to_string_lossy().into_owned();
        let sin_ext = ruta.file_stem().unwrap().to_string_lossy().into_owned();
        println!("\n── {nombre}  →  «{}»", plantillas::nombre_legible(&sin_ext));
        hojas(&ruta);
        match plantillas::leer(&ruta) {
            Ok(p) => {
                let c = &p.columnas;
                println!("  hoja «{}», cabecera en la fila {}", c.hoja, c.fila_cabecera);
                println!(
                    "  ID = col {} «{}» · nombre = col {} «{}» · nota = col {} «{}»",
                    c.id.indice, c.id.cabecera, c.nombre.indice, c.nombre.cabecera, c.nota.indice, c.nota.cabecera
                );
                let ids_numericos = p.alumnos.iter().filter(|a| a.id.chars().all(|ch| ch.is_ascii_digit())).count();
                println!(
                    "  {} alumnos, {} con nota · IDs solo cifras: {} · nombres con coma: {} · con espacios de más: {}",
                    p.alumnos.len(),
                    p.calificados(),
                    ids_numericos,
                    p.alumnos.iter().filter(|a| a.nombre.contains(',')).count(),
                    p.alumnos.iter().filter(|a| a.nombre.split_whitespace().collect::<Vec<_>>().join(" ") != a.nombre).count()
                );
                for a in &p.avisos {
                    let texto = match a {
                        Aviso::IdRepetido { filas, .. } => format!("IdRepetido en las filas {filas:?}"),
                        otro => format!("{otro:?}"),
                    };
                    println!("  AVISO {texto}");
                }
            }
            Err(e) => println!("  ERROR {e}"),
        }
    }
}

/// Las hojas del libro y, de la primera, la forma de sus primeras filas hasta
/// la cabecera incluida (la cabecera, con su texto).
fn hojas(ruta: &Path) {
    let Ok(mut libro) = open_workbook_auto(ruta) else { return };
    let nombres = libro.sheet_names().to_vec();
    println!("  hojas: {nombres:?}");
    let Some(Ok(rango)) = nombres.first().map(|h| libro.worksheet_range(h)) else { return };
    println!("  rango usado: {:?} → {:?}", rango.start(), rango.end());
    let cabecera = plantillas::leer(ruta).map(|p| p.columnas.fila_cabecera).unwrap_or(1);
    let (fila0, _) = rango.start().unwrap_or((0, 0));
    for (i, fila) in rango.rows().enumerate() {
        let n = fila0 + i as u32 + 1;
        if n > cabecera {
            break;
        }
        let celdas: Vec<String> = fila
            .iter()
            .map(|c| match c {
                Data::Empty => "·".into(),
                Data::String(s) if n == cabecera => format!("«{s}»"),
                Data::String(s) => format!("texto({})", s.chars().count()),
                Data::Float(_) | Data::Int(_) => "número".into(),
                otra => format!("{otra:?}"),
            })
            .collect();
        println!("  fila {n}: {}", celdas.join(" | "));
    }
    // La forma de la primera fila de datos, sin su contenido.
    if let Some(fila) = rango.rows().nth((cabecera - fila0) as usize) {
        let tipos: Vec<&str> = fila
            .iter()
            .map(|c| match c {
                Data::Empty => "vacía",
                Data::String(_) => "texto",
                Data::Float(_) | Data::Int(_) => "número",
                _ => "otra",
            })
            .collect();
        println!("  primera fila de datos (tipos): {}", tipos.join(" | "));
    }
}
