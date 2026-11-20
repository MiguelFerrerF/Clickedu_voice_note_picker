//! **Las fichas** (H4, la capa resistente): cada dato que se lee de
//! Clickedu, **cómo se busca** (primero lo más fiable: los parámetros de las
//! direcciones y los nombres de los campos; luego la estructura y las clases
//! CSS) y **qué tiene que cumplir** lo encontrado. Lo que no cumple, no vale.
//!
//! Cada lectura acaba en uno de tres estados ([`Lectura`]):
//!
//! - **Bien.**
//! - **Degradado**: la manera de siempre falló y acertó otra, o falta algo
//!   que no impide seguir (las fechas de las evaluaciones). El profesor
//!   sigue; queda el aviso, con el motivo.
//! - **Roto** ([`Roto`]): no se encuentra el dato, o lo encontrado no
//!   cumple. «Clickedu ha cambiado: …».
//!
//! Las páginas de los tests son de verdad, anonimizadas
//! (`tests/datos/clickedu/`, `examples/anonimizar_clickedu`). Lo que se
//! sabe de cada una, en `docs/03-clickedu.md`.

use std::collections::{HashMap, HashSet};

use reqwest::Url;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

/// Cómo ha ido una lectura que ha encontrado el dato.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "estado", rename_all = "snake_case")]
pub enum Estado {
    Bien,
    /// Se ha leído, pero no como siempre: el aviso temprano.
    Degradado {
        motivo: String,
    },
}

/// Un dato leído y cómo ha ido.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Leido<T> {
    pub dato: T,
    pub estado: Estado,
}

/// No se ha podido leer: Clickedu ha cambiado (o no es la página que se
/// esperaba). Sin nada de la página: va a la interfaz y al registro (D8).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Roto {
    /// La ficha: `materias`, `evaluaciones`, `items`, `importar`, `notas`.
    pub ficha: &'static str,
    pub motivo: String,
}

impl std::fmt::Display for Roto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clickedu ha cambiado: {}", self.motivo)
    }
}

pub type Lectura<T> = Result<Leido<T>, Roto>;

fn bien<T>(dato: T) -> Lectura<T> {
    Ok(Leido { dato, estado: Estado::Bien })
}

fn degradado<T>(dato: T, motivo: impl Into<String>) -> Lectura<T> {
    Ok(Leido { dato, estado: Estado::Degradado { motivo: motivo.into() } })
}

fn roto<T>(ficha: &'static str, motivo: impl Into<String>) -> Lectura<T> {
    Err(Roto { ficha, motivo: motivo.into() })
}

// ───────────────────────── Ayudas ─────────────────────────

fn selector(s: &str) -> Selector {
    Selector::parse(s).expect("selector fijo")
}

/// El texto de un elemento, con los espacios juntados.
fn texto(e: ElementRef) -> String {
    e.text().collect::<Vec<_>>().join(" ").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Los parámetros de una dirección (relativa o entera).
fn parametros(href: &str) -> HashMap<String, String> {
    let base = Url::parse("https://clickedu.invalid/go/").expect("base fija");
    base.join(href)
        .map(|u| u.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect())
        .unwrap_or_default()
}

fn es_id(v: &str) -> bool {
    !v.is_empty() && v.chars().all(|c| c.is_ascii_digit())
}

// ───────────────────────── Materias ─────────────────────────

/// Una materia del profesor, de `ctr=Subjects/Subjects`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Materia {
    /// El `id` de lo nuevo (`ctr=…&id=`), que es el `assig` de lo antiguo.
    pub id: String,
    /// «Física y Química».
    pub nombre: String,
    /// El curso y grupo: «3º E.S.O.-A», «PENDIENTE FÍSICA Y QUÍMICA 2ºESO».
    pub grupo: String,
    /// El apartado en que sale: «Tercero de ESO», «Primero de Bachillerato».
    pub curso: Option<String>,
}

