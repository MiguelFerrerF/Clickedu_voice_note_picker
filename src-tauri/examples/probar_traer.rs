//! H5, la prueba real de **traer las plantillas**: entra con la cuenta
//! guardada, baja, prepara y planea contra la carpeta de plantillas de
//! verdad, y enseña el plan **sin nombres de alumnos** (ficheros, estados y
//! recuentos).
//!
//!     cargo run --example probar_traer              # solo el plan
//!     cargo run --example probar_traer -- --aplicar # y lo hace, en una copia
//!     cargo run --example probar_traer -- --aplicar --a-ninguna
//!
//! - Contra Clickedu, lo mismo que la app (`clickedu::traer::bajar`): la
//!   página de materias y, de cada una, la de Ítems y la descarga. Solo
//!   `GET`.
//! - Lo bajado va a `%APPDATA%\es.gestornotas.app\clickedu\probar-traer\bajadas\`,
//!   no a la `bajadas` de la app.
//! - **La carpeta de plantillas, `sesion.json` y `traidas.json` de verdad
//!   solo se leen.** Con `--aplicar`, se copian a
//!   `clickedu\probar-traer\copia\` y se aplica ahí; luego se vuelve a
//!   planear contra la copia, que ha de salir todo igual. La copia (con
//!   nombres de alumnos) **se queda** para mirarla, en local; la siguiente
//!   vuelta la sustituye.
//! - Sin las huellas: la prueba no guarda nada en `clickedu\huellas`.
//! - Las dudas no se responden: sus viejas se quedan en la raíz de la copia.
//!   Con `--a-ninguna`, a todas se responde «a ninguna»: se apartan y sus
//!   notas no pasan.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use gestor_de_notas_lib::ajustes;
use gestor_de_notas_lib::clickedu::cliente::{self, Cliente};
use gestor_de_notas_lib::clickedu::cuenta::{self, LlaveroWindows};
use gestor_de_notas_lib::clickedu::fichas::Estado;
use gestor_de_notas_lib::clickedu::traer::aplicar::{self, Hecho, Rutas, ANTIGUAS};
use gestor_de_notas_lib::clickedu::traer::planear::{self, EstadoMateria, Plan, Respuesta};
use gestor_de_notas_lib::clickedu::traer::{self, Bajado};
use gestor_de_notas_lib::registro;
use gestor_de_notas_lib::sesion::{self, Cerrojo, Sesion};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let aplicar_en_copia = args.iter().any(|a| a == "--aplicar");
    let a_ninguna = args.iter().any(|a| a == "--a-ninguna");

    let app = dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app");
    let carpeta_ce = app.join("clickedu");
    let prueba = carpeta_ce.join("probar-traer");
    let bajadas = prueba.join("bajadas");
    let _ = std::fs::remove_dir_all(&bajadas);

    let documentos = dirs::document_dir().expect("no se encuentra la carpeta Documentos");
    let ajustes = ajustes::leer(&app.join("ajustes.json")).unwrap_or_else(|e| panic!("{e}"));
    let plantillas = PathBuf::from(ajustes::carpeta_plantillas(&ajustes, &documentos).ruta);
    let ruta_sesion = app.join("sesion.json");
    let ruta_recordadas = carpeta_ce.join("traidas.json");
    println!("Carpeta de plantillas: {}", plantillas.display());

    // Entrar.
    let guardada = cuenta::leer(&carpeta_ce, &LlaveroWindows)
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("no hay cuenta guardada: entra antes desde la pantalla Conexión");
    let archivo = guardada.archivo.as_ref().expect("falta el archivo de seguridad");
    let archivo = cuenta::leer_archivo(archivo).unwrap_or_else(|e| panic!("{e}"));
    let cliente = Cliente::new(cliente::BASE).unwrap_or_else(|e| panic!("{e}"));
    println!("Entrando…");
    bloquear(cliente.entrar(&guardada.usuario, &guardada.contrasena, (&archivo.0, &archivo.1)))
        .unwrap_or_else(|e| panic!("no se ha podido entrar: {e}"));

    // Bajar.
    let inicio = Instant::now();
    let Bajado { materias, bajadas: lo_bajado, .. } = bloquear(traer::bajar(&cliente, |hechas, total| {
        println!("  bajadas {hechas} de {total} ({:.1} s)", inicio.elapsed().as_secs_f32());
    }))
    .unwrap_or_else(|e| panic!("no se ha podido traer: {e}"));
    println!("{} materias, leídas {}.", materias.dato.len(), estado(&materias.estado));

    // Preparar y planear, contra la carpeta de verdad (solo leyendo).
    let traidas = traer::preparar(lo_bajado, &bajadas).unwrap_or_else(|e| panic!("{e}"));
    let raiz = traer::leer_raiz(&plantillas).unwrap_or_else(|e| panic!("{e}"));
    for r in &raiz {
        if let Err(e) = &r.lectura {
            println!("  (en la raíz, «{}» no se lee: {e})", r.fichero);
        }
    }
    let s = sesion::leer(&ruta_sesion).unwrap_or_else(|e| panic!("{e}"));
    let recordadas = aplicar::leer_recordadas(&ruta_recordadas).unwrap_or_else(|e| panic!("{e}"));
    println!(
        "En la raíz, {} plantillas; en la sesión, notas puestas en {} clases; {} materias recordadas{}.",
        raiz.len(),
        s.clases.values().filter(|p| !p.is_empty()).count(),
        recordadas.materias.len(),
        if recordadas.confirmada { ", ya confirmada la primera vez" } else { ": sería la primera vez" }
    );
    let mut plan = planear::planear(&traidas, &raiz, &s, &recordadas, &HashMap::new());
    println!("\n══ El plan, contra la carpeta de verdad");
    ensenar(&plan);
    let respuestas: HashMap<String, Respuesta> = if a_ninguna {
        plan.dudas.iter().map(|d| (d.fichero.clone(), Respuesta::Ninguna)).collect()
    } else {
        HashMap::new()
    };
    if !respuestas.is_empty() {
        plan = planear::planear(&traidas, &raiz, &s, &recordadas, &respuestas);
        println!("\n══ El plan, respondiendo «a ninguna» a las {} dudas", respuestas.len());
        ensenar(&plan);
    }

    if !aplicar_en_copia {
        let _ = std::fs::remove_dir_all(&bajadas);
        println!("\nNo se ha tocado nada (lo bajado, borrado). Para hacerlo en una copia: -- --aplicar");
        return;
    }

    // Aplicar, en una copia.
    let copia = prueba.join("copia");
    let _ = std::fs::remove_dir_all(&copia);
    let copia_plantillas = copia.join("plantillas");
    let copia_sesion = copia.join("sesion.json");
    let copia_recordadas = copia.join("traidas.json");
    copiar_raiz(&plantillas, &copia_plantillas);
    copiar_si_esta(&ruta_sesion, &copia_sesion);
    copiar_si_esta(&ruta_recordadas, &copia_recordadas);

    let rutas =
        Rutas { plantillas: &copia_plantillas, sesion: &copia_sesion, recordadas: &copia_recordadas, bajadas: &bajadas };
    let (a, m, d) = registro::fecha(registro::ahora());
    let hecho = aplicar::aplicar(&plan, &rutas, &format!("{a:04}-{m:02}-{d:02}"), &Cerrojo::default());
    println!("\n══ Hecho, en la copia ({})", copia.display());
    ensenar_hecho(&hecho);
    println!("\nLa raíz de la copia:");
    listar(&copia_plantillas, "  ");
    println!("Y en «{ANTIGUAS}»:");
    listar(&copia_plantillas.join(ANTIGUAS), "  ");
    let notas_antes = contar_notas(&s);
    let despues = sesion::leer(&copia_sesion).unwrap_or_else(|e| panic!("{e}"));
    println!("Notas puestas en la sesión: {notas_antes} antes, {} después.", contar_notas(&despues));

    // Una segunda vuelta, contra la copia: ha de salir todo igual.
    let raiz = traer::leer_raiz(&copia_plantillas).unwrap_or_else(|e| panic!("{e}"));
    let recordadas = aplicar::leer_recordadas(&copia_recordadas).unwrap_or_else(|e| panic!("{e}"));
    let otra = planear::planear(&traidas, &raiz, &despues, &recordadas, &HashMap::new());
    println!("\n══ Otra vuelta, contra la copia (sin volver a bajar)");
    let distintas = otra
        .materias
        .iter()
        .filter(|m| !matches!(m.estado, EstadoMateria::Igual { .. } | EstadoMateria::Vacia | EstadoMateria::Fallo { .. }))
        .count();
    // Las dudas sin responder se quedan en la raíz: han de salir las mismas.
    if distintas == 0 && otra.apartadas.is_empty() && otra.dudas == plan.dudas && recordadas.confirmada {
        println!(
            "Todo igual: nada que escribir ni apartar, la primera vez confirmada y las mismas {} dudas sin responder.",
            otra.dudas.len()
        );
    } else {
        println!("NO sale todo igual ({distintas} materias por escribir):");
        ensenar(&otra);
    }
}

