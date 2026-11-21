//! H5, lo primero: **¿cambia la plantilla con la evaluación?** Entra con la
//! cuenta guardada y, de cada materia, baja la plantilla de cada evaluación
//! y las compara, **solo leyendo** y **sin enseñar nombres**.
//!
//!     cargo run --example comparar_plantillas [-- --materias 3]
//!
//! - Solo `GET`, una petición por segundo: la página de materias, la de
//!   Ítems de cada una (las evaluaciones, `trm`) y la descarga
//!   (`importar_notes_items.php?accio=descarregar_plantilla&assig=N&id_aval=T`,
//!   la que usaba la v1).
//! - Las plantillas bajadas (con nombres de alumnos) van a
//!   `%APPDATA%\es.gestornotas.app\clickedu\comparar\` solo para leerlas, y
//!   **se borran al acabar**.
//! - Lo que sale por pantalla: la materia y el grupo (rótulos de Clickedu),
//!   el nombre que pone Clickedu al fichero, y por evaluación solo
//!   recuentos y si es igual a la primera.

use std::path::Path;
use std::time::Duration;

use calamine::{open_workbook_auto, Data, Reader};
use gestor_de_notas_lib::clickedu::cliente::{self, Cliente};
use gestor_de_notas_lib::clickedu::cuenta::{self, LlaveroWindows};
use gestor_de_notas_lib::clickedu::fichas;
use gestor_de_notas_lib::plantillas::{self, Nota};

const PAUSA: Duration = Duration::from_secs(1);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tope = args
        .iter()
        .position(|a| a == "--materias")
        .and_then(|i| args.get(i + 1))
        .map(|n| n.parse::<usize>().expect("--materias es un número"))
        .unwrap_or(usize::MAX);

    let app = dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app");
    let carpeta = app.join("clickedu");
    let bajadas = carpeta.join("comparar");
    let _ = std::fs::remove_dir_all(&bajadas);
    std::fs::create_dir_all(&bajadas).expect("no se puede crear la carpeta de las bajadas");

    let guardada = cuenta::leer(&carpeta, &LlaveroWindows)
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("no hay cuenta guardada: entra antes desde la pantalla Conexión");
    let archivo = guardada.archivo.as_ref().expect("falta el archivo de seguridad");
    let archivo = cuenta::leer_archivo(archivo).unwrap_or_else(|e| panic!("{e}"));

    let cliente = Cliente::new(cliente::BASE).unwrap_or_else(|e| panic!("{e}"));
    println!("Entrando…");
    bloquear(cliente.entrar(&guardada.usuario, &guardada.contrasena, (&archivo.0, &archivo.1)))
        .unwrap_or_else(|e| panic!("no se ha podido entrar: {e}"));

    let resultado = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| recorrer(&cliente, &bajadas, tope)));
    let _ = std::fs::remove_dir_all(&bajadas);
    println!("\nLas plantillas bajadas, borradas ({}).", bajadas.display());
    if resultado.is_err() {
        std::process::exit(1);
    }
}

fn recorrer(cliente: &Cliente, bajadas: &Path, tope: usize) {
    let pagina = abrir(cliente, cliente::ASIGNATURAS);
    let materias = fichas::materias(&pagina).unwrap_or_else(|e| panic!("{e}"));
    println!("{} materias ({:?}).", materias.dato.len(), materias.estado);

    let (mut iguales, mut distintas, mut sin_plantilla) = (0, 0, 0);
    for (n, m) in materias.dato.iter().take(tope).enumerate() {
        println!("\n── {} · {} · {} (curso: {})", n + 1, m.nombre, m.grupo, m.curso.as_deref().unwrap_or("—"));
        let items = abrir(cliente, &format!("/go/Index.php?ctr=Subject/Items&do=Index&id={}", m.id));
        let evaluaciones = match fichas::evaluaciones(&items) {
            Ok(e) => e.dato,
            Err(e) => {
                println!("  sin evaluaciones: {e}");
                continue;
            }
        };
        let mut primera: Option<Bajada> = None;
        for ev in &evaluaciones {
            let direccion = format!(
                "/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig={}&id_aval={}",
                m.id, ev.id
            );
            std::thread::sleep(PAUSA);
            let leida = bloquear(cliente.leer(&direccion)).unwrap_or_else(|e| panic!("{e}"));
            let es_excel = leida.cuerpo.starts_with(b"PK");
            if leida.estado != 200 || !es_excel {
                println!(
                    "  {}: no es un Excel (estado {}, tipo «{}», {} bytes, disposición «{}»)",
                    ev.nombre,
                    leida.estado,
                    leida.tipo,
                    leida.cuerpo.len(),
                    leida.disposicion
                );
                sin_plantilla += 1;
                continue;
            }
            let ruta = bajadas.join(format!("{}-{}.xlsx", m.id, ev.id));
            std::fs::write(&ruta, &leida.cuerpo).expect("no se puede escribir la bajada");
            let b = Bajada::leer(&ruta, leida.cuerpo.len(), &leida.disposicion);
            print!(
                "  {} (id_aval {}): {} bytes, fichero «{}», {} alumnos, {} con nota, {} filas × {} columnas",
                ev.nombre, ev.id, b.bytes, b.fichero, b.alumnos.len(), b.con_nota, b.filas, b.columnas
            );
            match &primera {
                None => {
                    println!(" ← la primera");
                    primera = Some(b);
                }
                Some(p) => {
                    let diferencias = p.diferencias(&b);
                    if diferencias.is_empty() {
                        println!(" · IGUAL a la primera, celda a celda");
                        iguales += 1;
                    } else {
                        println!(" · DISTINTA: {}", diferencias.join("; "));
                        distintas += 1;
                    }
                }
            }
        }
    }
    println!(
        "\nResumen: {iguales} iguales a la primera de su materia, {distintas} distintas, {sin_plantilla} sin plantilla."
    );
}