/// Las materias, de la página `ctr=Subjects/Subjects`.
///
/// 1. **Por sus bloques** (`div.bloc-assignatura`, como la v1): el `id` del
///    enlace a la materia, `.title` y `.sub-title`, y el apartado
///    (`.panel-separador`) de antes. El `id` tiene que ser el mismo en
///    «Evaluar» (`id=`) y en las sesiones (`assig=`).
/// 2. **Solo por los enlaces** (`ctr=Subject/Subject&id=N`), si los bloques
///    no están o no cumplen: el nombre es el texto del enlace, todo junto.
///    Degradado.
pub fn materias(html: &str) -> Lectura<Vec<Materia>> {
    let doc = Html::parse_document(html);
    let por_bloques = materias_por_bloques(&doc);
    let motivo = match &por_bloques {
        Ok(m) if m.is_empty() => "no están en sus bloques de siempre (`bloc-assignatura`)".to_string(),
        Ok(m) => match comprobar_materias(m) {
            Ok(()) => return bien(por_bloques.unwrap_or_default()),
            Err(e) => e,
        },
        Err(e) => e.clone(),
    };
    let por_enlaces = materias_por_enlaces(&doc);
    if !por_enlaces.is_empty() && comprobar_materias(&por_enlaces).is_ok() {
        return degradado(por_enlaces, format!("las materias {motivo}: leídas solo por sus enlaces"));
    }
    roto("materias", format!("no encuentro las materias (las de siempre {motivo})"))
}

fn materias_por_bloques(doc: &Html) -> Result<Vec<Materia>, String> {
    let (titulo, subtitulo, enlace) = (selector(".title"), selector(".sub-title"), selector("a[href]"));
    let mut res = Vec::new();
    let mut curso = None;
    for e in doc.select(&selector("div.panel-separador, div.bloc-assignatura")) {
        if e.value().classes().any(|c| c == "panel-separador") {
            curso = e.select(&titulo).next().map(texto).filter(|t| !t.is_empty());
            continue;
        }
        // El id de cada enlace del bloque, por su forma.
        let mut id = None;
        for a in e.select(&enlace) {
            let p = parametros(a.value().attr("href").unwrap_or_default());
            let de_este = match p.get("ctr").map(String::as_str) {
                Some("Subject/Subject") | Some("Evaluate/EvaluateTermly") => p.get("id"),
                _ => p.get("assig"),
            };
            if let Some(v) = de_este {
                match &id {
                    None => id = Some(v.clone()),
                    Some(antes) if antes != v => {
                        return Err(format!("tienen enlaces con dos ids distintos ({antes} y {v})"));
                    }
                    _ => {}
                }
            }
        }
        let Some(id) = id else { return Err("tienen un bloque sin id".into()) };
        res.push(Materia {
            id,
            nombre: e.select(&titulo).next().map(texto).unwrap_or_default(),
            grupo: e.select(&subtitulo).next().map(texto).unwrap_or_default(),
            curso: curso.clone(),
        });
    }
    Ok(res)
}

fn materias_por_enlaces(doc: &Html) -> Vec<Materia> {
    let mut res: Vec<Materia> = Vec::new();
    for a in doc.select(&selector("a[href]")) {
        let p = parametros(a.value().attr("href").unwrap_or_default());
        if p.get("ctr").map(String::as_str) != Some("Subject/Subject") {
            continue;
        }
        let Some(id) = p.get("id") else { continue };
        if res.iter().any(|m| &m.id == id) {
            continue;
        }
        res.push(Materia { id: id.clone(), nombre: texto(a), grupo: String::new(), curso: None });
    }
    res
}