/// El plan, sin nombres de alumnos.
fn ensenar(plan: &Plan) {
    let (comunes, pendientes): (Vec<_>, Vec<_>) = plan.materias.iter().partition(|m| !m.pendiente);
    for (titulo, grupo) in [("Materias", comunes), ("Pendientes", pendientes)] {
        if grupo.is_empty() {
            continue;
        }
        println!("\n{titulo}:");
        for m in grupo {
            let que = match &m.estado {
                EstadoMateria::Nueva { alumnos } => format!("NUEVA, {alumnos} alumnos"),
                EstadoMateria::Igual { alumnos } => format!("igual, {alumnos} alumnos"),
                EstadoMateria::Cambia { alumnos } => format!("CAMBIA, {alumnos} alumnos"),
                EstadoMateria::Vacia => "sin alumnos en Clickedu: no se guarda".into(),
                EstadoMateria::Fallo { fallo } => format!("FALLO: {fallo:?}"),
            };
            println!("  {} · {que}", m.fichero);
            println!(
                "      Clickedu: «{}» · «{}» · «{}» (id {})",
                m.materia.nombre,
                m.materia.grupo,
                m.materia.curso.as_deref().unwrap_or("—"),
                m.materia.id
            );
            for r in &m.retoques {
                println!("      retoque: «{}» → «{}»", r.de, r.a);
            }
            for a in &m.avisos {
                println!("      aviso: {a:?}");
            }
            if let Some(e @ Estado::Degradado { .. }) = &m.estado_evaluaciones {
                println!("      evaluaciones leídas {}", estado(e));
            }
        }
    }
    println!("\nA «{ANTIGUAS}» ({}):", plan.apartadas.len());
    for a in &plan.apartadas {
        match &a.notas {
            None if a.descartadas > 0 => println!("  {} · sus {} notas NO pasan (a ninguna)", a.fichero, a.descartadas),
            None => println!("  {} · sin notas", a.fichero),
            Some(p) => println!(
                "  {} · sus notas → {} (por {:?}): pasan {}, ya no están en Clickedu {}, sin Id que sirva {}",
                a.fichero, p.a, p.por, p.pasan, p.sin_alumno, p.sin_id
            ),
        }
    }
    println!("\nDudas ({}):", plan.dudas.len());
    for d in &plan.dudas {
        println!("  {} · {} con nota; ¿a cuál?: {}", d.fichero, d.notas, d.candidatas.join(" | "));
    }
}

