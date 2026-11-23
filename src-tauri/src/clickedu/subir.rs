//! **Subir las notas** a un ítem de Clickedu (H6, `docs/01`). Lo de cada
//! página, en `docs/03` («Importar notas»): subir no es importar, y son
//! tres pasos que escriben.
//!
//! - [`leer_antes`] — solo lee: la página de importar (el ítem está y el
//!   formulario es el de siempre, **«bien»**: para escribir no vale «más o
//!   menos»), qué hay ya subido y las notas del ítem (`GetGrades`). Con
//!   [`comparar`], qué notas se pisarían: **se pregunta antes de subir**.
//! - [`subir`] — escribe: el `POST` de `carregar_plantilla` con el Excel;
//!   en la página otra vez, **el subido nuevo** (el que no estaba antes,
//!   uno y solo uno); su enlace `accio=importar_notes` (un `GET`); y
//!   `GetGrades` de nuevo, para **comprobar** que entraron.
//!
//! Lo subido se queda en la tabla de Clickedu, como si se hubiera subido a
//! mano: borrarlo no quita las notas importadas.

use std::collections::{BTreeMap, BTreeSet};

use scraper::{Html, Selector};
use serde::Serialize;

use super::cliente::{Cliente, Fallo};
use super::fichas::{self, Estado, Importar, Roto};
use super::items::{self, direccion_importar, direccion_notas};

/// A qué ítem se sube: los tres números de Clickedu.
#[derive(Debug, Clone, PartialEq)]
pub struct Destino {
    /// El `assig`.
    pub materia: String,
    /// El `id_aval` (`trm` en `GetGrades`).
    pub evaluacion: String,
    /// El `id_criteri_avaluacio`.
    pub item: String,
}

/// En qué paso va, para el avance y para decir dónde se paró.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Paso {
    Leer,
    Subir,
    Importar,
    Comprobar,
}

/// Por qué no se ha subido. `paso` dice dónde se paró: tras `subir`, el
/// fichero ya está en Clickedu sin importar (se puede importar allí a mano).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum FalloSubir {
    /// No se llega a Clickedu, o la sesión no vuelve.
    Conexion { paso: Paso, fallo: Fallo },
    /// No se ha entrado en Clickedu y no hay cuenta guardada.
    SinEntrar,
    /// El ítem ya no está en la página de importar (lo han borrado).
    SinItem,
    /// Una página no es como se esperaba: Clickedu ha cambiado, o no ha
    /// hecho lo que se le pedía. La página se guarda en local, no va a la
    /// interfaz.
    Cambiado {
        paso: Paso,
        motivo: String,
        #[serde(skip)]
        pagina: String,
    },
    /// Lo demás: el disco, la plantilla…
    Otro { detalle: String },
}

impl std::fmt::Display for FalloSubir {
    /// Para el registro (D8): sin páginas ni nombres.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FalloSubir::Conexion { paso, fallo } => write!(f, "al {paso:?}: {fallo}"),
            FalloSubir::SinEntrar => f.write_str("no se ha entrado en Clickedu"),
            FalloSubir::SinItem => f.write_str("el ítem ya no está en Clickedu"),
            FalloSubir::Cambiado { paso, motivo, .. } => write!(f, "al {paso:?}: Clickedu ha cambiado: {motivo}"),
            FalloSubir::Otro { detalle } => f.write_str(detalle),
        }
    }
}

impl From<String> for FalloSubir {
    fn from(detalle: String) -> Self {
        FalloSubir::Otro { detalle }
    }
}

fn cambiado(paso: Paso, motivo: impl Into<String>, pagina: &str) -> FalloSubir {
    FalloSubir::Cambiado { paso, motivo: motivo.into(), pagina: pagina.to_string() }
}

fn roto(paso: Paso, r: Roto, pagina: &str) -> FalloSubir {
    cambiado(paso, r.motivo, pagina)
}

/// Lo leído antes de subir.
#[derive(Debug, Clone)]
pub struct Antes {
    pub formulario: Importar,
    /// Los números de lo ya subido a esta materia y evaluación.
    pub subidos: BTreeSet<String>,
    /// Las notas del ítem en Clickedu, por `Id` de alumno ([`items::notas_de`]).
    pub notas: BTreeMap<String, String>,
}