fn comprobar_materias(materias: &[Materia]) -> Result<(), String> {
    let mut vistos = HashSet::new();
    for m in materias {
        if !es_id(&m.id) {
            return Err(format!("tienen un id que no es un número ({})", m.id));
        }
        if !vistos.insert(&m.id) {
            return Err(format!("repiten un id ({})", m.id));
        }
        if m.nombre.is_empty() {
            return Err("tienen una sin nombre".into());
        }
    }
    Ok(())
}

// ───────────────────────── Evaluaciones ─────────────────────────

/// Una evaluación de una materia, del `<select name="trm">` de «Ítems».
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Evaluacion {
    /// El `trm` de lo nuevo, que es el `id_aval` de importar.
    pub id: String,
    /// «1ª evaluación».
    pub nombre: String,
    /// `AAAA-MM-DD`; sin fechas, no se puede proponer la de hoy.
    pub desde: Option<String>,
    pub hasta: Option<String>,
    /// La que trae elegida Clickedu.
    pub elegida: bool,
}

/// Las evaluaciones, de la página `ctr=Subject/Items&do=Index&id=N`: las
/// opciones con número del `<select name="trm">` («Todas» es `all`), con
/// sus fechas («1ª evaluación (de 04/09/2026 a 20/11/2026)»), que tienen
/// que ir seguidas. Sin fechas o desordenadas: degradado.
pub fn evaluaciones(html: &str) -> Lectura<Vec<Evaluacion>> {
    let doc = Html::parse_document(html);
    let Some(select) = doc.select(&selector(r#"select[name="trm"]"#)).next() else {
        return roto("evaluaciones", "no encuentro el desplegable de las evaluaciones (`trm`)");
    };
    let mut res = Vec::new();
    for o in select.select(&selector("option")) {
        let id = o.value().attr("value").unwrap_or_default();
        if !es_id(id) {
            continue;
        }
        let t = texto(o);
        let nombre = t.split(" (").next().unwrap_or_default().trim().to_string();
        let fechas = fechas_en(&t);
        let (desde, hasta) = match fechas.as_slice() {
            [d, h] => (Some(d.clone()), Some(h.clone())),
            _ => (None, None),
        };
        if nombre.is_empty() {
            return roto("evaluaciones", format!("la evaluación {id} no tiene nombre"));
        }
        res.push(Evaluacion {
            id: id.to_string(),
            nombre,
            desde,
            hasta,
            elegida: o.value().attr("selected").is_some(),
        });
    }
    if res.is_empty() {
        return roto("evaluaciones", "el desplegable de las evaluaciones (`trm`) no tiene ninguna");
    }
    if res.iter().any(|e| e.desde.is_none()) {
        return degradado(res, "hay evaluaciones sin sus fechas: no se puede proponer la de hoy");
    }
    let seguidas = res.windows(2).all(|p| p[0].hasta < p[1].desde) && res.iter().all(|e| e.desde <= e.hasta);
    if !seguidas {
        return degradado(res, "las fechas de las evaluaciones no van seguidas: no se puede proponer la de hoy");
    }
    bien(res)
}

/// La evaluación de `hoy` (`AAAA-MM-DD`), si las fechas lo dicen.
pub fn evaluacion_de_hoy<'a>(evaluaciones: &'a [Evaluacion], hoy: &str) -> Option<&'a Evaluacion> {
    evaluaciones.iter().find(|e| match (&e.desde, &e.hasta) {
        (Some(d), Some(h)) => d.as_str() <= hoy && hoy <= h.as_str(),
        _ => false,
    })
}

/// Las fechas `d/m/aaaa` de un texto, como `aaaa-mm-dd`.
fn fechas_en(t: &str) -> Vec<String> {
    t.split(|c: char| !(c.is_ascii_digit() || c == '/'))
        .filter_map(|trozo| {
            let p: Vec<&str> = trozo.split('/').collect();
            let [d, m, a] = p.as_slice() else { return None };
            let (d, m, a): (u32, u32, u32) = (d.parse().ok()?, m.parse().ok()?, a.parse().ok()?);
            ((1..=31).contains(&d) && (1..=12).contains(&m) && a >= 2000).then(|| format!("{a:04}-{m:02}-{d:02}"))
        })
        .collect()
}