fn ensenar_hecho(h: &Hecho) {
    println!("Escritas ({}): {}", h.escritas.len(), h.escritas.join(" | "));
    println!("Apartadas ({}):", h.movidas.len());
    for m in &h.movidas {
        println!("  {} → {}", m.fichero, m.a);
    }
    println!("Notas que han pasado: {}", h.pasan);
    println!("Fallos ({}):", h.fallos.len());
    for f in &h.fallos {
        println!("  {f:?}");
    }
}

fn estado(e: &Estado) -> String {
    match e {
        Estado::Bien => "bien".into(),
        Estado::Degradado { motivo } => format!("DEGRADADO ({motivo})"),
    }
}

fn contar_notas(s: &Sesion) -> usize {
    s.clases.values().flat_map(|p| p.values()).filter(|n| n.is_some()).count()
}

/// Los ficheros de la raíz de `de` (sin subcarpetas: `antiguas` no cuenta).
fn copiar_raiz(de: &Path, a: &Path) {
    std::fs::create_dir_all(a).expect("no se puede crear la copia");
    let Ok(entradas) = std::fs::read_dir(de) else { return };
    for e in entradas.flatten() {
        if e.file_type().is_ok_and(|t| t.is_file()) {
            std::fs::copy(e.path(), a.join(e.file_name()))
                .unwrap_or_else(|err| panic!("no se puede copiar {}: {err}", e.path().display()));
        }
    }
}

fn copiar_si_esta(de: &Path, a: &Path) {
    if de.is_file() {
        std::fs::copy(de, a).unwrap_or_else(|e| panic!("no se puede copiar {}: {e}", de.display()));
    }
}

/// Los ficheros de `carpeta`, por subcarpetas.
fn listar(carpeta: &Path, sangria: &str) {
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        println!("{sangria}(nada)");
        return;
    };
    let entradas: BTreeMap<String, bool> = entradas
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.file_type().is_ok_and(|t| t.is_dir())))
        .collect();
    if entradas.is_empty() {
        println!("{sangria}(nada)");
    }
    for (nombre, es_carpeta) in entradas {
        if es_carpeta {
            if nombre != ANTIGUAS {
                println!("{sangria}{nombre}\\");
                listar(&carpeta.join(&nombre), &format!("{sangria}  "));
            }
        } else {
            println!("{sangria}{nombre}");
        }
    }
}

fn bloquear<F: std::future::Future>(f: F) -> F::Output {
    tauri::async_runtime::block_on(f)
}