/// Lee la página de importar y las notas del ítem. Solo lee.
pub async fn leer_antes(cliente: &Cliente, d: &Destino) -> Result<Antes, FalloSubir> {
    let conexion = |fallo| FalloSubir::Conexion { paso: Paso::Leer, fallo };
    let pagina = cliente.abrir(&direccion_importar(&d.materia, &d.evaluacion)).await.map_err(conexion)?.texto;
    let items = fichas::items(&pagina).map_err(|r| roto(Paso::Leer, r, &pagina))?;
    if !items.dato.iter().any(|i| i.id == d.item) {
        return Err(FalloSubir::SinItem);
    }
    let formulario = fichas::importar(&pagina, &d.materia, &d.evaluacion).map_err(|r| roto(Paso::Leer, r, &pagina))?;
    if let Estado::Degradado { motivo } = formulario.estado {
        return Err(cambiado(Paso::Leer, motivo, &pagina));
    }
    let notas = leer_notas(cliente, d, Paso::Leer).await?;
    Ok(Antes { formulario: formulario.dato, subidos: subidos(&pagina), notas })
}

async fn leer_notas(cliente: &Cliente, d: &Destino, paso: Paso) -> Result<BTreeMap<String, String>, FalloSubir> {
    let json = cliente
        .abrir(&direccion_notas(&d.materia, &d.evaluacion))
        .await
        .map_err(|fallo| FalloSubir::Conexion { paso, fallo })?
        .texto;
    let puestas = fichas::notas(&json).map_err(|r| roto(paso, r, &json))?;
    Ok(items::notas_de(&puestas.dato, &d.item))
}

/// Lo hecho al subir.
#[derive(Debug, Clone, PartialEq)]
pub struct Subida {
    /// Las notas del ítem en Clickedu, leídas después de importar.
    pub despues: BTreeMap<String, String>,
    /// Lo que Clickedu no ha dicho como siempre (el `alert` de un paso) sin
    /// que eso impida seguir: el aviso temprano.
    pub avisos: Vec<String>,
}

/// Sube `excel` (nombre y contenido) al ítem e importa sus notas; luego las
/// vuelve a leer. `antes` es lo de [`leer_antes`], recién leído. **Escribe.**
pub async fn subir(
    cliente: &Cliente,
    d: &Destino,
    antes: &Antes,
    excel: (&str, Vec<u8>),
    avance: impl Fn(Paso),
) -> Result<Subida, FalloSubir> {
    let mut avisos = Vec::new();
    let f = &antes.formulario;

    // 1. Subir.
    avance(Paso::Subir);
    let mut campos = vec![(f.campo_item.as_str(), d.item.as_str())];
    if let Some((n, v)) = &f.boton {
        campos.push((n.as_str(), v.as_str()));
    }
    let r = cliente
        .subir(&direccion_de(&f.accion), &campos, (&f.campo_fichero, excel.0, excel.1))
        .await
        .map_err(|fallo| FalloSubir::Conexion { paso: Paso::Subir, fallo })?;
    avisar(&mut avisos, "subir", &r.texto, "subido correctamente");

    // 2. El subido nuevo, en la página otra vez.
    let pagina = cliente
        .abrir(&direccion_importar(&d.materia, &d.evaluacion))
        .await
        .map_err(|fallo| FalloSubir::Conexion { paso: Paso::Importar, fallo })?
        .texto;
    let nuevos: Vec<String> = subidos(&pagina).difference(&antes.subidos).cloned().collect();
    let [nuevo] = nuevos.as_slice() else {
        let motivo = if nuevos.is_empty() {
            format!("tras subir, no sale el fichero en la tabla{}", motivo_alert(&r.texto))
        } else {
            format!("tras subir, salen {} ficheros nuevos y no se sabe cuál es", nuevos.len())
        };
        return Err(cambiado(Paso::Subir, motivo, &pagina));
    };
    let Some(enlace) = enlace_importar(&pagina, nuevo) else {
        return Err(cambiado(Paso::Importar, "el fichero subido no tiene el enlace para importarlo", &pagina));
    };

    // 3. Importar. Si la sesión ha caducado, `abrir` vuelve a entrar y lo
    // pide otra vez: la primera vez no ha llegado a importar.
    avance(Paso::Importar);
    let r = cliente.abrir(&enlace).await.map_err(|fallo| FalloSubir::Conexion { paso: Paso::Importar, fallo })?;
    avisar(&mut avisos, "importar", &r.texto, "importación correcta");

    // 4. Comprobar.
    avance(Paso::Comprobar);
    let despues = leer_notas(cliente, d, Paso::Comprobar).await?;
    Ok(Subida { despues, avisos })
}

/// La acción del formulario, de dentro: la trae relativa
/// (`importar_notes_items.php?…`) o entera desde la raíz.
fn direccion_de(accion: &str) -> String {
    let a = accion.replace("&amp;", "&");
    if a.starts_with('/') {
        a
    } else {
        format!("/assignatures/{}", a.trim_start_matches("./"))
    }
}