/// Una plantilla bajada, leída: lo justo para compararla, sin enseñarlo.
struct Bajada {
    bytes: usize,
    fichero: String,
    hojas: Vec<String>,
    celdas: Vec<Vec<String>>,
    filas: usize,
    columnas: usize,
    /// (id, nombre, nota), en su orden.
    alumnos: Vec<(String, String, String)>,
    con_nota: usize,
}

impl Bajada {
    fn leer(ruta: &Path, bytes: usize, disposicion: &str) -> Self {
        let mut libro = open_workbook_auto(ruta).expect("no se abre la bajada");
        let hojas = libro.sheet_names().to_vec();
        let rango = libro.worksheet_range(&hojas[0]).expect("no se lee la primera hoja");
        let celdas: Vec<Vec<String>> =
            rango.rows().map(|f| f.iter().map(|c| if matches!(c, Data::Empty) { String::new() } else { c.to_string() }).collect()).collect();
        let (filas, columnas) = rango.get_size();
        let (alumnos, con_nota) = match plantillas::leer(ruta) {
            Ok(p) => (
                p.alumnos.iter().map(|a| (a.id.clone(), a.nombre.clone(), format!("{:?}", a.nota))).collect(),
                p.alumnos.iter().filter(|a| !matches!(a.nota, Nota::Vacia)).count(),
            ),
            Err(e) => {
                let cabecera = celdas.first().map(|f| f.join(" | ")).unwrap_or_default();
                println!("  (plantillas.rs no la lee: {e}; hojas {hojas:?}, fila 1: «{cabecera}»)");
                (Vec::new(), 0)
            }
        };
        let fichero = disposicion
            .split("filename=")
            .nth(1)
            .map(|f| f.trim_matches(|c| c == '"' || c == ';' || c == ' ').to_string())
            .unwrap_or_default();
        Self { bytes, fichero, hojas, celdas, filas, columnas, alumnos, con_nota }
    }

    /// En qué se diferencia `otra` de esta, sin decir ningún nombre.
    fn diferencias(&self, otra: &Bajada) -> Vec<String> {
        let mut d = Vec::new();
        if self.hojas != otra.hojas {
            d.push(format!("las hojas ({:?} y {:?})", self.hojas, otra.hojas));
        }
        if (self.filas, self.columnas) != (otra.filas, otra.columnas) {
            d.push("el tamaño".into());
        }
        let ids = |b: &Bajada| b.alumnos.iter().map(|a| a.0.clone()).collect::<Vec<_>>();
        let (a, b) = (ids(self), ids(otra));
        if a != b {
            let mut sa = a.clone();
            let mut sb = b.clone();
            sa.sort();
            sb.sort();
            if sa == sb {
                d.push("los mismos Id, en otro orden".into());
            } else {
                let solo_a = a.iter().filter(|x| !b.contains(x)).count();
                let solo_b = b.iter().filter(|x| !a.contains(x)).count();
                d.push(format!("los Id: {solo_a} solo en la primera, {solo_b} solo en esta"));
            }
        }
        let nombres_distintos = self
            .alumnos
            .iter()
            .filter(|(id, nombre, _)| otra.alumnos.iter().any(|(i, n, _)| i == id && n != nombre))
            .count();
        if nombres_distintos > 0 {
            d.push(format!("{nombres_distintos} alumnos con el mismo Id y otro nombre"));
        }
        let notas_distintas = self
            .alumnos
            .iter()
            .filter(|(id, _, nota)| otra.alumnos.iter().any(|(i, _, n)| i == id && n != nota))
            .count();
        if notas_distintas > 0 {
            d.push(format!("{notas_distintas} alumnos con otra nota"));
        }
        if d.is_empty() && self.celdas != otra.celdas {
            let celdas = self
                .celdas
                .iter()
                .flatten()
                .zip(otra.celdas.iter().flatten())
                .filter(|(x, y)| x != y)
                .count();
            d.push(format!("{celdas} celdas fuera de Id, alumno y nota"));
        }
        d
    }
}

fn abrir(cliente: &Cliente, direccion: &str) -> String {
    std::thread::sleep(PAUSA);
    bloquear(cliente.abrir(direccion)).unwrap_or_else(|e| panic!("{direccion}: {e}")).texto
}

fn bloquear<F: std::future::Future>(f: F) -> F::Output {
    tauri::async_runtime::block_on(f)
}