// ───────────────────────── Ítems e importar ─────────────────────────

/// Un ítem de una evaluación: lo que se elige al subir las notas.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Item {
    /// El `id_criteri_avaluacio` al subir; el `it=` de «Ítems»; el
    /// `id_criteri` de `GetGrades`.
    pub id: String,
    pub nombre: String,
}

/// Los ítems de una evaluación, de la página de importar
/// (`importar_notes_items.php?assig=N&id_aval=T`): las opciones del
/// `<select name="id_criteri_avaluacio">` menos `---`. Puede no haber
/// ninguno (una evaluación aún sin ítems). Un id que no es un número,
/// roto: con él se sube.
pub fn items(html: &str) -> Lectura<Vec<Item>> {
    let doc = Html::parse_document(html);
    let Some(select) = doc.select(&selector(r#"select[name="id_criteri_avaluacio"]"#)).next() else {
        return roto("items", "no encuentro el desplegable de los ítems (`id_criteri_avaluacio`)");
    };
    let mut res: Vec<Item> = Vec::new();
    for o in select.select(&selector("option")) {
        let id = o.value().attr("value").unwrap_or_default().trim();
        if id.is_empty() {
            continue;
        }
        if !es_id(id) {
            return roto("items", format!("un ítem tiene un id que no es un número ({id})"));
        }
        let nombre = texto(o);
        if nombre.is_empty() {
            return roto("items", format!("el ítem {id} no tiene nombre"));
        }
        if res.iter().any(|i| i.id == id) {
            return roto("items", format!("el ítem {id} sale dos veces"));
        }
        res.push(Item { id: id.to_string(), nombre });
    }
    bien(res)
}

/// El formulario para subir la plantilla con las notas, de la página de
/// importar. **Solo se sube con él «bien»** (`docs/01`, H4).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Importar {
    /// La dirección a la que se manda (`…?accio=carregar_plantilla&assig=N&id_aval=T`).
    pub accion: String,
    /// El campo del ítem: `id_criteri_avaluacio`.
    pub campo_item: String,
    /// El campo del fichero: `v_arxiu_importar`.
    pub campo_fichero: String,
    /// El botón, que también se manda: `BotoGenerar`, con su valor.
    pub boton: Option<(String, String)>,
}

