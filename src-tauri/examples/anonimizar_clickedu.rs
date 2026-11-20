//! Las páginas de Clickedu de los tests (H4), **sin nombres**: saca de una
//! captura del explorador (`explorar_clickedu`) las que leen las fichas y
//! las escribe en `src-tauri/tests/datos/clickedu/`, que va al repo
//! (**público**).
//!
//!     cargo run --example anonimizar_clickedu -- <carpeta de la captura> [<salida>]
//!
//! La carpeta es la de `explorar_clickedu` (`paginas\` con `indice.txt`, y
//! `descargas\`), la de `exploracion\` o una de `clickedu\capturas\<fecha>\`.
//!
//! **Sin nombres por construcción**, como el informe del explorador: lo que
//! sale es una página nueva hecha solo con lo que pasa una lista blanca.
//!
//! - Fuera los scripts, los estilos, los comentarios, las imágenes y todo
//!   atributo que no sea de estructura ([`ATRIBUTOS`]; nada de `title`,
//!   `alt`, `onclick`, `data-…`).
//! - Los textos, con [`Tapar`]: solo los rótulos de la interfaz, las
//!   palabras cortas y los números de hasta cuatro cifras (fechas, notas); lo
//!   demás, `▢`.
//! - **Los números de tres cifras o más** de las direcciones, los nombres de
//!   campo y los valores (los ids de materias, evaluaciones, ítems y
//!   alumnos) se cambian por otros inventados, **siempre el mismo por el
//!   mismo** en todas las páginas: los cruces (el `id` de la materia también
//!   como `assig`, el alumno de `evalObject_` y de `GetGrades`) siguen
//!   valiendo.
//! - En el JSON, las notas se inventan; los textos, como arriba.
//!
//! **Y al final se comprueba**: si en lo que sale queda una palabra de un
//! nombre de las plantillas de la carpeta, de las bajadas en la captura
//! (`descargas\`) o del usuario, no se escribe nada (y no se dice cuál).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use gestor_de_notas_lib::ajustes;
use gestor_de_notas_lib::clickedu::cliente::decodificar;
use gestor_de_notas_lib::clickedu::cuenta::{self, LlaveroWindows};
use gestor_de_notas_lib::clickedu::tapar::{trozos, Tapar, PARAMETROS_DE_CODIGO, PARAMETROS_DE_VERBO};
use gestor_de_notas_lib::plantillas;
use reqwest::Url;
use scraper::node::Node;
use scraper::{ElementRef, Html};

/// Las páginas que se sacan: el nombre del fichero y cómo se reconoce su
/// dirección. De cada una, las primeras `cuantas`.
const PAGINAS: &[(&str, &[&str], usize)] = &[
    ("materias", &["ctr=Subjects/Subjects"], 1),
    ("items", &["ctr=Subject/Items&do=Index", "trm="], 1),
    ("importar", &["importar_notes_items.php?assig="], 1),
    ("notas", &["do=GetGrades"], 2),
];

/// Los elementos que no se copian, con todo lo que llevan dentro.
const FUERA: &[&str] = &["script", "style", "noscript", "svg", "iframe", "img", "link", "meta", "object", "embed"];

/// Los atributos que se copian (los demás, no).
const ATRIBUTOS: &[&str] = &[
    "name", "class", "id", "type", "method", "action", "href", "value", "selected", "checked", "for", "enctype",
    "multiple", "disabled", "readonly",
];

