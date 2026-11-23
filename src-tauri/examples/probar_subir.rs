//! H6, la primera subida, paso a paso y **con el usuario delante**: subir
//! unas notas de prueba a un ítem de prueba (ponderación vacía, sin que lo
//! vean las familias) y ver qué hace Clickedu en cada paso.
//!
//!     cargo run --example probar_subir -- --materia M --evaluacion T --item I
//!         solo leyendo: la página de importar, la plantilla (se baja) y el
//!         Excel de prueba que se subiría; las notas de antes (`GetGrades`)
//!     … --subir
//!         **escribe**: el `POST` de `carregar_plantilla` con ese Excel; luego
//!         vuelve a leer la página de importar y enseña lo subido. No importa.
//!     … --importar <id>
//!         **escribe**: abre el enlace `accio=importar_notes` del subido
//!         `<id>` (el que trae la página, no uno inventado) y compara las
//!         notas de `GetGrades` con las del Excel.
//!     … --comprobar
//!         solo leyendo: compara `GetGrades` con las del Excel.
//!     … --borrar-subido <id>
//!         **escribe**: `accio=esborrar_arxiu` del subido `<id>`, armado con
//!         los atributos de su botón «Eliminar» (es JS, no un enlace); luego
//!         compara las notas, para ver si borrar lo subido las quita.
//!
//! - Las notas de prueba son inventadas (de una lista fija, por orden), con
//!   el 0, el 10 y dos decimales; el Excel sale de `exportar`, el de la app.
//! - Por pantalla, **sin nombres**: cuántos, las formas de los enlaces y
//!   los recuentos. Lo crudo (la plantilla, el Excel, las páginas), solo en
//!   `%APPDATA%\es.gestornotas.app\clickedu\capturas\subir\`; las notas, con
//!   la hora, en `capturas\notas-h6\`. **Nunca en el repo.**

use std::collections::BTreeMap;
use std::path::Path;

use gestor_de_notas_lib::clickedu::cliente::{self, Cliente};
use gestor_de_notas_lib::clickedu::cuenta::{self, LlaveroWindows};
use gestor_de_notas_lib::clickedu::fichas;
use gestor_de_notas_lib::exportar;
use gestor_de_notas_lib::plantillas;
use gestor_de_notas_lib::sesion::Puestas;
use scraper::{Html, Selector};

