//! H6, antes del primer `POST`: mirar **solo leyendo** cómo se da de alta un
//! ítem en Clickedu, para crear el de prueba (ponderación 0, sin que lo vean
//! las familias) en la materia de la primera subida.
//!
//!     cargo run --example mirar_alta_item                  # las materias, con su id
//!     cargo run --example mirar_alta_item -- --materia <id>
//!
//! - **Solo `GET`**, y solo de tres páginas: la de Ítems de esa materia, el
//!   formulario de «Nuevo ítem» (`Subject/Item/ItemDefault&do=New`, el
//!   enlace que trae la propia página) y el de editar un ítem que ya hay
//!   (`Subject/Item/Item&do=Edit`), para ver cómo queda uno relleno. Ningún
//!   formulario se manda.
//! - Enseña los formularios (a dónde van, con qué método y cada campo con
//!   su valor por defecto y su etiqueta) y los trozos del JS que mandan
//!   algo. Son páginas del profesor, sin alumnos.
//! - Lo crudo, solo en local, en
//!   `%APPDATA%\es.gestornotas.app\clickedu\capturas\alta-item\`; **nunca
//!   en el repo**.

use std::path::Path;

use gestor_de_notas_lib::clickedu::cliente::{self, Cliente};
use gestor_de_notas_lib::clickedu::cuenta::{self, LlaveroWindows};
use gestor_de_notas_lib::clickedu::fichas;
use scraper::{ElementRef, Html, Selector};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let materia = args.iter().position(|a| a == "--materia").and_then(|i| args.get(i + 1)).cloned();

    let carpeta_ce = dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app").join("clickedu");
    let crudo = carpeta_ce.join("capturas").join("alta-item");
    let _ = std::fs::remove_dir_all(&crudo);
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

    let Some(materia) = materia else {
        let pagina = abrir(&cliente, cliente::ASIGNATURAS, &crudo, "materias");
        let leidas = fichas::materias(&pagina).unwrap_or_else(|e| panic!("{e:?}"));
        println!("\nLas materias (elige con -- --materia <id>):");
        for m in &leidas.dato {
            println!("  {:>8}  {} · {}", m.id, m.nombre, m.grupo);
        }
        return;
    };

    // La página de Ítems.
    let items = abrir(&cliente, &format!("/go/Index.php?ctr=Subject/Items&do=Index&id={materia}"), &crudo, "items");
    let doc = Html::parse_document(&items);
    if let Ok(ev) = fichas::evaluaciones(&items) {
        println!("\nEvaluaciones:");
        for e in &ev.dato {
            println!("  {:>6}  {}{}", e.id, e.nombre, if e.elegida { "  (la elegida)" } else { "" });
        }
    }
    let enlaces: Vec<String> = doc
        .select(&sel("a[href]"))
        .filter_map(|a| a.value().attr("href"))
        .map(|h| h.trim_start_matches("..").to_string())
        .collect();
    println!("\nEnlaces de ítems en la página:");
    let mut vistos = std::collections::BTreeSet::new();
    for h in enlaces.iter().filter(|h| h.contains("ctr=Subject/Item/")) {
        let forma = forma(h);
        if vistos.insert(forma.clone()) {
            println!("  {forma}");
        }
    }

    // Los ítems de la tabla: ponderación y si lo ven las familias.
    println!("\nÍtems de la evaluación elegida (it · ponderación · familias · nombre):");
    for fila in doc.select(&sel("tr")) {
        let Some(pct) = fila.select(&sel("input.InputPercent")).next() else { continue };
        let it = pct.value().attr("attr-item").unwrap_or("");
        if it == "0" || pct.value().attr("attr-type") != Some("0") {
            continue;
        }
        let peso = pct.value().attr("value").filter(|v| !v.is_empty()).unwrap_or("(vacía)");
        let visible = fila
            .select(&sel(r#"input[name="ind_visible_pm"]"#))
            .next()
            .map(|c| if c.value().attr("checked").is_some() { "la ven" } else { "no" })
            .unwrap_or("?");
        let nombre = fila.select(&sel("td")).map(texto).filter(|t| !t.is_empty()).take(2).collect::<Vec<_>>().join(" · ");
        println!("  {it:>7} · {peso:>7} · {visible:<6} · {nombre}");
    }

    // El formulario de «Nuevo ítem».
    let nuevo = enlaces
        .iter()
        .find(|h| h.contains("ctr=Subject/Item/ItemDefault&do=New") && h.contains(&format!("id={materia}")))
        .expect("no encuentro el enlace de «Nuevo ítem» en la página de Ítems");
    let alta = abrir(&cliente, nuevo, &crudo, "nuevo-item");
    println!("\n══ Nuevo ítem: {}", forma(nuevo));
    ensenar_formularios(&alta);

    // Un ítem que ya hay, para verlo relleno.
    if let Some(editar) = enlaces.iter().find(|h| h.contains("ctr=Subject/Item/Item&do=Edit")) {
        let pagina = abrir(&cliente, editar, &crudo, "editar-item");
        println!("\n══ Editar un ítem que ya hay: {}", forma(editar));
        ensenar_formularios(&pagina);
    } else {
        println!("\n(No hay ningún ítem que editar en la evaluación elegida.)");
    }
    // Las notas de la evaluación elegida, para comparar antes y después (no
    // se borran al volver a pasar: van con la hora).
    if let Some(trm) = fichas::evaluaciones(&items).ok().and_then(|e| e.dato.into_iter().find(|e| e.elegida)) {
        let json = bloquear(cliente.abrir(&format!("/go/Index.php?ctr=Evaluate/EvaluateTermly&do=GetGrades&id={materia}&trm={}", trm.id)))
            .unwrap_or_else(|e| panic!("{e}"))
            .texto;
        let (a, m, d) = gestor_de_notas_lib::registro::fecha(gestor_de_notas_lib::registro::ahora());
        let hora = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let notas = carpeta_ce.join("capturas").join("notas-h6");
        std::fs::create_dir_all(&notas).unwrap();
        let ruta = notas.join(format!("{a:04}-{m:02}-{d:02}-{hora}-{materia}-{}.json", trm.id));
        std::fs::write(&ruta, &json).unwrap();
        let puestas = fichas::notas(&json).map(|l| l.dato).unwrap_or_default();
        println!("\nNotas de {}: {} filas, guardadas en {}", trm.nombre, puestas.len(), ruta.display());
    }
    println!("\nLo crudo, en {}", crudo.display());
}

fn abrir(cliente: &Cliente, direccion: &str, crudo: &Path, nombre: &str) -> String {
    let a = bloquear(cliente.abrir(direccion)).unwrap_or_else(|e| panic!("no se puede abrir {direccion}: {e}"));
    std::fs::write(crudo.join(format!("{nombre}.html")), format!("<!-- {direccion} -->\n{}", a.texto))
        .expect("no se puede guardar lo crudo");
    a.texto
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

fn ensenar_formularios(html: &str) {
    let doc = Html::parse_document(html);
    let titulo = doc.select(&sel("h1, h2, h3, .page-title")).map(texto).find(|t| !t.is_empty());
    if let Some(t) = titulo {
        println!("Título: {t}");
    }
    for (i, form) in doc.select(&sel("form")).enumerate() {
        let a = form.value();
        let accion = a.attr("action").unwrap_or("(la misma página)");
        if accion.contains("form_idioma") || a.attr("name") == Some("form_idioma") {
            continue;
        }
        println!(
            "\nFormulario {} · {} {} · id={} name={} enctype={}",
            i + 1,
            a.attr("method").unwrap_or("GET").to_uppercase(),
            forma(accion),
            a.attr("id").unwrap_or("-"),
            a.attr("name").unwrap_or("-"),
            a.attr("enctype").unwrap_or("-")
        );
        for campo in form.select(&sel("input, select, textarea, button")) {
            let v = campo.value();
            let nombre = v.attr("name").unwrap_or("");
            let tipo = match v.name() {
                "input" => v.attr("type").unwrap_or("text").to_string(),
                otro => otro.to_string(),
            };
            let mut linea = format!("  {tipo:<9} {nombre:<32}");
            match v.name() {
                "select" => {
                    let opciones: Vec<String> = campo
                        .select(&sel("option"))
                        .map(|o| {
                            let marca = if o.value().attr("selected").is_some() { "*" } else { "" };
                            format!("{marca}{}={}", o.value().attr("value").unwrap_or(""), texto(o))
                        })
                        .collect();
                    linea.push_str(&format!(" [{}]", opciones.join(" | ")));
                }
                "textarea" => linea.push_str(&format!(" «{}»", texto(campo))),
                "button" => linea.push_str(&format!(" «{}»", texto(campo))),
                _ => {
                    if let Some(val) = v.attr("value") {
                        linea.push_str(&format!(" = «{val}»"));
                    }
                    if v.attr("checked").is_some() {
                        linea.push_str(" (marcado)");
                    }
                }
            }
            for extra in ["id", "required", "disabled", "readonly", "min", "max", "step"] {
                if let Some(x) = v.attr(extra) {
                    linea.push_str(&format!(" {extra}={x}"));
                }
            }
            if let Some(etiqueta) = etiqueta(&doc, campo) {
                linea.push_str(&format!("  ← «{etiqueta}»"));
            }
            println!("{linea}");
        }
    }
    // El JS que manda algo.
    println!("\nJS que manda algo:");
    for script in doc.select(&sel("script")) {
        let js = script.text().collect::<String>();
        for (n, l) in js.lines().enumerate() {
            let t = l.trim();
            let manda = ["do=", "$.post", "$.ajax", ".submit(", "action", "url:", "Save", "Insert"];
            if manda.iter().any(|m| t.contains(m)) && t.len() < 300 {
                println!("  {n:>4}: {t}");
            }
        }
    }
}

/// La etiqueta del campo: la `<label for>` o la `<label>` que lo envuelve, o
/// la de su `.form-group`.
fn etiqueta(doc: &Html, campo: ElementRef) -> Option<String> {
    if let Some(id) = campo.value().attr("id") {
        let s = Selector::parse(&format!(r#"label[for="{id}"]"#)).ok()?;
        if let Some(l) = doc.select(&s).next() {
            return Some(texto(l)).filter(|t| !t.is_empty());
        }
    }
    let mut padre = campo.parent();
    for _ in 0..4 {
        let p = padre?;
        if let Some(e) = ElementRef::wrap(p) {
            if e.value().name() == "label" {
                return Some(texto(e)).filter(|t| !t.is_empty());
            }
            if let Some(l) = e.select(&sel("label")).next() {
                return Some(texto(l)).filter(|t| !t.is_empty());
            }
        }
        padre = p.parent();
    }
    None
}

fn texto(e: ElementRef) -> String {
    e.text().collect::<Vec<_>>().join(" ").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sel(s: &str) -> Selector {
    Selector::parse(s).unwrap()
}

fn bloquear<F: std::future::Future>(f: F) -> F::Output {
    tauri::async_runtime::block_on(f)
}