/// Los elementos sin cierre.
/// Elementos (por `id`) cuyo texto se tapa entero, rótulos incluidos: el
/// nombre del profesor de la cabecera, que si no deja la inicial («Mª»).
const SIN_TEXTO: &[&str] = &["nomUsuari"];
const VACIOS: &[&str] = &["input", "br", "hr", "col", "area", "base", "wbr", "source", "track", "param"];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(captura) = args.first().map(PathBuf::from) else {
        eprintln!("Uso: cargo run --example anonimizar_clickedu -- <carpeta de la captura> [<salida>]");
        std::process::exit(2);
    };
    let salida = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("datos").join("clickedu"));

    let app = dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app");
    let mut nombres = nombres_de_las_plantillas(&app);
    nombres.extend(nombres_de_las_descargas(&captura.join("descargas")));
    let mut tapar = Tapar::new(&nombres);
    if let Ok(Some(g)) = cuenta::leer(&app.join("clickedu"), &LlaveroWindows) {
        tapar.anadir(&g.usuario);
    }
    println!("{} palabras de nombres, siempre tapadas.", tapar.cuantos());
    if tapar.cuantos() == 0 {
        panic!("no hay nombres con que comprobar: sin las plantillas, no se anonimiza");
    }

    let indice = std::fs::read_to_string(captura.join("paginas").join("indice.txt"))
        .unwrap_or_else(|e| panic!("no se puede leer paginas\\indice.txt de la captura: {e}"));
    let mut a = Anonimizar::new(tapar);
    let mut hechas: Vec<(String, String)> = Vec::new();
    for (nombre, trozos_de_la_direccion, cuantas) in PAGINAS {
        let elegidas: Vec<(&str, &str)> = indice
            .lines()
            .filter_map(|l| {
                let mut c = l.split('\t');
                Some((c.next()?, c.next()?))
            })
            .filter(|(_, d)| trozos_de_la_direccion.iter().all(|t| d.contains(t)))
            .filter(|(_, d)| *nombre != "importar" || !d.contains("accio="))
            .take(*cuantas)
            .collect();
        if elegidas.is_empty() {
            println!("  {nombre}: no está en la captura");
        }
        for (i, (n, direccion)) in elegidas.into_iter().enumerate() {
            let cuerpo = std::fs::read(captura.join("paginas").join(format!("{n}.html"))).expect("página del índice");
            let texto = decodificar(&cuerpo);
            let (fichero, hecho) = match serde_json::from_str::<serde_json::Value>(&texto) {
                Ok(json) => {
                    let mut json = json;
                    a.json(&mut json, "");
                    (format!("{nombre}-{}.json", i + 1), serde_json::to_string_pretty(&json).expect("JSON") + "\n")
                }
                Err(_) => {
                    let base = Url::parse(direccion).expect("dirección del índice");
                    let fichero =
                        if *cuantas == 1 { format!("{nombre}.html") } else { format!("{nombre}-{}.html", i + 1) };
                    // La dirección, en un comentario tras el DOCTYPE: de qué página es.
                    let comentario = format!("<!DOCTYPE html>\n<!-- {} -->\n", a.url(base.as_str(), &base));
                    (fichero, a.pagina(&texto, &base).replacen("<!DOCTYPE html>\n", &comentario, 1))
                }
            };
            println!("  {fichero}: {} bytes (de {})", hecho.len(), cuerpo.len());
            hechas.push((fichero, hecho));
        }
    }

    // La comprobación: ninguna palabra de un nombre en lo que se lee (los
    // textos, los valores de los atributos, el JSON). Las etiquetas y los
    // nombres de atributo son de HTML: una alumna «Li» no está en `<li>`.
    // Lo que se encuentre no se enseña (sería el nombre): solo cuántas.
    let mut con_nombres = 0;
    for (fichero, hecho) in &hechas {
        let n: usize = lo_que_se_lee(fichero, hecho)
            .iter()
            .map(|t| trozos(t).into_iter().filter(|(p, l)| *l && a.tapar.es_nombre(p)).count())
            .sum();
        if n > 0 {
            eprintln!("  ✗ {fichero}: {n} palabras de nombres en textos o valores");
            con_nombres += n;
        }
        let en_html = trozos(hecho).into_iter().filter(|(p, l)| *l && a.tapar.es_nombre(p)).count();
        if en_html > n {
            println!(
                "  {fichero}: {} palabras de HTML que se escriben como un nombre (etiquetas, atributos)",
                en_html - n
            );
        }
    }
    if con_nombres > 0 {
        eprintln!("No se escribe nada: quedan nombres. Hay que tapar más.");
        std::process::exit(1);
    }

    std::fs::create_dir_all(&salida).expect("no se puede crear la carpeta de salida");
    for (fichero, hecho) in &hechas {
        std::fs::write(salida.join(fichero), hecho).expect("no se puede escribir");
    }
    println!("{} ficheros en {}. Míralos antes de subirlos: van a un repo público.", hechas.len(), salida.display());
    println!("{} números cambiados por otros inventados.", a.numeros.len());
}