/// Si el `alert` del paso no dice lo de siempre, un aviso (sin parar: lo que
/// manda es la tabla y las notas).
fn avisar(avisos: &mut Vec<String>, paso: &str, html: &str, esperado: &str) {
    let a = alertas(html);
    if !a.iter().any(|t| t.to_lowercase().contains(esperado)) {
        avisos.push(format!("al {paso}, Clickedu no ha dicho «{esperado}»{}", motivo_alert(html)));
    }
}

/// « (ha dicho «…»)» con el primer `alert`, corto: es un mensaje de
/// Clickedu, no lleva nombres.
fn motivo_alert(html: &str) -> String {
    match alertas(html).first() {
        Some(t) => format!(" (ha dicho «{}»)", t.chars().take(120).collect::<String>()),
        None => String::new(),
    }
}

// ───────────────────────── Leer las páginas ─────────────────────────

fn selector(s: &str) -> Selector {
    Selector::parse(s).expect("selector fijo")
}

/// Los textos de los `alert("…")` de los `<script>`: así contesta Clickedu
/// a cada paso.
pub fn alertas(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);
    let mut res = Vec::new();
    for s in doc.select(&selector("script")) {
        let codigo: String = s.text().collect();
        let mut resto = codigo.as_str();
        while let Some(i) = resto.find("alert(") {
            resto = resto[i + "alert(".len()..].trim_start();
            let Some(comilla) = resto.chars().next().filter(|c| *c == '"' || *c == '\'') else { continue };
            let cuerpo = &resto[1..];
            if let Some(fin) = cuerpo.find(comilla) {
                res.push(cuerpo[..fin].trim().to_string());
                resto = &cuerpo[fin + 1..];
            }
        }
    }
    res
}

/// Los números de lo subido, de la tabla «Importación de datos»: el `ind`
/// del botón «Eliminar» (`a.BotoEsborrarImportacio`) y el `id` de los
/// enlaces `accio=importar_notes` y `accio=veure`. Importado, una fila
/// pierde el de importar, pero no los otros.
pub fn subidos(html: &str) -> BTreeSet<String> {
    let doc = Html::parse_document(html);
    let mut res = BTreeSet::new();
    for a in doc.select(&selector("a.BotoEsborrarImportacio[ind]")) {
        res.extend(a.value().attr("ind").map(str::trim).filter(|i| !i.is_empty()).map(String::from));
    }
    for a in doc.select(&selector("a[href]")) {
        let p = fichas::parametros(a.value().attr("href").unwrap_or_default());
        if matches!(p.get("accio").map(String::as_str), Some("importar_notes" | "veure")) {
            res.extend(p.get("id").filter(|i| !i.is_empty()).cloned());
        }
    }
    res
}

/// El enlace para importar lo subido `id`, de dentro (`/assignatures/…`).
pub fn enlace_importar(html: &str, id: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let href = doc.select(&selector("a[href]")).filter_map(|a| a.value().attr("href")).find(|h| {
        let p = fichas::parametros(h);
        p.get("accio").map(String::as_str) == Some("importar_notes") && p.get("id").map(String::as_str) == Some(id)
    })?;
    Some(direccion_de(href))
}

// ───────────────────────── Comparar ─────────────────────────

/// Las notas de la app frente a las de Clickedu, por `Id` de alumno.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Comparacion {
    /// Sin nota en Clickedu.
    pub nuevas: Vec<String>,
    /// La misma nota.
    pub iguales: Vec<String>,
    /// Otra nota en Clickedu (la de allí, tal cual): **se pisaría**.
    pub cambiarian: Vec<(String, String)>,
}

/// Compara las que se van a subir con las de Clickedu. Una de Clickedu que
/// no es un número (`NP`) también cambiaría.
pub fn comparar(app: &BTreeMap<String, f64>, clickedu: &BTreeMap<String, String>) -> Comparacion {
    let mut c = Comparacion::default();
    for (alumno, &nota) in app {
        match clickedu.get(alumno) {
            None => c.nuevas.push(alumno.clone()),
            Some(t) if numero(t).is_some_and(|v| (v - nota).abs() < 1e-6) => c.iguales.push(alumno.clone()),
            Some(t) => c.cambiarian.push((alumno.clone(), t.clone())),
        }
    }
    c
}

