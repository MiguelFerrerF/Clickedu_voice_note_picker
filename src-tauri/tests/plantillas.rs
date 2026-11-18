//! Leer plantillas de Clickedu (H1) sobre plantillas **inventadas**, escritas
//! al vuelo por cada test: nunca alumnos reales en el repo.

use std::path::{Path, PathBuf};

use gestor_de_notas_lib::plantillas::{self, Aviso, ErrorPlantilla, Lectura, Nota};
use rust_xlsxwriter::Workbook;

/// Una celda de la plantilla de prueba.
enum C {
    T(&'static str),
    N(f64),
    /// Vacía.
    V,
}
use C::{N, T, V};

/// Escribe `filas` en la primera hoja de `nombre`, desde la A1.
fn plantilla(dir: &Path, nombre: &str, filas: &[&[C]]) -> PathBuf {
    let mut libro = Workbook::new();
    let hoja = libro.add_worksheet();
    for (f, fila) in filas.iter().enumerate() {
        for (c, celda) in fila.iter().enumerate() {
            match celda {
                T(t) => {
                    hoja.write_string(f as u32, c as u16, *t).unwrap();
                }
                N(n) => {
                    hoja.write_number(f as u32, c as u16, *n).unwrap();
                }
                V => {}
            }
        }
    }
    let ruta = dir.join(nombre);
    libro.save(&ruta).unwrap();
    ruta
}

#[test]
fn la_de_la_v1_id_nombre_nota() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = plantilla(
        dir.path(),
        "matematicas_3_a_eso.xlsx",
        &[
            &[T("ID"), T("Alumno"), T("Nota")],
            &[N(1001.0), T("Castro Molina, Lucía"), V],
            &[N(1002.0), T("Martínez Ruiz, Hugo"), N(8.7)],
            &[N(1003.0), T("Ionescu, Ștefan"), T("3,4")],
        ],
    );
    let p = plantillas::leer(&ruta).unwrap();

    assert_eq!(p.alumnos.len(), 3);
    assert_eq!(p.calificados(), 2);
    assert_eq!(p.columnas.fila_cabecera, 1);
    assert_eq!((p.columnas.id.indice, p.columnas.nombre.indice, p.columnas.nota.indice), (0, 1, 2));

    let a = &p.alumnos;
    assert_eq!((a[0].fila, a[0].id.as_str(), a[0].nombre.as_str()), (2, "1001", "Castro Molina, Lucía"));
    assert_eq!(a[0].nota, Nota::Vacia);
    assert_eq!(a[1].nota, Nota::Valor(8.7));
    assert_eq!(a[2].nombre, "Ionescu, Ștefan");
    assert_eq!(a[2].nota, Nota::Valor(3.4));
    // La coma se dice.
    assert_eq!(p.avisos, vec![Aviso::NotasConComa { filas: vec![4] }]);
}

#[test]
fn columnas_por_cabecera_en_otro_orden_y_en_catalan() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = plantilla(
        dir.path(),
        "Matemàtiques - 3r ESO A.xlsx",
        &[
            &[T("Llistat de l'assignatura")],
            &[],
            &[T("Qualificació"), T("Codi"), T("Cognoms i nom de l'alumne"), T("Observacions")],
            &[N(5.0), T("A-77"), T("Navarro Gil, Mateo"), T("—")],
            &[V, T("A-78"), T("Pérez Gómez, Irene"), V],
        ],
    );
    let p = plantillas::leer(&ruta).unwrap();

    assert!(p.avisos.is_empty(), "{:?}", p.avisos);
    assert_eq!(p.columnas.fila_cabecera, 3);
    assert_eq!(p.columnas.nota.cabecera, "Qualificació");
    assert_eq!(p.columnas.id.cabecera, "Codi");
    assert_eq!(p.columnas.nombre.indice, 2);
    assert_eq!(p.alumnos[0].id, "A-77");
    assert_eq!(p.alumnos[0].nota, Nota::Valor(5.0));
    assert_eq!(p.alumnos[1].fila, 5);
}

#[test]
fn sin_cabeceras_reconocidas_va_por_posicion_y_lo_dice() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = plantilla(
        dir.path(),
        "x.xlsx",
        &[&[T("Col1"), T("Col2"), T("Col3")], &[N(1.0), T("Ramos Ortega, Martina"), N(6.0)]],
    );
    let p = plantillas::leer(&ruta).unwrap();
    assert_eq!(p.avisos, vec![Aviso::ColumnasPorPosicion]);
    assert_eq!(p.alumnos[0].nombre, "Ramos Ortega, Martina");
    assert_eq!(p.alumnos[0].nota, Nota::Valor(6.0));
}