/// El formulario de subir, de la página de importar: un `<form>` `POST`
/// *multipart* con el desplegable del ítem y un campo de fichero, hacia
/// `accio=carregar_plantilla` con el `assig` y el `id_aval` de la página.
/// Si algo no es como siempre (otro nombre de campo, otra acción), roto:
/// para escribir no vale «más o menos».
pub fn importar(html: &str, assig: &str, id_aval: &str) -> Lectura<Importar> {
    let doc = Html::parse_document(html);
    let fichero = selector(r#"input[type="file" i]"#);
    let Some(form) = doc.select(&selector("form")).find(|f| f.select(&fichero).next().is_some()) else {
        return roto("importar", "no encuentro el formulario para subir la plantilla");
    };
    let f = form.value();
    let accion = f.attr("action").unwrap_or_default().to_string();
    let p = parametros(&accion);
    let mut fallos = Vec::new();
    if !f.attr("method").is_some_and(|m| m.eq_ignore_ascii_case("post")) {
        fallos.push("no se manda por POST".to_string());
    }
    if !f.attr("enctype").is_some_and(|e| e.eq_ignore_ascii_case("multipart/form-data")) {
        fallos.push("no es multipart".to_string());
    }
    if p.get("accio").map(String::as_str) != Some("carregar_plantilla") {
        fallos.push("la acción no es `carregar_plantilla`".to_string());
    }
    if p.get("assig").map(String::as_str) != Some(assig) || p.get("id_aval").map(String::as_str) != Some(id_aval) {
        fallos.push("no es de esta materia y evaluación".to_string());
    }
    let campo_fichero = form.select(&fichero).next().and_then(|i| i.value().attr("name")).unwrap_or_default();
    if campo_fichero != "v_arxiu_importar" {
        fallos.push(format!("el campo del fichero se llama `{campo_fichero}`"));
    }
    let campo_item =
        form.select(&selector("select[name]")).next().and_then(|s| s.value().attr("name")).unwrap_or_default();
    if campo_item != "id_criteri_avaluacio" {
        fallos.push("no tiene el desplegable del ítem".to_string());
    }
    if !fallos.is_empty() {
        return roto("importar", format!("el formulario para subir la plantilla ha cambiado: {}", fallos.join("; ")));
    }
    let boton = form.select(&selector(r#"input[type="submit" i][name], button[name]"#)).next().map(|b| {
        (
            b.value().attr("name").unwrap_or_default().to_string(),
            b.value().attr("value").unwrap_or_default().to_string(),
        )
    });
    bien(Importar { accion, campo_item: campo_item.into(), campo_fichero: campo_fichero.into(), boton })
}

// ───────────────────────── Notas puestas ─────────────────────────

/// Una nota ya puesta, de `GetGrades`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Puesta {
    /// El alumno: el `Id` de la plantilla.
    pub alumno: String,
    /// El ítem; o la materia, en la nota final.
    pub criterio: String,
    /// `0` en los ítems; otros números en la nota final (`docs/03`).
    pub tipo: Option<i64>,
    /// Las dos que da Clickedu, tal cual (un número o un texto). Por lo
    /// visto, `nota` es la calculada y `nota_mod` la puesta a mano: está sin
    /// confirmar (H6, la primera subida).
    pub nota: Option<String>,
    pub nota_mod: Option<String>,
}