/// Lo que se lee de lo que sale: de una página, sus textos y los valores de
/// sus atributos (también el comentario con la dirección); de un JSON, todo.
fn lo_que_se_lee(fichero: &str, hecho: &str) -> Vec<String> {
    if fichero.ends_with(".json") {
        return vec![hecho.to_string()];
    }
    let doc = Html::parse_document(hecho);
    let mut res: Vec<String> = Vec::new();
    for nodo in doc.tree.root().descendants() {
        match nodo.value() {
            Node::Text(t) => res.push(t.to_string()),
            Node::Comment(c) => res.push(c.to_string()),
            Node::Element(e) => res.extend(e.attrs().map(|(_, v)| v.to_string())),
            _ => {}
        }
    }
    res
}

fn nombres_de_las_plantillas(app: &Path) -> Vec<String> {
    let Some(documentos) = dirs::document_dir() else { return Vec::new() };
    let Ok(a) = ajustes::leer(&app.join("ajustes.json")) else { return Vec::new() };
    let carpeta = PathBuf::from(ajustes::carpeta_plantillas(&a, &documentos).ruta);
    nombres_de_la_carpeta(&carpeta, plantillas::es_plantilla)
}

fn nombres_de_las_descargas(carpeta: &Path) -> Vec<String> {
    nombres_de_la_carpeta(carpeta, |n| n.ends_with(".xlsx") || n.ends_with(".xls"))
}

fn nombres_de_la_carpeta(carpeta: &Path, es: impl Fn(&str) -> bool) -> Vec<String> {
    let Ok(ficheros) = std::fs::read_dir(carpeta) else { return Vec::new() };
    ficheros
        .filter_map(|f| f.ok().map(|f| f.path()))
        .filter(|p| es(&p.file_name().unwrap_or_default().to_string_lossy()))
        .filter_map(|p| plantillas::leer(&p).ok())
        .flat_map(|p| p.alumnos.into_iter().map(|a| a.nombre))
        .collect()
}

struct Anonimizar {
    tapar: Tapar,
    /// Número de verdad → inventado, con las mismas cifras.
    numeros: HashMap<String, String>,
    inventados: HashSet<String>,
    /// Cuántos inventados van de cada longitud.
    siguientes: BTreeMap<usize, u64>,
    /// Para inventar notas.
    notas: u64,
}

impl Anonimizar {
    fn new(tapar: Tapar) -> Self {
        Anonimizar { tapar, numeros: HashMap::new(), inventados: HashSet::new(), siguientes: BTreeMap::new(), notas: 0 }
    }

    /// Un número de más de 100 (un id), cambiado por uno inventado: siempre
    /// el mismo para el mismo, y nunca dos iguales. Hasta 100 se quedan: son
    /// tipos, porcentajes, notas.
    fn numero(&mut self, n: &str) -> String {
        if n.parse::<u64>().map_or(true, |v| v <= 100) {
            return n.to_string();
        }
        if let Some(m) = self.numeros.get(n) {
            return m.clone();
        }
        let siguiente = self.siguientes.entry(n.len()).or_insert(0);
        let inventado = loop {
            *siguiente += 1;
            // 9…, con las mismas cifras (si caben).
            let i = format!("9{:0>width$}", *siguiente, width = n.len() - 1);
            if !self.inventados.contains(&i) {
                break i;
            }
        };
        self.inventados.insert(inventado.clone());
        self.numeros.insert(n.to_string(), inventado.clone());
        inventado
    }

    /// Un texto: con [`Tapar`] y, además, sin números de tres cifras o más
    /// que no sean un año (los teléfonos del pie, `948 …`).
    fn texto(&self, t: &str) -> String {
        let tapado = self.tapar.texto_entero(t);
        trozos(&tapado)
            .into_iter()
            .map(|(p, letras)| {
                let largo = !letras && p.len() >= 3 && p.chars().all(|c| c.is_ascii_digit());
                let ano = p.len() == 4 && (p.starts_with("19") || p.starts_with("20"));
                if largo && !ano && p.parse::<u64>().map_or(true, |v| v > 100) {
                    "▢"
                } else {
                    p
                }
            })
            .collect()
    }

    /// Un trozo de código (un nombre de campo, una clase): las palabras se
    /// quedan, salvo un nombre (`x`); los números, inventados.
    fn codigo(&mut self, codigo: &str) -> String {
        let mut res = String::new();
        for (t, letras) in trozos(codigo) {
            if letras {
                res.push_str(if self.tapar.es_nombre(t) { "x" } else { t });
            } else if t.chars().all(|c| c.is_ascii_digit()) {
                res.push_str(&self.numero(t));
            } else if t.chars().any(char::is_alphanumeric) {
                res.push('x');
            } else {
                res.push_str(t);
            }
        }
        res
    }