/// Las notas de prueba, por orden de la plantilla.
const NOTAS: [f64; 10] = [5.0, 6.5, 7.25, 8.0, 10.0, 0.0, 3.75, 9.5, 4.0, 2.5];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let valor = |n: &str| args.iter().position(|a| a == n).and_then(|i| args.get(i + 1)).cloned();
    let (Some(materia), Some(evaluacion), Some(item)) = (valor("--materia"), valor("--evaluacion"), valor("--item"))
    else {
        panic!("uso: -- --materia M --evaluacion T --item I [--subir | --importar <id> | --comprobar | --borrar-subido <id>]");
    };
    let subir = args.iter().any(|a| a == "--subir");
    let importar = valor("--importar");
    let comprobar = args.iter().any(|a| a == "--comprobar");
    let borrar = valor("--borrar-subido");

    let carpeta_ce = dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app").join("clickedu");
    let crudo = carpeta_ce.join("capturas").join("subir");
    std::fs::create_dir_all(&crudo).expect("no se puede crear la carpeta de lo crudo");

    let guardada = cuenta::leer(&carpeta_ce, &LlaveroWindows)
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("no hay cuenta guardada: entra antes desde la pantalla Conexión");
    let archivo = guardada.archivo.as_ref().expect("falta el archivo de seguridad");
    let archivo = cuenta::leer_archivo(archivo).unwrap_or_else(|e| panic!("{e}"));
    let cliente = Cliente::new(cliente::BASE).unwrap_or_else(|e| panic!("{e}"));
    println!("Entrando…");
    bloquear(cliente.entrar(&guardada.usuario, &guardada.contrasena, (&archivo.0, &archivo.1)))
        .unwrap_or_else(|e| panic!("no se ha podido entrar: {e}"));

    let pagina_importar = format!("/assignatures/importar_notes_items.php?assig={materia}&id_aval={evaluacion}");

    // 1. La página de importar: el ítem está y el formulario es el de siempre.
    let pagina = abrir(&cliente, &pagina_importar, &crudo, "importar-antes");
    let items = fichas::items(&pagina).unwrap_or_else(|e| panic!("{e:?}"));
    println!("\nÍtems en la página de importar ({:?}):", items.estado);
    for i in &items.dato {
        println!("  {:>7}  {}{}", i.id, i.nombre, if i.id == item { "  ← el de prueba" } else { "" });
    }
    assert!(items.dato.iter().any(|i| i.id == item), "el ítem {item} no está en la página de importar");
    let form = fichas::importar(&pagina, &materia, &evaluacion).unwrap_or_else(|e| panic!("{e:?}"));
    println!("Formulario de subir: {:?} · {} · campo {}", form.estado, form.dato.accion, form.dato.campo_fichero);
    ensenar_subidos(&pagina);

    // 2. La plantilla, recién bajada, y el Excel de prueba que sale de ella.
    let bajada = bloquear(cliente.bajar(&format!(
        "/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig={materia}&id_aval={evaluacion}"
    )))
    .unwrap_or_else(|e| panic!("no se puede bajar la plantilla: {e}"));
    assert!(bajada.cuerpo.starts_with(b"PK"), "lo bajado no es un Excel");
    let ruta_plantilla = crudo.join("plantilla.xlsx");
    std::fs::write(&ruta_plantilla, &bajada.cuerpo).unwrap();
    let plantilla = plantillas::leer(&ruta_plantilla).unwrap_or_else(|e| panic!("{e}"));
    let mut puestas = Puestas::new();
    let mut esperadas = BTreeMap::new();
    for (n, a) in plantilla.alumnos.iter().enumerate() {
        let nota = NOTAS[n % NOTAS.len()];
        puestas.insert(a.clave.clone(), Some(nota));
        esperadas.insert(a.id.clone(), nota);
    }
    let exportado = exportar::exportar(&plantilla, &puestas, "prueba.xlsx", &crudo.join("exportado"))
        .unwrap_or_else(|e| panic!("{e}"));
    println!(
        "\nPlantilla: {} alumnos ({} avisos). Excel de prueba: {} con nota, en {}",
        plantilla.alumnos.len(),
        plantilla.avisos.len(),
        exportado.con_nota,
        exportado.ruta
    );

    // Las notas de antes.
    let antes = notas_del_item(&cliente, &carpeta_ce, &materia, &evaluacion, &item, "antes");
    println!("Notas del ítem {item} antes: {}", antes.len());

    if subir {
        println!("\n══ Subiendo (POST carregar_plantilla)…");
        let contenido = std::fs::read(&exportado.ruta).unwrap();
        let nombre = Path::new(&exportado.ruta).file_name().unwrap().to_string_lossy().into_owned();
        let direccion =
            format!("/assignatures/importar_notes_items.php?accio=carregar_plantilla&assig={materia}&id_aval={evaluacion}");
        // La página la trae relativa (`importar_notes_items.php?…`).
        assert!(direccion.ends_with(form.dato.accion.trim_start_matches('/')), "la dirección no es la del formulario");
        let mut campos = vec![(form.dato.campo_item.as_str(), item.as_str())];
        if let Some((n, v)) = &form.dato.boton {
            campos.push((n.as_str(), v.as_str()));
        }
        println!("Campos: {campos:?} y el fichero en `{}`", form.dato.campo_fichero);
        let r = bloquear(cliente.subir(&direccion, &campos, (&form.dato.campo_fichero, &nombre, contenido)))
        .unwrap_or_else(|e| panic!("no se ha podido subir: {e}"));
        guardar(&crudo, "subir-respuesta", &direccion, &r.texto);
        println!("Respuesta: {} ({} bytes)", forma(&r.url), r.texto.len());
        ensenar_mensajes(&r.texto);
        let despues = abrir(&cliente, &pagina_importar, &crudo, "importar-despues");
        ensenar_subidos(&despues);
        println!("\nNo se ha importado nada. Para importar: -- … --importar <id>");
    }

    if let Some(id) = &importar {
        let pagina = abrir(&cliente, &pagina_importar, &crudo, "importar-antes-de-importar");
        let enlace = enlace_de(&pagina, "importar_notes", id)
            .unwrap_or_else(|| panic!("no hay enlace `importar_notes` del subido {id} en la página"));
        println!("\n══ Importando: {}", forma(&enlace));
        let r = abrir(&cliente, &enlace, &crudo, "importar-respuesta");
        ensenar_mensajes(&r);
        ensenar_subidos(&abrir(&cliente, &pagina_importar, &crudo, "importar-tras-importar"));
    }

    if let Some(id) = &borrar {
        let pagina = abrir(&cliente, &pagina_importar, &crudo, "importar-antes-de-borrar");
        let enlace = enlace_borrar(&pagina, id)
            .unwrap_or_else(|| panic!("no hay botón `BotoEsborrarImportacio` del subido {id} en la página"));
        println!("\n══ Borrando el subido: {}", forma(&enlace));
        let r = abrir(&cliente, &enlace, &crudo, "borrar-respuesta");
        ensenar_mensajes(&r);
        ensenar_subidos(&abrir(&cliente, &pagina_importar, &crudo, "importar-tras-borrar"));
    }

    if importar.is_some() || borrar.is_some() || comprobar {
        let ahora = notas_del_item(&cliente, &carpeta_ce, &materia, &evaluacion, &item, "despues");
        comparar(&esperadas, &ahora);
    }
    println!("\nLo crudo, en {}", crudo.display());
}