/// Las notas puestas, del JSON de
/// `ctr=Evaluate/EvaluateTermly&do=GetGrades&id=N&trm=T`: una lista de
/// objetos con `id_alumne`, `id_criteri`, `nota`, `nota_mod` y `type`. Un
/// alumno o un criterio sin número, roto: con eso se comprueba lo subido.
pub fn notas(json: &str) -> Lectura<Vec<Puesta>> {
    use serde_json::Value;
    let Ok(v) = serde_json::from_str::<Value>(json) else {
        return roto("notas", "las notas puestas no llegan como JSON");
    };
    let Value::Array(filas) = v else {
        return roto("notas", "las notas puestas no son una lista");
    };
    let como_texto = |v: Option<&Value>| match v {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    };
    let mut res = Vec::new();
    let mut sin_nota = 0;
    for f in &filas {
        let Value::Object(o) = f else { return roto("notas", "una nota puesta no es un objeto") };
        let (Some(alumno), Some(criterio)) = (como_texto(o.get("id_alumne")), como_texto(o.get("id_criteri"))) else {
            return roto("notas", "una nota puesta no dice de qué alumno o de qué ítem es");
        };
        if !es_id(&alumno) || !es_id(&criterio) {
            return roto("notas", "una nota puesta tiene un alumno o un ítem que no es un número");
        }
        if !o.contains_key("nota") && !o.contains_key("nota_mod") {
            sin_nota += 1;
        }
        res.push(Puesta {
            alumno,
            criterio,
            tipo: o.get("type").and_then(Value::as_i64),
            nota: como_texto(o.get("nota")),
            nota_mod: como_texto(o.get("nota_mod")),
        });
    }
    if sin_nota > 0 && sin_nota == res.len() {
        return roto("notas", "las notas puestas no traen ni `nota` ni `nota_mod`");
    }
    bien(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATERIAS: &str = include_str!("../../tests/datos/clickedu/materias.html");
    const ITEMS: &str = include_str!("../../tests/datos/clickedu/items.html");
    const IMPORTAR: &str = include_str!("../../tests/datos/clickedu/importar.html");
    const NOTAS_1: &str = include_str!("../../tests/datos/clickedu/notas-1.json");
    const NOTAS_2: &str = include_str!("../../tests/datos/clickedu/notas-2.json");

    #[test]
    fn las_materias_de_verdad() {
        let l = materias(MATERIAS).unwrap();
        assert_eq!(l.estado, Estado::Bien);
        assert_eq!(l.dato.len(), 10);
        assert_eq!(
            l.dato[0],
            Materia {
                id: "9001".into(),
                nombre: "Biología y Geología".into(),
                grupo: "3º E.S.O.-A".into(),
                curso: Some("Tercero de ESO".into()),
            }
        );
        assert_eq!(l.dato[5].curso.as_deref(), Some("Primero de Bachillerato"));
        assert_eq!(l.dato[3].grupo, "PENDIENTE FÍSICA Y QUÍMICA 2ºESO");
    }

    #[test]
    fn materias_sin_sus_bloques_degradado() {
        let sin_clases = MATERIAS.replace("bloc-assignatura", "otra-cosa");
        let l = materias(&sin_clases).unwrap();
        assert!(matches!(l.estado, Estado::Degradado { .. }), "{:?}", l.estado);
        assert_eq!(l.dato.len(), 10);
        assert_eq!(l.dato[0].id, "9001");
        assert!(l.dato[0].nombre.contains("Biología y Geología"), "{}", l.dato[0].nombre);
    }

    #[test]
    fn materias_que_no_cuadran_degradado() {
        // El `assig` de las sesiones ya no es el `id`: los bloques no valen.
        let cambiado = MATERIAS.replacen("sessions.php?assig=9001", "sessions.php?assig=9999", 1);
        let l = materias(&cambiado).unwrap();
        match l.estado {
            Estado::Degradado { motivo } => assert!(motivo.contains("dos ids distintos"), "{motivo}"),
            otro => panic!("{otro:?}"),
        }
    }

    #[test]
    fn sin_materias_roto() {
        let r = materias("<html><body><p>Bienvenido al nuevo Clickedu</p></body></html>").unwrap_err();
        assert_eq!(r.ficha, "materias");
        assert!(r.to_string().starts_with("Clickedu ha cambiado: no encuentro las materias"), "{r}");
    }

    #[test]
    fn las_evaluaciones_de_verdad() {
        let l = evaluaciones(ITEMS).unwrap();
        assert_eq!(l.estado, Estado::Bien);
        assert_eq!(l.dato.len(), 3);
        assert_eq!(
            l.dato[0],
            Evaluacion {
                id: "902".into(),
                nombre: "1ª evaluación".into(),
                desde: Some("2026-09-04".into()),
                hasta: Some("2026-11-20".into()),
                elegida: true,
            }
        );
        assert_eq!(l.dato[2].hasta.as_deref(), Some("2027-06-21"));
        assert_eq!(evaluacion_de_hoy(&l.dato, "2026-11-21").map(|e| e.id.as_str()), Some("903"));
        assert_eq!(evaluacion_de_hoy(&l.dato, "2026-11-20").map(|e| e.id.as_str()), Some("902"));
        assert_eq!(evaluacion_de_hoy(&l.dato, "2027-08-01"), None);
    }

    #[test]
    fn evaluaciones_sin_fechas_degradado() {
        let sin_fechas = ITEMS.replace("(de 21/11/2026 a 12/03/2027)", "");
        let l = evaluaciones(&sin_fechas).unwrap();
        assert!(matches!(l.estado, Estado::Degradado { .. }), "{:?}", l.estado);
        assert_eq!(l.dato[1].nombre, "2ª evaluación");
        let desordenadas = ITEMS.replace("(de 21/11/2026 a 12/03/2027)", "(de 21/11/2025 a 12/03/2026)");
        assert!(matches!(evaluaciones(&desordenadas).unwrap().estado, Estado::Degradado { .. }));
        assert_eq!(evaluaciones(MATERIAS).unwrap_err().ficha, "evaluaciones");
    }

    #[test]
    fn los_items_de_verdad() {
        let l = items(IMPORTAR).unwrap();
        assert_eq!(l.estado, Estado::Bien);
        assert_eq!(l.dato.len(), 10);
        assert_eq!(l.dato[8], Item { id: "90009".into(), nombre: "Parcial UD1".into() });
        // Los mismos que los de la página de Ítems (`it=`).
        for i in &l.dato {
            assert!(ITEMS.contains(&format!("it={}", i.id)) || ITEMS.contains(&format!("_{}_", i.id)), "{}", i.id);
        }
        let sin_id = IMPORTAR.replace(r#"<option value="90009">"#, r#"<option value="ítem9">"#);
        assert_eq!(items(&sin_id).unwrap_err().ficha, "items");
        assert_eq!(items(MATERIAS).unwrap_err().ficha, "items");
    }

    #[test]
    fn el_formulario_de_importar_de_verdad() {
        let l = importar(IMPORTAR, "9001", "902").unwrap();
        assert_eq!(l.estado, Estado::Bien);
        assert_eq!(
            l.dato.accion,
            "/assignatures/importar_notes_items.php?accio=carregar_plantilla&assig=9001&id_aval=902"
        );
        assert_eq!(l.dato.campo_item, "id_criteri_avaluacio");
        assert_eq!(l.dato.campo_fichero, "v_arxiu_importar");
        assert_eq!(l.dato.boton.as_ref().map(|b| b.0.as_str()), Some("BotoGenerar"));
        // De otra materia, o con otro campo de fichero: no se sube.
        assert!(importar(IMPORTAR, "9002", "902").is_err());
        let otro_campo = IMPORTAR.replace(r#"name="v_arxiu_importar""#, r#"name="fitxer""#);
        let r = importar(&otro_campo, "9001", "902").unwrap_err();
        assert!(r.motivo.contains("`fitxer`"), "{r}");
        assert!(importar(MATERIAS, "9001", "902").is_err());
    }

    #[test]
    fn las_notas_de_verdad() {
        let l = notas(NOTAS_1).unwrap();
        assert_eq!(l.estado, Estado::Bien);
        assert_eq!(l.dato.len(), 266);
        // Las de los ítems, `type` 0; la final, con el id de la materia.
        let del_item: Vec<&Puesta> = l.dato.iter().filter(|p| p.criterio == "90001").collect();
        assert_eq!(del_item.len(), 30);
        assert!(del_item.iter().all(|p| p.tipo == Some(0)));
        assert!(l.dato.iter().filter(|p| p.criterio == "9001").all(|p| p.tipo != Some(0)));
        // Los alumnos de las notas, los de una clase: 30.
        let alumnos: HashSet<&str> = l.dato.iter().map(|p| p.alumno.as_str()).collect();
        assert_eq!(alumnos.len(), 30);

        // La otra forma: `nota` como texto y sin `nota_mod`.
        let l = notas(NOTAS_2).unwrap();
        assert_eq!(l.dato.len(), 30);
        assert!(l.dato.iter().all(|p| p.nota.is_some() && p.nota_mod.is_none()));
    }

    #[test]
    fn notas_que_no_son_notas_roto() {
        assert_eq!(notas("<html></html>").unwrap_err().ficha, "notas");
        assert!(notas(r#"{"a": 1}"#).is_err());
        assert!(notas(r#"[{"id_alumne": "x", "id_criteri": 1, "nota": 5}]"#).is_err());
        assert!(notas(r#"[{"id_alumne": 1, "id_criteri": 2}]"#).is_err());
        assert_eq!(notas("[]").unwrap().dato, []);
    }
}
