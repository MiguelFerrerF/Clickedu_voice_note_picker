//! **La huella** de una página de Clickedu (H4, la capa resistente): su
//! forma, **sin nombres** (con [`Tapar`], como el informe del explorador):
//! los formularios con sus campos, los desplegables, las formas de los
//! enlaces y las clases CSS de las que tiran las fichas.
//!
//! Se guarda **la última buena** de cada ficha
//! (`%APPDATA%\es.gestornotas.app\clickedu\huellas\<ficha>.json`) y se
//! compara con la de ahora: la diferencia dice qué ha cambiado y dónde. Es
//! el aviso temprano, aunque la ficha aún lea bien, y lo que irá en el aviso
//! de D8: no lleva nada de nadie.
//!
//! Lo que cambia con los datos no entra: cuántas opciones tiene un
//! desplegable, los textos, los números (`id=▢`).

use std::collections::BTreeSet;
use std::path::Path;

use reqwest::Url;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use super::tapar::Tapar;

/// Las clases CSS de las que tiran las fichas: si desaparecen, se dice.
/// Las demás (colores, rejilla) cambian sin que importe.
pub const CLASES_QUE_SE_USAN: &[&str] = &["bloc-assignatura", "panel-separador", "title", "sub-title"];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Huella {
    /// `POST /assignatures/importar_notes_items.php?accio=carregar_plantilla&assig=▢&id_aval=▢
    /// multipart: BotoGenerar, id_criteri_avaluacio, v_arxiu_importar (fichero)`.
    pub formularios: Vec<String>,
    /// Los nombres de los `<select>`: `id_criteri_avaluacio`, `trm`.
    pub desplegables: Vec<String>,
    /// Las formas de los enlaces: `/go/Index.php?ctr=Subject/Subject&do=Index&id=▢`.
    pub enlaces: Vec<String>,
    /// De [`CLASES_QUE_SE_USAN`], las que hay.
    pub clases: Vec<String>,
}

fn selector(s: &str) -> Selector {
    Selector::parse(s).expect("selector fijo")
}

/// La huella de `html`, que es la página de `url`.
pub fn huella(html: &str, url: &str, tapar: &Tapar) -> Huella {
    let doc = Html::parse_document(html);
    let base = Url::parse(url).unwrap_or_else(|_| Url::parse("https://clickedu.invalid/").expect("base fija"));
    let forma = |href: &str| -> Option<String> {
        let href = href.trim();
        if href.is_empty() || href.starts_with('#') || href.to_lowercase().starts_with("javascript:") {
            return None;
        }
        let u = base.join(href).ok()?;
        (u.host_str() == base.host_str()).then(|| tapar.url(&u, false))
    };

    let mut formularios = BTreeSet::new();
    let campos = selector("input[name], select[name], textarea[name], button[name]");
    for f in doc.select(&selector("form")) {
        let e = f.value();
        let metodo = e.attr("method").unwrap_or("get").to_uppercase();
        let accion = e.attr("action").and_then(forma).unwrap_or_else(|| "(la misma página)".into());
        let mut nombres: BTreeSet<String> = BTreeSet::new();
        for c in f.select(&campos) {
            let nombre = tapar.codigo(c.value().attr("name").unwrap_or_default());
            let fichero = c.value().attr("type").is_some_and(|t| t.eq_ignore_ascii_case("file"));
            nombres.insert(if fichero { format!("{nombre} (fichero)") } else { nombre });
        }
        let multipart = e.attr("enctype").is_some_and(|t| t.eq_ignore_ascii_case("multipart/form-data"));
        let nombres = nombres.into_iter().collect::<Vec<_>>().join(", ");
        formularios.insert(format!("{metodo} {accion}{}: {nombres}", if multipart { " multipart" } else { "" }));
    }

    let desplegables: BTreeSet<String> = doc
        .select(&selector("select[name]"))
        .map(|s| tapar.codigo(s.value().attr("name").unwrap_or_default()))
        .collect();
    let enlaces: BTreeSet<String> =
        doc.select(&selector("a[href]")).filter_map(|a| a.value().attr("href").and_then(forma)).collect();
    let mut clases: BTreeSet<String> = BTreeSet::new();
    for e in doc.select(&selector("[class]")) {
        for c in e.value().classes() {
            if CLASES_QUE_SE_USAN.contains(&c) {
                clases.insert(c.to_string());
            }
        }
    }
    Huella {
        formularios: formularios.into_iter().collect(),
        desplegables: desplegables.into_iter().collect(),
        enlaces: enlaces.into_iter().collect(),
        clases: clases.into_iter().collect(),
    }
}