    /// Un valor: si es un número, inventado; si no, como texto.
    fn valor(&mut self, v: &str) -> String {
        if !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()) {
            self.numero(v)
        } else {
            self.texto(v)
        }
    }

    /// Una dirección, relativa a la página: `/ruta?…`, con la ruta como
    /// código y cada valor según su parámetro. Lo de fuera de Clickedu y el
    /// JS, `#`.
    fn url(&mut self, crudo: &str, base: &Url) -> String {
        let crudo = crudo.trim();
        if crudo.to_lowercase().starts_with("javascript:") || crudo.starts_with('#') {
            return "#".into();
        }
        let Ok(u) = base.join(crudo) else { return "#".into() };
        if u.host_str() != base.host_str() {
            return "#".into();
        }
        let mut res = self.codigo(u.path());
        let pares: Vec<(String, String)> = u.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
        for (i, (k, v)) in pares.iter().enumerate() {
            res.push(if i == 0 { '?' } else { '&' });
            res.push_str(&self.codigo(k));
            res.push('=');
            let kl = k.to_lowercase();
            let v = if PARAMETROS_DE_VERBO.contains(&kl.as_str()) || PARAMETROS_DE_CODIGO.contains(&kl.as_str()) {
                self.codigo(v)
            } else {
                self.valor(v)
            };
            res.push_str(&v);
        }
        res
    }

    fn pagina(&mut self, html: &str, base: &Url) -> String {
        let doc = Html::parse_document(html);
        let mut res = String::from("<!DOCTYPE html>\n");
        self.elemento(doc.root_element(), base, &mut res);
        res.push('\n');
        res
    }

    fn elemento(&mut self, e: ElementRef, base: &Url, res: &mut String) {
        let nombre = e.value().name();
        if FUERA.contains(&nombre) {
            return;
        }
        let _ = write!(res, "<{nombre}");
        for (k, v) in e.value().attrs() {
            if !ATRIBUTOS.contains(&k) {
                continue;
            }
            let v = match k {
                "href" | "action" => self.url(v, base),
                "name" | "class" | "id" | "for" => self.codigo(v),
                "value" => self.valor(v),
                // `type`, `method`, `selected`…: palabras sueltas.
                _ if v.chars().all(|c| c.is_ascii_alphabetic() || "-/".contains(c)) => v.to_string(),
                _ => continue,
            };
            let _ = write!(res, " {k}=\"{}\"", escapar(&v));
        }
        res.push('>');
        if VACIOS.contains(&nombre) {
            return;
        }
        if e.value().id().is_some_and(|id| SIN_TEXTO.contains(&id)) {
            let _ = write!(res, "▢</{nombre}>");
            return;
        }
        for hijo in e.children() {
            if let Some(h) = ElementRef::wrap(hijo) {
                self.elemento(h, base, res);
            } else if let Node::Text(t) = hijo.value() {
                let t: &str = t;
                if t.trim().is_empty() {
                    res.push_str(if t.contains('\n') {
                        "\n"
                    } else if t.is_empty() {
                        ""
                    } else {
                        " "
                    });
                } else {
                    res.push_str(&escapar(&self.texto(t)));
                }
            }
        }
        let _ = write!(res, "</{nombre}>");
    }

    /// Un JSON: los nombres de campo se quedan (son de Clickedu); los ids,
    /// inventados; las notas, inventadas; los textos, tapados.
    fn json(&mut self, v: &mut serde_json::Value, campo: &str) {
        use serde_json::Value;
        let es_id = campo.starts_with("id");
        match v {
            Value::Array(a) => a.iter_mut().for_each(|x| self.json(x, campo)),
            Value::Object(o) => {
                let viejo = std::mem::take(o);
                for (k, mut x) in viejo {
                    let k = if k.chars().all(|c| c.is_ascii_digit()) { self.numero(&k) } else { self.codigo(&k) };
                    self.json(&mut x, &k);
                    o.insert(k, x);
                }
            }
            Value::Number(n) if es_id => {
                *v = self.numero(&n.to_string()).parse::<u64>().map(Value::from).unwrap_or(Value::Null)
            }
            Value::String(s) if es_id && s.chars().all(|c| c.is_ascii_digit()) => *s = self.numero(s),
            Value::Number(_) if campo.starts_with("nota") => *v = Value::from(self.nota()),
            Value::String(s) if campo.starts_with("nota") && !s.is_empty() => *s = format!("{}", self.nota()),
            Value::String(s) => *s = self.texto(s),
            _ => {}
        }
    }

    /// Una nota inventada, de 0 a 10 con un decimal.
    fn nota(&mut self) -> f64 {
        self.notas += 1;
        ((self.notas * 37) % 101) as f64 / 10.0
    }
}