/// Las filas de `GetGrades` del ítem, guardadas con la hora.
fn notas_del_item(
    cliente: &Cliente,
    carpeta_ce: &Path,
    materia: &str,
    evaluacion: &str,
    item: &str,
    cuando: &str,
) -> Vec<fichas::Puesta> {
    let json = bloquear(cliente.abrir(&format!(
        "/go/Index.php?ctr=Evaluate/EvaluateTermly&do=GetGrades&id={materia}&trm={evaluacion}"
    )))
    .unwrap_or_else(|e| panic!("{e}"))
    .texto;
    let (a, m, d) = gestor_de_notas_lib::registro::fecha(gestor_de_notas_lib::registro::ahora());
    let hora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let carpeta = carpeta_ce.join("capturas").join("notas-h6");
    std::fs::create_dir_all(&carpeta).unwrap();
    std::fs::write(carpeta.join(format!("{a:04}-{m:02}-{d:02}-{hora}-{materia}-{evaluacion}-{cuando}.json")), &json)
        .unwrap();
    let todas = fichas::notas(&json).unwrap_or_else(|e| panic!("{e:?}")).dato;
    println!("GetGrades ({cuando}): {} filas en total", todas.len());
    todas.into_iter().filter(|p| p.criterio == item).collect()
}

/// Lo esperado frente a lo puesto, por alumno: en `nota` y en `nota_mod`.
fn comparar(esperadas: &BTreeMap<String, f64>, puestas: &[fichas::Puesta]) {
    let numero = |s: &Option<String>| s.as_deref().and_then(|t| t.trim().replace(',', ".").parse::<f64>().ok());
    let (mut en_nota, mut en_mod, mut distintas, mut faltan) = (0, 0, 0, 0);
    let mut tipos = BTreeMap::new();
    for (alumno, &esperada) in esperadas {
        let Some(p) = puestas.iter().find(|p| &p.alumno == alumno) else {
            faltan += 1;
            continue;
        };
        *tipos.entry(p.tipo).or_insert(0) += 1;
        let igual = |v: Option<f64>| v.is_some_and(|v| (v - esperada).abs() < 1e-6);
        let (n, m) = (igual(numero(&p.nota)), igual(numero(&p.nota_mod)));
        en_nota += n as usize;
        en_mod += m as usize;
        if !n && !m {
            distintas += 1;
        }
    }
    let ajenas = puestas.iter().filter(|p| !esperadas.contains_key(&p.alumno)).count();
    println!("\n══ Comparación ({} esperadas, {} filas del ítem)", esperadas.len(), puestas.len());
    println!("  iguales en `nota`: {en_nota} · en `nota_mod`: {en_mod}");
    println!("  distintas: {distintas} · faltan: {faltan} · de alumnos que no están en la plantilla: {ajenas}");
    println!("  `type` de las filas: {tipos:?}");
}