/// Lo que ha cambiado de `antes` a `ahora`, una línea por cosa: «ya no
/// está el formulario …», «formulario nuevo: …».
pub fn diferencias(antes: &Huella, ahora: &Huella) -> Vec<String> {
    let mut res = Vec::new();
    let mut comparar = |que: &str, a: &[String], b: &[String]| {
        for x in a.iter().filter(|x| !b.contains(x)) {
            res.push(format!("ya no está {que} {x}"));
        }
        for x in b.iter().filter(|x| !a.contains(x)) {
            res.push(format!("{que} nuevo: {x}"));
        }
    };
    comparar("el formulario", &antes.formularios, &ahora.formularios);
    comparar("el desplegable", &antes.desplegables, &ahora.desplegables);
    comparar("el enlace", &antes.enlaces, &ahora.enlaces);
    comparar("la clase", &antes.clases, &ahora.clases);
    res
}

/// La ficha de la página de Ítems de una materia.
pub const FICHA_ITEMS: &str = "evaluaciones";

/// La huella de la página de Ítems, solo con lo que la app lee de ella (el
/// desplegable `trm`, la ponderación, las clases). Fuera, lo que depende de
/// que haya ítems o criterios: el formulario de borrar (`Subject/Items&do=Delete`,
/// con sus casillas `percent_…`) y los enlaces de editar o borrar un ítem
/// (`Subject/Item/Item&do=…`): cambian al crear o editar un ítem, no porque
/// Clickedu cambie.
pub fn solo_lo_que_se_lee_de_items(h: &Huella) -> Huella {
    let mut h = h.clone();
    h.formularios.retain(|f| !f.contains("ctr=Subject/Items&do=Delete"));
    h.enlaces.retain(|e| !e.contains("ctr=Subject/Item/Item&do="));
    h
}