fn escapar(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://colegio.clickedu.eu/go/Index.php?ctr=Subject/Items&do=Index&id=1234").unwrap()
    }

    #[test]
    fn una_pagina_sin_nombres() {
        let mut a = Anonimizar::new(Tapar::new(["GARCÍA LÓPEZ, Lucía"]));
        let html = r#"<html><head><title>Ítems - Matemáticas</title><script>var alumno = "Lucía";</script></head>
            <body><!-- Lucía --><h1 title="Lucía García">Hola, Pablo Sanz</h1>
            <select name="trm"><option value="331" selected>1ª evaluación (de 04/09/2026 a 20/11/2026)</option></select>
            <input class="InputGrades" name="evalObject_123456_4321_1" value="7,5" onclick="x()">
            <a href="/go/Index.php?ctr=Subject/Item/Item&amp;do=Edit&amp;id=1234&amp;it=4321&amp;nom=Lucia">Examen</a>
            <a href="javascript:obrir('Lucía')">x</a><a href="https://otro.eu/a">fuera</a>
            <a href="../assignatures/importar_notes_items.php?assig=1234&amp;id_aval=331">Importar notas</a>
            <input type="number" name="percent_parent_0" value="100"><p>Tel. 948 111 222 · 2026-2027 · 100 %</p>
            </body></html>"#;
        let p = a.pagina(html, &base());
        for nada in
            ["Lucía", "García", "Pablo", "Sanz", "Lucia", "123456", "4321", "331", "1234", "onclick", "title="]
        {
            assert!(!p.contains(nada), "{nada} en {p}");
        }
        assert!(p.contains("<title>Ítems - Matemáticas</title>"), "{p}");
        assert!(p.contains("1ª evaluación (de 04/09/2026 a 20/11/2026)"), "{p}");
        // Los inventados, por orden de aparición y con las mismas cifras: 331
        // → 901, 123456 → 900001, 4321 → 9001, 1234 → 9002. El 1, igual.
        assert!(p.contains(r#"value="901">1ª evaluación"#), "{p}");
        assert!(p.contains(r#"name="evalObject_900001_9001_1""#), "{p}");
        // El mismo id, el mismo inventado en todas partes.
        assert!(p.contains("&amp;id=9002&amp;it=9001&amp;nom=▢"), "{p}");
        assert!(p.contains("/assignatures/importar_notes_items.php?assig=9002&amp;id_aval=901"), "{p}");
        assert!(p.contains(r##"<a href="#">x</a><a href="#">▢</a>"##), "{p}");
        assert!(!p.contains("<script"), "{p}");
        // Hasta 100 no es un id; los teléfonos, fuera; los años, no.
        assert!(p.contains(r#"value="100""#), "{p}");
        assert!(p.contains("<p>▢. ▢ ▢ ▢ · 2026-2027 · 100 %</p>"), "{p}");
    }

    #[test]
    fn un_json_sin_nombres() {
        let mut a = Anonimizar::new(Tapar::new(["Lucía García"]));
        let mut j: serde_json::Value = serde_json::from_str(
            r#"[{"id_alumne": 123456, "id_criteri": "4321", "nota": 7.5, "nota_mod": null, "text": "Muy bien, Lucía", "type": 1},
                {"id_alumne": "123456", "id_avaluacio": 331, "nota": "8", "type": 2}]"#,
        )
        .unwrap();
        a.json(&mut j, "");
        let s = j.to_string();
        for nada in ["Lucía", "123456", "4321", "331", "7.5"] {
            assert!(!s.contains(nada), "{nada} en {s}");
        }
        assert_eq!(j[0]["id_alumne"], 900001);
        assert_eq!(j[1]["id_alumne"], "900001");
        assert_eq!(j[0]["nota_mod"], serde_json::Value::Null);
        assert_eq!(j[0]["type"], 1);
        assert!(j[1]["nota"].is_string());
    }
}