/// Una nota de Clickedu como número: con punto o con coma.
pub fn numero(t: &str) -> Option<f64> {
    t.trim().replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickedu::falso::{self, Modo, ARCHIVO_BUENO, CONTRASENA, USUARIO};

    const IMPORTAR: &str = include_str!("../../tests/datos/clickedu/importar.html");

    /// La tabla de lo subido como la de verdad (`docs/03`): una fila sin
    /// importar y otra ya importada.
    const CON_SUBIDOS: &str = r##"<html><body><table>
<tr><td>1</td><td><a href="importar_notes_items.php?accio=importar_notes&amp;assig=7&amp;id_aval=3&amp;id=77" class="configurar Petit">i</a>
<a href="importar_notes_items.php?accio=veure&amp;assig=7&amp;id_aval=3&amp;id=77" class="veure Petit">v</a>
<a href="#a" class="esborrar Petit BotoEsborrarImportacio" cgap="7" ind="77" aval="3">e</a></td></tr>
<tr><td>2</td><td><a href="importar_notes_items.php?accio=veure&amp;assig=7&amp;id_aval=3&amp;id=78" class="veure Petit">v</a>
<a href="#a" class="esborrar Petit BotoEsborrarImportacio" cgap="7" ind="78" aval="3">e</a></td></tr>
</table><a href="importar_notes_items.php?accio=descarregar_plantilla&amp;assig=7&amp;id_aval=3">p</a></body></html>"##;

    #[test]
    fn lee_lo_subido_y_el_enlace_de_importar() {
        assert_eq!(subidos(CON_SUBIDOS), BTreeSet::from(["77".to_string(), "78".to_string()]));
        assert_eq!(
            enlace_importar(CON_SUBIDOS, "77").as_deref(),
            Some("/assignatures/importar_notes_items.php?accio=importar_notes&assig=7&id_aval=3&id=77")
        );
        // El importado ya no tiene enlace de importar.
        assert_eq!(enlace_importar(CON_SUBIDOS, "78"), None);
        // La de verdad, sin nada subido.
        assert!(subidos(IMPORTAR).is_empty());
    }

    #[test]
    fn lee_los_alert() {
        let html = r#"<html><head><script>
            alert("Archivo subido correctamente.");
            if (x) { alert( 'Otro, con "comillas"' ); }
            alert(variable);
        </script></head></html>"#;
        assert_eq!(alertas(html), ["Archivo subido correctamente.", r#"Otro, con "comillas""#]);
        assert!(alertas("<html><body>alert(\"fuera de un script\")</body></html>").is_empty());
    }

    #[test]
    fn la_accion_relativa_o_desde_la_raiz() {
        assert_eq!(
            direccion_de("importar_notes_items.php?a=1&amp;b=2"),
            "/assignatures/importar_notes_items.php?a=1&b=2"
        );
        assert_eq!(
            direccion_de("/assignatures/importar_notes_items.php?a=1"),
            "/assignatures/importar_notes_items.php?a=1"
        );
    }

    #[test]
    fn compara_nuevas_iguales_y_las_que_cambiarian() {
        let app = BTreeMap::from([("1".into(), 7.5), ("2".into(), 6.0), ("3".into(), 5.0), ("4".into(), 8.0)]);
        let clickedu = BTreeMap::from([
            ("2".to_string(), "6".to_string()),
            ("3".to_string(), "4,25".to_string()),
            ("4".to_string(), "NP".to_string()),
            ("9".to_string(), "10".to_string()),
        ]);
        let c = comparar(&app, &clickedu);
        assert_eq!(c.nuevas, ["1"]);
        assert_eq!(c.iguales, ["2"]);
        assert_eq!(c.cambiarian, [("3".to_string(), "4,25".to_string()), ("4".to_string(), "NP".to_string())]);
        assert_eq!(numero(" 7,5 "), Some(7.5));
    }

    // ── Contra el Clickedu falso ──

    fn destino() -> Destino {
        Destino { materia: "101".into(), evaluacion: "7".into(), item: "9001".into() }
    }

    fn entrado(modo: Modo) -> (falso::Falso, Cliente) {
        let f = falso::arrancar(modo);
        let cliente = Cliente::new(&f.base).unwrap();
        tauri::async_runtime::block_on(cliente.entrar(USUARIO, CONTRASENA, ("a.txt", ARCHIVO_BUENO))).unwrap();
        (f, cliente)
    }

    /// El Excel que sube la app, de la plantilla del falso.
    fn excel(notas: &[(&str, f64)]) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("p.xlsx");
        std::fs::write(&ruta, falso::plantilla_101()).unwrap();
        let p = crate::plantillas::leer(&ruta).unwrap();
        let puestas = p
            .alumnos
            .iter()
            .filter_map(|a| notas.iter().find(|(id, _)| *id == a.id).map(|(_, n)| (a.clave.clone(), Some(*n))))
            .collect();
        let e = crate::exportar::exportar(&p, &puestas, "p.xlsx", dir.path()).unwrap();
        std::fs::read(e.ruta).unwrap()
    }

    #[test]
    fn lee_antes_sube_importa_y_comprueba() {
        let (f, cliente) = entrado(Modo::Normal);
        let d = destino();
        let antes = tauri::async_runtime::block_on(leer_antes(&cliente, &d)).unwrap();
        assert_eq!(antes.notas, BTreeMap::from([("5001".to_string(), "7.5".to_string())]));
        assert!(antes.subidos.is_empty());

        let pasos = std::sync::Mutex::new(Vec::new());
        let contenido = excel(&[("5001", 8.25), ("5002", 6.0)]);
        let s = tauri::async_runtime::block_on(subir(&cliente, &d, &antes, ("notas.xlsx", contenido), |p| {
            pasos.lock().unwrap().push(p)
        }))
        .unwrap();
        assert_eq!(*pasos.lock().unwrap(), [Paso::Subir, Paso::Importar, Paso::Comprobar]);
        assert!(s.avisos.is_empty(), "{:?}", s.avisos);
        assert_eq!(
            s.despues,
            BTreeMap::from([("5001".to_string(), "8.25".to_string()), ("5002".to_string(), "6".to_string())])
        );
        let escritas: Vec<String> =
            f.pedidas().into_iter().filter(|p| p.contains("carregar") || p.contains("accio=importar_notes")).collect();
        assert_eq!(escritas.len(), 2, "{escritas:?}");
        assert!(escritas[0].starts_with("POST "));
    }

    #[test]
    fn con_lo_subido_de_antes_importa_solo_el_nuevo() {
        let (f, cliente) = entrado(Modo::Normal);
        let d = destino();
        for nota in [5.0, 9.0] {
            let antes = tauri::async_runtime::block_on(leer_antes(&cliente, &d)).unwrap();
            let contenido = excel(&[("5002", nota)]);
            let s = tauri::async_runtime::block_on(subir(&cliente, &d, &antes, ("n.xlsx", contenido), |_| {})).unwrap();
            assert_eq!(s.despues.get("5002").and_then(|t| numero(t)), Some(nota));
        }
        let importados: Vec<String> = f.pedidas().into_iter().filter(|p| p.contains("accio=importar_notes")).collect();
        assert_eq!(importados.len(), 2);
        assert_ne!(importados[0], importados[1]);
    }

    #[test]
    fn si_clickedu_no_importa_se_ve_al_comprobar() {
        let (_f, cliente) = entrado(Modo::NoImporta);
        let d = destino();
        let antes = tauri::async_runtime::block_on(leer_antes(&cliente, &d)).unwrap();
        let s =
            tauri::async_runtime::block_on(subir(&cliente, &d, &antes, ("n.xlsx", excel(&[("5002", 4.0)])), |_| {}))
                .unwrap();
        assert_eq!(s.avisos.len(), 1, "{:?}", s.avisos);
        assert!(s.avisos[0].contains("importación correcta"));
        // La nota no ha entrado: lo dice la comparación de quien llama.
        let c = comparar(&BTreeMap::from([("5002".to_string(), 4.0)]), &s.despues);
        assert_eq!(c.nuevas, ["5002"]);
    }

    #[test]
    fn un_item_que_ya_no_esta_no_se_sube() {
        let (f, cliente) = entrado(Modo::Normal);
        let d = Destino { item: "9999".into(), ..destino() };
        assert_eq!(tauri::async_runtime::block_on(leer_antes(&cliente, &d)).unwrap_err(), FalloSubir::SinItem);
        assert!(!f.pedidas().iter().any(|p| p.starts_with("POST /assignatures")));
    }

    #[test]
    fn si_no_sale_el_subido_se_para_antes_de_importar() {
        let (f, cliente) = entrado(Modo::NoGuardaSubido);
        let d = destino();
        let antes = tauri::async_runtime::block_on(leer_antes(&cliente, &d)).unwrap();
        let e =
            tauri::async_runtime::block_on(subir(&cliente, &d, &antes, ("n.xlsx", excel(&[("5002", 4.0)])), |_| {}))
                .unwrap_err();
        assert!(
            matches!(&e, FalloSubir::Cambiado { paso: Paso::Subir, motivo, .. } if motivo.contains("no sale")),
            "{e:?}"
        );
        assert!(!f.pedidas().iter().any(|p| p.contains("accio=importar_notes")));
    }
}