#[test]
fn lo_que_no_cuadra_se_queda_y_se_dice() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = plantilla(
        dir.path(),
        "y.xlsx",
        &[
            &[T("ID"), T("Nombre"), T("Nota")],
            &[N(1.0), T("  Castro   Molina, Lucía "), N(12.0)],
            &[N(2.0), V, T("NP")],
            &[],
            &[N(1.0), T("Martínez Ruiz, Hugo"), N(7.0)],
            &[V, V, N(4.0)],
        ],
    );
    let p = plantillas::leer(&ruta).unwrap();

    // La fila vacía y la que solo tiene nota no son alumnos.
    assert_eq!(p.alumnos.len(), 3);
    // El nombre, tal cual: los dobles espacios son de Clickedu y no se tocan.
    assert_eq!(p.alumnos[0].nombre, "  Castro   Molina, Lucía ");
    assert_eq!(p.alumnos[0].nota, Nota::Invalida("12".into()));
    assert_eq!(p.alumnos[1].nota, Nota::Invalida("NP".into()));
    assert_eq!(p.calificados(), 1);
    assert_eq!(
        p.avisos,
        vec![
            Aviso::NotasInvalidas { filas: vec![2, 3] },
            Aviso::SinNombre { filas: vec![3] },
            Aviso::IdRepetido { id: "1".into(), filas: vec![2, 5] },
        ]
    );
}

#[test]
fn errores() {
    let dir = tempfile::tempdir().unwrap();
    let falso = dir.path().join("falso.xlsx");
    std::fs::write(&falso, "esto es un csv, no un excel").unwrap();
    assert_eq!(plantillas::leer(&falso).unwrap_err(), ErrorPlantilla::FormatoDesconocido);

    assert!(matches!(plantillas::leer(&dir.path().join("no-esta.xlsx")), Err(ErrorPlantilla::NoAbre(_))));

    let vacia = plantilla(dir.path(), "vacia.xlsx", &[]);
    assert_eq!(plantillas::leer(&vacia).unwrap_err(), ErrorPlantilla::SinDatos);

    let solo_cabecera = plantilla(dir.path(), "cab.xlsx", &[&[T("ID"), T("Alumno"), T("Nota")]]);
    assert_eq!(plantillas::leer(&solo_cabecera).unwrap_err(), ErrorPlantilla::SinAlumnos);
}

#[test]
fn listar_la_carpeta() {
    let dir = tempfile::tempdir().unwrap();
    let cab: &[C] = &[T("ID"), T("Alumno"), T("Nota")];
    plantilla(dir.path(), "matematicas_3_a_eso.xlsx", &[cab, &[N(1.0), T("A"), N(5.0)], &[N(2.0), T("B"), V]]);
    plantilla(dir.path(), "biologia_1_ab_bach.xlsx", &[cab, &[N(1.0), T("A"), V]]);
    std::fs::write(dir.path().join("rota.xlsx"), "nada").unwrap();
    // Lo que no es plantilla no aparece: el bloqueo de Excel, otros ficheros,
    // las subcarpetas.
    std::fs::write(dir.path().join("~$matematicas_3_a_eso.xlsx"), "bloqueo").unwrap();
    std::fs::write(dir.path().join("léeme.txt"), "").unwrap();
    std::fs::create_dir(dir.path().join("notas.xlsx")).unwrap();

    let clases = plantillas::listar(dir.path()).unwrap();
    let nombres: Vec<&str> = clases.iter().map(|c| c.nombre.as_str()).collect();
    assert_eq!(nombres, ["Biología 1.º AB Bachillerato", "Matemáticas 3.º A ESO", "Rota"]);

    assert!(matches!(clases[1].lectura, Lectura::Bien { alumnos: 2, calificados: 1, avisos: 0 }));
    assert!(matches!(&clases[2].lectura, Lectura::Mal { error } if error.contains("no es un Excel")));
}

#[test]
fn la_plantilla_no_se_toca() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = plantilla(dir.path(), "z.xlsx", &[&[T("ID"), T("Alumno"), T("Nota")], &[N(1.0), T("A"), T("7,5")]]);
    let antes = std::fs::read(&ruta).unwrap();
    let modificada = std::fs::metadata(&ruta).unwrap().modified().unwrap();
    plantillas::leer(&ruta).unwrap();
    plantillas::listar(dir.path()).unwrap();
    assert_eq!(std::fs::read(&ruta).unwrap(), antes);
    assert_eq!(std::fs::metadata(&ruta).unwrap().modified().unwrap(), modificada);
}