/// Los enlaces con `accio=` de la página (lo subido y lo que se hace con
/// ello), con los números tapados salvo el `id`, y los textos de las filas
/// de la tabla de subidos, sin nombres de alumnos (son nombres de fichero).
fn ensenar_subidos(html: &str) {
    let doc = Html::parse_document(html);
    let mut enlaces = Vec::new();
    for a in doc.select(&sel("a[href], [onclick]")) {
        for attr in ["href", "onclick"] {
            if let Some(v) = a.value().attr(attr) {
                if v.contains("accio=") && !v.contains("descarregar_plantilla") {
                    enlaces.push(format!("{attr}: {} (id={})", forma(v), parametro(v, "id").unwrap_or_default()));
                }
            }
        }
    }
    println!("\nEn la página de importar, {} enlaces con `accio=`:", enlaces.len());
    for e in &enlaces {
        println!("  {e}");
    }
    let filas: Vec<String> = doc
        .select(&sel("table tr"))
        .map(|f| f.text().collect::<Vec<_>>().join(" ").split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|t| t.contains(".xls") || t.to_lowercase().contains("import"))
        .collect();
    for f in filas {
        println!("  fila: {f}");
    }
}

/// Los avisos de la página (`alert`, mensajes): solo su forma y su texto
/// corto, para ver si Clickedu se queja.
fn ensenar_mensajes(html: &str) {
    let doc = Html::parse_document(html);
    for e in doc.select(&sel(".alert, .error, .missatge, .message, .msg, .avis, .notice, .success")) {
        let t = e.text().collect::<Vec<_>>().join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
        if !t.is_empty() && t.len() < 300 {
            println!("  aviso: {t}");
        }
    }
    for script in doc.select(&sel("script")) {
        for l in script.text().collect::<String>().lines() {
            let t = l.trim();
            if t.contains("alert(") && t.len() < 300 {
                println!("  alert: {t}");
            }
        }
    }
}

/// El enlace de la página con `accio=<accion>` e `id=<id>`, tal como viene.
fn enlace_de(html: &str, accion: &str, id: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let h = doc
        .select(&sel("a[href]"))
        .filter_map(|a| a.value().attr("href"))
        .map(|h| h.replace("&amp;", "&"))
        .find(|h| h.contains(&format!("accio={accion}")) && parametro(h, "id").as_deref() == Some(id))?;
    Some(if h.starts_with('/') { h } else { format!("/assignatures/{}", h.trim_start_matches("./")) })
}

/// «Eliminar» no es un enlace: es `<a class="BotoEsborrarImportacio"
/// cgap="‹materia›" aval="‹evaluación›" ind="‹id›">`, y el JS de la página,
/// tras un `confirm`, va a `importar_notes_items.php?accio=esborrar_arxiu&assig=‹cgap›&id_aval=‹aval›&id=‹ind›`
/// (un `GET`). Se arma con lo que trae el botón.
fn enlace_borrar(html: &str, id: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let b = doc.select(&sel("a.BotoEsborrarImportacio")).find(|a| a.value().attr("ind") == Some(id))?;
    let (cgap, aval) = (b.value().attr("cgap")?, b.value().attr("aval")?);
    Some(format!("/assignatures/importar_notes_items.php?accio=esborrar_arxiu&assig={cgap}&id_aval={aval}&id={id}"))
}

fn parametro(h: &str, nombre: &str) -> Option<String> {
    let consulta = h.replace("&amp;", "&");
    let consulta = consulta.split_once('?').map(|(_, q)| q).unwrap_or(&consulta).to_string();
    consulta.split(['&', '\'', '"', ')']).find_map(|p| p.strip_prefix(&format!("{nombre}="))).map(str::to_string)
}

fn abrir(cliente: &Cliente, direccion: &str, crudo: &Path, nombre: &str) -> String {
    let a = bloquear(cliente.abrir(direccion)).unwrap_or_else(|e| panic!("no se puede abrir {direccion}: {e}"));
    guardar(crudo, nombre, direccion, &a.texto);
    a.texto
}

fn guardar(crudo: &Path, nombre: &str, direccion: &str, texto: &str) {
    std::fs::write(crudo.join(format!("{nombre}.html")), format!("<!-- {direccion} -->\n{texto}"))
        .expect("no se puede guardar lo crudo");
}

/// La dirección con los números tapados.
fn forma(h: &str) -> String {
    let mut s = String::new();
    let mut en_numero = false;
    for c in h.replace("&amp;", "&").chars() {
        if c.is_ascii_digit() {
            if !en_numero {
                s.push('▢');
            }
            en_numero = true;
        } else {
            en_numero = false;
            s.push(c);
        }
    }
    s
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).unwrap()
}

fn bloquear<F: std::future::Future>(f: F) -> F::Output {
    tauri::async_runtime::block_on(f)
}