/// Compara `ahora` con la última buena de la `ficha`, guardada en
/// `carpeta\huellas\`, y devuelve las diferencias. Si `es_buena` (la ficha
/// ha leído bien), `ahora` pasa a ser la última buena. La primera vez no hay
/// con qué comparar: se guarda y no hay diferencias.
pub fn comparar_y_guardar(carpeta: &Path, ficha: &str, ahora: &Huella, es_buena: bool) -> Result<Vec<String>, String> {
    let dir = carpeta.join("huellas");
    let ruta = dir.join(format!("{ficha}.json"));
    // La de Ítems se filtra también la guardada: las de antes, sin filtrar,
    // no deben avisar de que «ya no está» lo que ya no se mira.
    let filtrar = |h: &Huella| if ficha == FICHA_ITEMS { solo_lo_que_se_lee_de_items(h) } else { h.clone() };
    let antes: Option<Huella> =
        std::fs::read_to_string(&ruta).ok().and_then(|t| serde_json::from_str::<Huella>(&t).ok()).map(|h| filtrar(&h));
    let ahora = &filtrar(ahora);
    let cambios = antes.as_ref().map(|a| diferencias(a, ahora)).unwrap_or_default();
    if es_buena && antes.as_ref() != Some(ahora) {
        std::fs::create_dir_all(&dir).map_err(|e| format!("no se puede crear {}: {e}", dir.display()))?;
        let json = serde_json::to_string_pretty(ahora).map_err(|e| e.to_string())?;
        std::fs::write(&ruta, json).map_err(|e| format!("no se puede guardar la huella en {}: {e}", ruta.display()))?;
    }
    Ok(cambios)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MATERIAS: &str = include_str!("../../tests/datos/clickedu/materias.html");
    const IMPORTAR: &str = include_str!("../../tests/datos/clickedu/importar.html");
    const URL_IMPORTAR: &str =
        "https://colegio.clickedu.eu/assignatures/importar_notes_items.php?assig=9001&id_aval=902";

    fn tapar() -> Tapar {
        Tapar::new(["GARCÍA LÓPEZ, Lucía"])
    }

    #[test]
    fn la_huella_de_importar() {
        let h = huella(IMPORTAR, URL_IMPORTAR, &tapar());
        assert!(
            h.formularios.contains(
                &"POST /assignatures/importar_notes_items.php?accio=carregar_plantilla&assig=▢&id_aval=▢ multipart: \
                  BotoGenerar, id_criteri_avaluacio, v_arxiu_importar (fichero)"
                    .to_string()
            ),
            "{:#?}",
            h.formularios
        );
        assert!(h.desplegables.contains(&"id_criteri_avaluacio".to_string()), "{:?}", h.desplegables);
        assert!(
            h.enlaces.contains(
                &"/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig=▢&id_aval=▢".into()
            ),
            "{:#?}",
            h.enlaces
        );
        // Ni un número de verdad ni un texto.
        let todo = format!("{h:?}");
        assert!(!todo.contains("9001") && !todo.contains("Biología"), "{todo}");
    }

    #[test]
    fn misma_pagina_misma_huella_y_un_cambio_se_dice() {
        let t = tapar();
        let antes = huella(IMPORTAR, URL_IMPORTAR, &t);
        // Otros datos, misma forma: otra materia y un ítem más.
        let otros = IMPORTAR.replace("9001", "9002").replace(
            r#"<option value="90009">Parcial UD1</option>"#,
            r#"<option value="90009">Parcial UD1</option><option value="90011">Parcial UD2</option>"#,
        );
        assert_eq!(diferencias(&antes, &huella(&otros, URL_IMPORTAR, &t)), Vec::<String>::new());
        // El campo del fichero cambia de nombre.
        let cambiada = IMPORTAR.replace("v_arxiu_importar", "fitxer");
        let d = diferencias(&antes, &huella(&cambiada, URL_IMPORTAR, &t));
        assert_eq!(d.len(), 2, "{d:#?}");
        assert!(d[0].starts_with("ya no está el formulario POST") && d[0].contains("v_arxiu_importar"), "{d:#?}");
        assert!(d[1].starts_with("el formulario nuevo: POST") && d[1].contains("fitxer (fichero)"), "{d:#?}");
    }

    #[test]
    fn las_clases_que_se_usan() {
        let h = huella(MATERIAS, "https://colegio.clickedu.eu/go/Index.php?ctr=Subjects/Subjects", &tapar());
        assert_eq!(h.clases, ["bloc-assignatura", "panel-separador", "sub-title", "title"]);
        let sin = MATERIAS.replace("bloc-assignatura", "subject-card");
        let d =
            diferencias(&h, &huella(&sin, "https://colegio.clickedu.eu/go/Index.php?ctr=Subjects/Subjects", &tapar()));
        assert_eq!(d, ["ya no está la clase bloc-assignatura"]);
    }

    #[test]
    fn se_guarda_la_ultima_buena() {
        let dir = tempfile::tempdir().unwrap();
        let t = tapar();
        let buena = huella(IMPORTAR, URL_IMPORTAR, &t);
        // La primera vez, nada con que comparar.
        assert_eq!(comparar_y_guardar(dir.path(), "importar", &buena, true).unwrap(), Vec::<String>::new());
        let mala = huella(&IMPORTAR.replace("v_arxiu_importar", "fitxer"), URL_IMPORTAR, &t);
        assert_eq!(comparar_y_guardar(dir.path(), "importar", &mala, false).unwrap().len(), 2);
        // La mala no ha sustituido a la buena.
        assert_eq!(comparar_y_guardar(dir.path(), "importar", &mala, false).unwrap().len(), 2);
        assert_eq!(comparar_y_guardar(dir.path(), "importar", &buena, true).unwrap(), Vec::<String>::new());
    }

    const ITEMS: &str = include_str!("../../tests/datos/clickedu/items.html");
    const URL_ITEMS: &str = "https://colegio.clickedu.eu/go/Index.php?ctr=Subject/Items&do=Index&id=9001";

    #[test]
    fn items_no_avisa_por_crear_o_editar_items() {
        let t = tapar();
        let dir = tempfile::tempdir().unwrap();
        let antes = huella(ITEMS, URL_ITEMS, &t);
        assert_eq!(comparar_y_guardar(dir.path(), FICHA_ITEMS, &antes, true).unwrap(), Vec::<String>::new());
        // Otro formulario de borrar y otros enlaces de editar.
        let otra = ITEMS
            .replace(r#"name="items_eliminar""#, r#"name="otros_eliminar""#)
            .replace("Subject/Item/Item&amp;do=Edit", "Subject/Item/Item&amp;do=Modify")
            .replace("Subject/Item/Item&amp;do=Delete", "Subject/Item/Item&amp;do=Remove");
        let h = huella(&otra, URL_ITEMS, &t);
        assert_ne!(diferencias(&antes, &h), Vec::<String>::new(), "sin filtrar sí cambia");
        assert_eq!(comparar_y_guardar(dir.path(), FICHA_ITEMS, &h, true).unwrap(), Vec::<String>::new());
        // Sin el formulario ni los enlaces de ítems (una materia sin ítems).
        let mut sin = antes.clone();
        sin.formularios.retain(|f| !f.contains("Subject/Items&do=Delete"));
        sin.enlaces.retain(|e| !e.contains("Subject/Item/Item&do="));
        assert_ne!(sin, antes);
        assert_eq!(comparar_y_guardar(dir.path(), FICHA_ITEMS, &sin, true).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn items_sigue_avisando_de_lo_que_se_lee() {
        let t = tapar();
        let dir = tempfile::tempdir().unwrap();
        let antes = huella(ITEMS, URL_ITEMS, &t);
        comparar_y_guardar(dir.path(), FICHA_ITEMS, &antes, true).unwrap();
        let renombrado = ITEMS.replace(r#"name="trm""#, r#"name="trimestre""#);
        let d = comparar_y_guardar(dir.path(), FICHA_ITEMS, &huella(&renombrado, URL_ITEMS, &t), false).unwrap();
        assert!(d.iter().any(|x| x.contains("ya no está el desplegable trm")), "{d:#?}");
    }

    #[test]
    fn items_con_una_huella_vieja_sin_filtrar_no_avisa() {
        let t = tapar();
        let dir = tempfile::tempdir().unwrap();
        let vieja = huella(ITEMS, URL_ITEMS, &t);
        assert!(vieja.formularios.iter().any(|f| f.contains("Subject/Items&do=Delete")), "{vieja:#?}");
        std::fs::create_dir_all(dir.path().join("huellas")).unwrap();
        std::fs::write(dir.path().join("huellas").join("evaluaciones.json"), serde_json::to_string(&vieja).unwrap())
            .unwrap();
        let otra = ITEMS.replace(r#"name="items_eliminar""#, r#"name="otros_eliminar""#);
        let h = huella(&otra, URL_ITEMS, &t);
        assert_eq!(comparar_y_guardar(dir.path(), FICHA_ITEMS, &h, true).unwrap(), Vec::<String>::new());
    }
}
