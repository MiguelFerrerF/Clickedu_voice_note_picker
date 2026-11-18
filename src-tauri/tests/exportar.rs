//! Exportar (H2): **ida y vuelta**. Una plantilla inventada, notas puestas en
//! la sesión, exportar, y releer lo exportado con el mismo lector de
//! plantillas: ha de salir lo mismo, con la nota como texto con punto, que es
//! como la v1 lo dejaba y Clickedu lo aceptaba.

mod comun;

use std::path::Path;

use calamine::{open_workbook, Data, Reader, Xlsx};
use comun::{plantilla, C::*};
use gestor_de_notas_lib::exportar::{exportar, ErrorExportar};
use gestor_de_notas_lib::plantillas::{self, Nota};
use gestor_de_notas_lib::sesion::Puestas;

/// Como las reales: `Id | Alumnos | Nota` en la fila 1.
fn la_plantilla(dir: &Path) -> std::path::PathBuf {
    plantilla(
        dir,
        "matematicas_3_a_eso.xlsx",
        &[
            &[T("Id"), T("Alumnos"), T("Nota")],
            &[N(20231.0), T("Castro Molina, Lucía"), V],
            &[N(20232.0), T("Martínez  Ruiz, Hugo"), N(4.0)],
            &[T("007"), T("Ionescu, Ștefan"), T("NP")],
            &[N(20234.0), T("Nowak, Łukasz"), N(9.0)],
        ],
    )
}

fn puestas(pares: &[(&str, Option<f64>)]) -> Puestas {
    pares.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn ida_y_vuelta() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = la_plantilla(dir.path());
    let antes = std::fs::read(&ruta).unwrap();
    let p = plantillas::leer(&ruta).unwrap();

    // La primera puesta, el NP sustituido y la de Nowak quitada a propósito.
    let notas = puestas(&[("20231", Some(7.5)), ("007", Some(6.0)), ("20234", None)]);
    let carpeta = dir.path().join("notas");
    let hecho = exportar(&p, &notas, "matematicas_3_a_eso.xlsx", &carpeta).unwrap();

    let exportado = carpeta.join("notas_matematicas_3_a_eso.xlsx");
    assert_eq!(hecho.ruta, exportado.display().to_string());
    assert_eq!((hecho.con_nota, hecho.alumnos, hecho.viejo), (3, 4, None));
    // La plantilla, intacta (D4).
    assert_eq!(std::fs::read(&ruta).unwrap(), antes);

    // Releído con el lector de plantillas: lo mismo, y sin nada que avisar.
    let vuelta = plantillas::leer(&exportado).unwrap();
    assert!(vuelta.avisos.is_empty(), "{:?}", vuelta.avisos);
    assert_eq!(vuelta.columnas.hoja, "Sheet1");
    let c = &vuelta.columnas;
    assert_eq!((c.id.cabecera.as_str(), c.nombre.cabecera.as_str(), c.nota.cabecera.as_str()), ("Id", "Alumnos", "Nota"));
    let filas: Vec<(&str, &str, &Nota)> = vuelta.alumnos.iter().map(|a| (a.id.as_str(), a.nombre.as_str(), &a.nota)).collect();
    assert_eq!(
        filas,
        [
            ("20231", "Castro Molina, Lucía", &Nota::Valor(7.5)),
            // El nombre, tal cual, con su doble espacio.
            ("20232", "Martínez  Ruiz, Hugo", &Nota::Valor(4.0)),
            ("007", "Ionescu, Ștefan", &Nota::Valor(6.0)),
            ("20234", "Nowak, Łukasz", &Nota::Vacia),
        ]
    );

    // Celda a celda: la nota como texto con punto y el ID como número, salvo
    // el que empieza por cero.
    let mut libro: Xlsx<_> = open_workbook(&exportado).unwrap();
    let hoja = libro.worksheet_range("Sheet1").unwrap();
    let col = |c: usize| hoja.rows().skip(1).map(|f| f[c].clone()).collect::<Vec<_>>();
    assert_eq!(
        col(2),
        [Data::String("7.5".into()), Data::String("4".into()), Data::String("6".into()), Data::Empty]
    );
    assert_eq!(
        col(0),
        [Data::Float(20231.0), Data::Float(20232.0), Data::String("007".into()), Data::Float(20234.0)]
    );
}

#[test]
fn el_exportado_viejo_se_aparta() {
    let dir = tempfile::tempdir().unwrap();
    let p = plantillas::leer(&la_plantilla(dir.path())).unwrap();
    let carpeta = dir.path().join("notas");
    let notas = |v: f64| puestas(&[("007", Some(v))]);

    assert_eq!(exportar(&p, &notas(5.0), "matematicas_3_a_eso.xlsx", &carpeta).unwrap().viejo, None);
    let segundo = exportar(&p, &notas(6.0), "matematicas_3_a_eso.xlsx", &carpeta).unwrap();
    let tercero = exportar(&p, &notas(7.0), "matematicas_3_a_eso.xlsx", &carpeta).unwrap();

    let viejas = carpeta.join("notas_viejas");
    assert_eq!(segundo.viejo, Some(viejas.join("notas_matematicas_3_a_eso.xlsx").display().to_string()));
    assert_eq!(tercero.viejo, Some(viejas.join("notas_matematicas_3_a_eso (2).xlsx").display().to_string()));

    let nota_de_ionescu = |ruta: &Path| plantillas::leer(ruta).unwrap().alumnos[2].nota.clone();
    assert_eq!(nota_de_ionescu(&carpeta.join("notas_matematicas_3_a_eso.xlsx")), Nota::Valor(7.0));
    assert_eq!(nota_de_ionescu(&viejas.join("notas_matematicas_3_a_eso.xlsx")), Nota::Valor(5.0));
    assert_eq!(nota_de_ionescu(&viejas.join("notas_matematicas_3_a_eso (2).xlsx")), Nota::Valor(6.0));
    // Ningún temporal suelto.
    assert_eq!(std::fs::read_dir(&carpeta).unwrap().count(), 2);
}

#[test]
fn lo_que_no_es_nota_no_se_exporta() {
    let dir = tempfile::tempdir().unwrap();
    let p = plantillas::leer(&la_plantilla(dir.path())).unwrap();
    let carpeta = dir.path().join("notas");

    // El «NP» de la fila 4 sigue ahí.
    let error = exportar(&p, &Puestas::new(), "matematicas_3_a_eso.xlsx", &carpeta).unwrap_err();
    assert_eq!(error, ErrorExportar::NotasInvalidas(vec![4]));
    assert!(!carpeta.exists());

    // Quitado a propósito, ya se exporta, sin nota.
    let hecho = exportar(&p, &puestas(&[("007", None)]), "matematicas_3_a_eso.xlsx", &carpeta).unwrap();
    assert_eq!(hecho.con_nota, 2);
}
