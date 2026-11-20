//! Investigar Clickedu **solo leyendo** (H4, antes de la segunda parte):
//! entra con la cuenta guardada y recorre la web desde la página de inicio
//! para hacer el mapa de `docs/03-clickedu.md`.
//!
//!     cargo run --example explorar_clickedu [-- --paginas 300 --profundidad 4 --muestras 2
//!                                              --permitir <verbo> --solo <trozo,…>
//!                                              --sesion <minutos,…>]
//!
//! - **Solo leer.** Solo `GET` (el único `POST` es el de entrar) y solo el
//!   host del colegio. Se siguen los enlaces sin verbo o con un verbo de
//!   leer (`do=Index`, `do=See`…); lo demás no se abre y se apunta como «lo
//!   que se puede hacer». Ninguna palabra de escribir (`esborrar`,
//!   `importar`, `edit`, `guardar`…) en la dirección, ni en la ruta ni en
//!   los parámetros: en la v1, importar y borrar un fichero subido eran
//!   **`GET`**. Las únicas excepciones, las dos páginas de leer que ya usaba
//!   la v1 ([`DE_LA_V1`]). Ningún formulario se manda.
//! - **Despacio y con tope**: una petición por segundo, `--paginas` y
//!   `--profundidad`, y `--muestras` direcciones de cada forma (dos
//!   asignaturas, no todas). Si Clickedu manda a entrar, se para y se dice.
//! - **Lo crudo, solo en local**, en
//!   `%APPDATA%\es.gestornotas.app\clickedu\exploracion\`: las páginas tal
//!   cual (`paginas\`) y los Excel que se bajen (`descargas\`, para
//!   `revisar_plantillas`). Llevan nombres de alumnos: al acabar, o se
//!   borra o se mueve a `clickedu\capturas\<fecha>\` (de ahí salen, ya
//!   anonimizadas, las páginas de los tests); **nunca dentro del repo**.
//!   Cada vuelta vacía `exploracion\`. Lo demás que se baja no se guarda.
//! - **Lo que se lee fuera es `informe.md`**, en esa carpeta, sin nombres:
//!   las direcciones con los números tapados (`id=▢`), los textos con solo
//!   las palabras de la interfaz ([`LETREROS`]; lo demás, `▢`, y siempre
//!   `▢` los nombres de las plantillas de la carpeta), los enlaces por
//!   forma, los formularios, las tablas y las clases CSS.
//!
//! `--permitir` añade un verbo de leer que no está en la lista, tras verlo
//! en un informe. `--solo` abre solo las direcciones con uno de esos trozos
//! (las semillas también: pon el de la página de materias). `--sesion`, al
//! acabar, deja la sesión quieta cada una de esas pausas y la comprueba tras
//! cada una (comprobar la renueva), hasta perderla: cuánto dura parada.
//!
//! El informe trae también **los ids**: cuántos valores distintos de cada
//! clase (`alum=`, las opciones de los `<select>`, los `Id` de las
//! plantillas…), qué clase cae en cada número de `evalObject_▢_▢_▢` y los
//! cruces; solo recuentos. Lo que vuelve en JSON, solo su forma.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gestor_de_notas_lib::ajustes;
use gestor_de_notas_lib::clickedu::cliente::{self, Cliente, Leida};
use gestor_de_notas_lib::clickedu::cuenta::{self, LlaveroWindows};
use gestor_de_notas_lib::clickedu::tapar::{recortar, trozos, Tapar, PARAMETROS_DE_VERBO};
use gestor_de_notas_lib::plantillas;
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};

/// Los verbos de leer: un enlace con uno de estos (o sin verbo) se abre.
#[rustfmt::skip]
const VERBOS_DE_LEER: &[&str] = &[
    "index", "see", "view", "show", "list", "ver", "veure", "listar", "llistar", "llista", "lista", "llistat",
    "listado", "mostrar", "consultar", "consulta", "detail", "details", "detall", "detalle", "info", "search",
    "cercar", "buscar",
];

/// Palabras de escribir: si una sale en la dirección (ruta, nombres o
/// valores de parámetros), no se abre. Por el principio de la palabra…
#[rustfmt::skip]
const PROHIBIDAS_PREFIJO: &[&str] = &[
    "borr", "esborr", "elimin", "delet", "remov", "suprim", "guard", "desar", "import", "carreg", "cargar",
    "upload", "pujar", "subir", "enviar", "envia", "envio", "send", "logout", "logoff", "sortir", "salir",
    "desconn", "edit", "modif", "updat", "actualitz", "actualiz", "insert", "crear", "create", "afegi", "anadi",
    "añadi", "nuevo", "nueva", "canvi", "cambi", "chang", "bloq", "block", "unlock", "valid", "confirm", "publica",
    "publish", "tanca", "cerra", "close", "reset", "duplic", "copia", "copiar", "mover", "firma", "tramit",
    "accept", "acept", "rebutj", "rechaz", "reject", "aprova", "aprob", "approv", "notifica", "sincro", "generar",
    "generate", "calcul", "recalc", "activar", "desactiv", "enable", "disable", "assignar", "asignar", "escriu",
    "escrib", "write", "neteja", "limpi", "purge", "restaur", "restor", "desfer", "deshac", "inscri", "matricul",
    "pagar", "comprar", "votar", "marcar", "desmarc", "ordenar", "reorden", "mail", "correu", "login", "token",
    "csrf", "nonce",
    // Lo que hace algo al abrirlo aunque no lo diga (visto en el menú el
    // 2026-11-20): AJAX (`ajax_assistencia_treballadors.php` huele a fichar),
    // sincronizar, notificar, OAuth, y los mensajes, que quedan leídos.
    "ajax", "sync", "notify", "notif", "oauth", "provider", "chat", "messag", "mensaj", "missatg", "communic",
    "comunica",
    // Datos sensibles que no hacen falta para las notas.
    "medica", "incident", "incidenc", "behaviour", "salut", "health",
    // Abiertas en la primera pasada sin hacer falta, y `substituir` sin poder
    // asegurar que solo lee: fuera.
    "substitu", "assistenc", "sessio", "fitxa",
];
/// Parámetros que cambian una preferencia de la cuenta (cómo se ven las
/// materias, el idioma, el orden): no se abren.
#[rustfmt::skip]
const PARAMETROS_DE_PREFERENCIA: &[&str] = &[
    "layout", "lang", "idioma", "selector_idioma", "theme", "skin", "order", "ordre", "orden", "sort",
];

/// … y palabras cortas, enteras.
#[rustfmt::skip]
const PROHIBIDAS_ENTERAS: &[&str] = &[
    "set", "put", "add", "new", "nou", "mou", "post", "del", "save", "copy", "move", "sign", "lock", "exit", "vot",
    "sms",
];

/// Las páginas de leer que ya pedía la v1 con `GET`, aunque su ruta diga
/// «importar»: la lista de ficheros subidos y bajar la plantilla. Ruta,
/// parámetros exactos y el valor de `accio`, si lo lleva.
const DE_LA_V1: &[(&str, &[&str], Option<&str>)] = &[
    ("/assignatures/importar_notes_items.php", &["assig", "id_aval"], None),
    ("/assignatures/importar_notes_items.php", &["accio", "assig", "id_aval"], Some("descarregar_plantilla")),
];


fn main() {
    let opciones = Opciones::de(std::env::args().skip(1).collect());
    let app = dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app");
    let carpeta = app.join("clickedu");
    let exploracion = carpeta.join("exploracion");
    let _ = std::fs::remove_dir_all(&exploracion);
    std::fs::create_dir_all(exploracion.join("paginas")).expect("no se puede crear la carpeta de la exploración");

    let guardada = cuenta::leer(&carpeta, &LlaveroWindows)
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("no hay cuenta guardada: entra antes desde la pantalla Conexión");
    let archivo = guardada.archivo.as_ref().expect("falta el archivo de seguridad: elígelo en la pantalla Conexión");
    let archivo = cuenta::leer_archivo(archivo).unwrap_or_else(|e| panic!("{e}"));

    let mut tapar = Tapar::new(nombres_de_las_plantillas(&app));
    tapar.anadir(&guardada.usuario);
    println!("{} palabras de nombres de las plantillas, siempre tapadas.", tapar.cuantos());

    let cliente = Cliente::new(cliente::BASE).unwrap_or_else(|e| panic!("{e}"));
    let base = Url::parse(cliente::BASE).expect("BASE es una dirección");
    let mut e = Exploracion::new(&cliente, base, tapar, opciones, exploracion.clone());

    // La página de entrar, antes: para ver las cookies que pone.
    e.abrir(&format!("{}/user.php?action=login", cliente::BASE), 0, "antes de entrar".into(), false);
    println!("Entrando…");
    let entrada = bloquear(cliente.entrar(&guardada.usuario, &guardada.contrasena, (&archivo.0, &archivo.1)));
    e.apuntar_pasos();
    match entrada {
        Ok(()) => {
            e.dentro_desde = Some(Instant::now());
            println!("Dentro. Explorando, una página por segundo…");
            // El inicio de verdad, al que manda `controlArxiuPas`.
            for semilla in [cliente::ASIGNATURAS, "/sumari/index.php"] {
                e.encolar(&format!("{}{semilla}", cliente::BASE), 0, "semilla".into());
            }
            e.recorrer();
            if !e.opciones.sesion.is_empty() {
                // El informe del recorrido ya, sin esperar a las pausas; al
                // acabar se reescribe con la sesión.
                escribir_informe(&e, &exploracion);
                e.medir_sesion();
            }
        }
        Err(f) => e.parada = Some(format!("**No se ha podido entrar**: {f}.")),
    }

    escribir_informe(&e, &exploracion);
    println!("Lo crudo (CON nombres; bórralo o muévelo a capturas\\, fuera del repo): {}", exploracion.display());
}

fn escribir_informe(e: &Exploracion, carpeta: &Path) {
    let ruta = carpeta.join("informe.md");
    std::fs::write(&ruta, e.informe()).expect("no se puede escribir el informe");
    println!("\n{} páginas abiertas. {}", e.paginas.len(), e.parada.as_deref().unwrap_or("Recorrido acabado."));
    println!("Informe (sin nombres): {}", ruta.display());
}

fn bloquear<F: std::future::Future>(f: F) -> F::Output {
    tauri::async_runtime::block_on(f)
}

struct Opciones {
    paginas: usize,
    profundidad: usize,
    muestras: usize,
    permitidos: Vec<String>,
    /// Pausas, en minutos, para medir cuánto aguanta la sesión quieta.
    sesion: Vec<u64>,
    /// Si no está vacío, solo se abren las direcciones con uno de estos
    /// trozos.
    solo: Vec<String>,
}

impl Opciones {
    fn de(args: Vec<String>) -> Self {
        let mut o = Opciones {
            paginas: 300,
            profundidad: 4,
            muestras: 2,
            permitidos: Vec::new(),
            sesion: Vec::new(),
            solo: Vec::new(),
        };
        let mut it = args.into_iter();
        while let Some(a) = it.next() {
            let mut valor = || it.next().unwrap_or_else(|| panic!("falta el valor de {a}"));
            match a.as_str() {
                "--paginas" => o.paginas = valor().parse().expect("--paginas es un número"),
                "--profundidad" => o.profundidad = valor().parse().expect("--profundidad es un número"),
                "--muestras" => o.muestras = valor().parse().expect("--muestras es un número"),
                "--permitir" => o.permitidos.push(valor().to_lowercase()),
                "--sesion" => {
                    o.sesion =
                        valor().split(',').map(|m| m.trim().parse().expect("--sesion: minutos con comas")).collect()
                }
                "--solo" => o.solo = valor().split(',').map(|t| t.trim().to_string()).collect(),
                otro => panic!("no conozco {otro}"),
            }
        }
        o
    }
}

/// Las palabras de los nombres de los alumnos de las plantillas de la
/// carpeta de los Ajustes: en el informe salen siempre tapadas.
fn nombres_de_las_plantillas(app: &Path) -> Vec<String> {
    let Some(documentos) = dirs::document_dir() else { return Vec::new() };
    let Ok(a) = ajustes::leer(&app.join("ajustes.json")) else { return Vec::new() };
    let carpeta = PathBuf::from(ajustes::carpeta_plantillas(&a, &documentos).ruta);
    let Ok(ficheros) = std::fs::read_dir(&carpeta) else { return Vec::new() };
    ficheros
        .filter_map(|f| f.ok().map(|f| f.path()))
        .filter(|p| plantillas::es_plantilla(&p.file_name().unwrap_or_default().to_string_lossy()))
        .filter_map(|p| plantillas::leer(&p).ok())
        .flat_map(|p| p.alumnos.into_iter().map(|a| a.nombre))
        .collect()
}

// ───────────────────────── Solo leer ─────────────────────────

/// Si se puede abrir `url` sin cambiar nada en Clickedu. `Err` dice por qué
/// no, para el informe.
fn se_puede_abrir(url: &Url, base: &Url, permitidos: &[String]) -> Result<(), String> {
    if url.scheme() != base.scheme() || url.host_str() != base.host_str() {
        return Err("fuera de Clickedu".into());
    }
    let pares: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    if es_de_la_v1(url.path(), &pares) {
        return Ok(());
    }
    if url.path().eq_ignore_ascii_case("/user.php") {
        return Err("la cuenta (user.php)".into());
    }
    // `/` lleva a la página de entrar, y esa **cierra la sesión** (visto el
    // 2026-11-20); `/index.php`, por si acaso.
    if matches!(url.path(), "/" | "/index.php" | "") {
        return Err("lleva a la página de entrar, que cierra la sesión".into());
    }
    if let Some((k, _)) = pares.iter().find(|(k, _)| PARAMETROS_DE_PREFERENCIA.contains(&k.to_lowercase().as_str())) {
        return Err(format!("cambia una preferencia ({k}=)"));
    }
    for (k, v) in &pares {
        let kl = k.to_lowercase();
        if PARAMETROS_DE_VERBO.contains(&kl.as_str()) {
            let vl = v.to_lowercase();
            if !VERBOS_DE_LEER.contains(&vl.as_str()) && !permitidos.contains(&vl) {
                return Err(format!("verbo {k}={v}"));
            }
        }
    }
    let palabras = std::iter::once(url.path()).chain(pares.iter().flat_map(|(k, v)| [k.as_str(), v.as_str()]));
    for p in palabras {
        for (trozo, letras) in trozos(p) {
            let t = trozo.to_lowercase();
            if letras
                && (PROHIBIDAS_ENTERAS.contains(&t.as_str()) || PROHIBIDAS_PREFIJO.iter().any(|x| t.starts_with(x)))
            {
                return Err(format!("«{t}» en la dirección"));
            }
        }
    }
    Ok(())
}

fn es_de_la_v1(ruta: &str, pares: &[(String, String)]) -> bool {
    let mut nombres: Vec<&str> = pares.iter().map(|(k, _)| k.as_str()).collect();
    nombres.sort();
    let accio = pares.iter().find(|(k, _)| k == "accio").map(|(_, v)| v.as_str());
    DE_LA_V1.iter().any(|(r, params, a)| {
        ruta == *r && nombres == *params && accio == *a && pares.iter().all(|(k, v)| k == "accio" || es_numero(v))
    })
}

fn es_numero(v: &str) -> bool {
    !v.is_empty() && v.chars().all(|c| c.is_ascii_digit())
}

// ───────────────────────── Recorrer ─────────────────────────

/// Lo que se apunta de cada página abierta.
#[derive(Default)]
struct Pagina {
    n: usize,
    profundidad: usize,
    forma: String,
    /// Tras las redirecciones, si cambia.
    final_: Option<String>,
    origen: String,
    estado: u16,
    tipo: String,
    bytes: usize,
    codificacion: &'static str,
    lang: String,
    titulo: String,
    encabezados: Vec<String>,
    /// Patrón → (cuántos, texto de ejemplo, de dónde: `a`, `js`, `frame`).
    enlaces: BTreeMap<String, (usize, String, BTreeSet<&'static str>)>,
    formularios: Vec<String>,
    /// Cada `<select>` distinto, con cuántas veces sale.
    selects: Vec<(String, usize)>,
    /// Lo que hacen los scripts sueltos: `alert('…')` y a dónde mandan.
    scripts: Vec<String>,
    tablas: Vec<String>,
    clases: Vec<(String, usize)>,
    cookies: Vec<String>,
    nota: Option<String>,
}

/// Lo que se sabe de cada forma de dirección en todo el recorrido.
#[derive(Default)]
struct Patron {
    /// En cuántas páginas sale.
    veces: usize,
    /// Las páginas abiertas con esta forma.
    abiertas: Vec<usize>,
    /// Por qué no se ha abierto (la primera razón).
    no: Option<String>,
    texto: String,
}

struct Exploracion<'a> {
    cliente: &'a Cliente,
    base: Url,
    tapar: Tapar,
    opciones: Opciones,
    carpeta: PathBuf,
    cola: VecDeque<(Url, usize, String)>,
    vistas: HashSet<String>,
    muestras: HashMap<String, usize>,
    patrones: BTreeMap<String, Patron>,
    paginas: Vec<Pagina>,
    ultima: Option<Instant>,
    dentro_desde: Option<Instant>,
    parada: Option<String>,
    sesion: Vec<String>,
    descargas: usize,
    /// Los valores vistos de cada clase de id (`alum=` en un enlace, las
    /// opciones de un `<select>`, los `Id` de la plantilla…): para saber qué
    /// es cada número de `evalObject_▢_▢_▢`. Solo en memoria; al informe
    /// van recuentos.
    conjuntos: BTreeMap<String, HashSet<String>>,
    /// De cada cuadrícula de notas: la página, los tres números de cada
    /// `evalObject_` y cuántos tienen nota.
    cuadriculas: Vec<(usize, Vec<[String; 3]>, usize)>,
}

impl<'a> Exploracion<'a> {
    fn new(cliente: &'a Cliente, base: Url, tapar: Tapar, opciones: Opciones, carpeta: PathBuf) -> Self {
        Self {
            cliente,
            base,
            tapar,
            opciones,
            carpeta,
            cola: VecDeque::new(),
            vistas: HashSet::new(),
            muestras: HashMap::new(),
            patrones: BTreeMap::new(),
            paginas: Vec::new(),
            ultima: None,
            dentro_desde: None,
            parada: None,
            sesion: Vec::new(),
            descargas: 0,
            conjuntos: BTreeMap::new(),
            cuadriculas: Vec::new(),
        }
    }

    /// Apunta el enlace y lo pone en la cola si se puede abrir y su forma no
    /// tiene ya bastantes muestras.
    fn encolar(&mut self, direccion: &str, profundidad: usize, origen: String) {
        let Ok(mut url) = Url::parse(direccion) else { return };
        url.set_fragment(None);
        let patron = self.tapar.url(&url, false);
        let p = self.patrones.entry(patron.clone()).or_default();
        p.veces += 1;
        if let Err(por_que) = se_puede_abrir(&url, &self.base, &self.opciones.permitidos) {
            p.no.get_or_insert(por_que);
            return;
        }
        if !self.opciones.solo.is_empty() && !self.opciones.solo.iter().any(|t| url.as_str().contains(t.as_str())) {
            p.no.get_or_insert("fuera de --solo".into());
            return;
        }
        if url.query_pairs().any(|(_, v)| v.is_empty()) && url.as_str().ends_with('=') {
            p.no.get_or_insert("incompleta (se completa en JS)".into());
            return;
        }
        if !self.vistas.insert(url.as_str().to_string()) {
            return;
        }
        let n = self.muestras.entry(patron).or_default();
        if *n >= self.opciones.muestras {
            p.no.get_or_insert("ya hay bastantes de esta forma".into());
            return;
        }
        if profundidad > self.opciones.profundidad {
            p.no.get_or_insert("demasiado hondo".into());
            return;
        }
        *n += 1;
        self.cola.push_back((url, profundidad, origen));
    }

    fn recorrer(&mut self) {
        while let Some((url, profundidad, origen)) = self.cola.pop_front() {
            if self.paginas.len() >= self.opciones.paginas {
                self.parada = Some(format!("Parado en el tope de {} páginas.", self.opciones.paginas));
                return;
            }
            if !self.abrir(url.as_str(), profundidad, origen, true) {
                return;
            }
        }
    }

    /// Una petición por segundo.
    fn esperar_turno(&mut self) {
        if let Some(u) = self.ultima {
            let pasado = u.elapsed();
            if pasado < Duration::from_secs(1) {
                std::thread::sleep(Duration::from_secs(1) - pasado);
            }
        }
        self.ultima = Some(Instant::now());
    }

    /// Abre una página y la apunta; con `seguir`, encola sus enlaces.
    /// `false` si hay que parar.
    fn abrir(&mut self, direccion: &str, profundidad: usize, origen: String, seguir: bool) -> bool {
        self.esperar_turno();
        let url = Url::parse(direccion).expect("dirección ya comprobada");
        let n = self.paginas.len() + 1;
        let mut pagina = Pagina { n, profundidad, forma: self.tapar.url(&url, true), origen, ..Default::default() };
        print!("{n:>4} {}", pagina.forma);
        let leida = match bloquear(self.cliente.leer(direccion)) {
            Ok(l) => l,
            Err(f) => {
                println!("  ✗ {f}");
                pagina.nota = Some(format!("no se ha podido abrir: {f}"));
                self.paginas.push(pagina);
                return true;
            }
        };
        let Leida { url: final_, estado, tipo, disposicion, cookies, cuerpo } = leida;
        let final_ = Url::parse(&final_).expect("reqwest da direcciones válidas");
        let forma_final = self.tapar.url(&final_, true);
        if forma_final != pagina.forma {
            pagina.final_ = Some(forma_final);
        }
        pagina.estado = estado;
        pagina.tipo = tipo.clone();
        pagina.bytes = cuerpo.len();
        pagina.cookies = cookies;
        println!("  {estado} {}", tipo.split(';').next().unwrap_or_default());
        self.marcar_abierta(&url, n);

        let _ = std::fs::write(self.carpeta.join("paginas").join(format!("{n:03}.html")), &cuerpo);
        let mut indice = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.carpeta.join("paginas").join("indice.txt"))
            .expect("índice");
        let _ = std::io::Write::write_all(&mut indice, format!("{n:03}\t{direccion}\t{}\n", final_).as_bytes());

        if let Some(json) = como_json(&cuerpo) {
            pagina.nota = Some(format!("JSON: {}", forma_json(&json, &self.tapar, 0)));
            self.paginas.push(pagina);
            return true;
        }
        let es_html = tipo.to_lowercase().contains("html") || (tipo.is_empty() && cuerpo.starts_with(b"<"));
        if !es_html || !disposicion.is_empty() {
            pagina.nota = Some(self.descarga(&tipo, &disposicion, &cuerpo));
            self.paginas.push(pagina);
            return true;
        }
        if final_.host_str() != self.base.host_str() {
            pagina.nota = Some("ha llevado fuera de Clickedu: no se mira".into());
            self.paginas.push(pagina);
            return true;
        }

        let (html, codificacion) = decodificar(&cuerpo);
        pagina.codificacion = codificacion;
        let mut sigue = true;
        if self.dentro_desde.is_some() && cliente::es_pagina_de_entrar(final_.as_str(), &html) {
            // ¿Se ha perdido la sesión, o es una página con una contraseña?
            self.esperar_turno();
            match bloquear(self.cliente.comprobar()) {
                Ok(()) => {
                    pagina.nota = Some("parece la de entrar, pero la sesión sigue: no se siguen sus enlaces".into())
                }
                Err(_) => {
                    let tras = self.dentro_desde.map(|d| d.elapsed().as_secs()).unwrap_or_default();
                    self.parada = Some(format!(
                        "**La sesión se ha perdido** al abrir la página {n} ({}), a los {tras} s de entrar.",
                        pagina.forma
                    ));
                    sigue = false;
                }
            }
            mirar(&self.tapar, &html, &final_, &mut pagina);
            self.paginas.push(pagina);
            return sigue;
        }

        let enlaces = mirar(&self.tapar, &html, &final_, &mut pagina);
        self.apuntar_ids(n, &html, &enlaces);
        let origen = format!("enlace de la página {n}");
        for direccion in enlaces {
            if seguir {
                self.encolar(&direccion, profundidad + 1, origen.clone());
            } else if let Ok(u) = Url::parse(&direccion) {
                self.patrones.entry(self.tapar.url(&u, false)).or_default().veces += 1;
            }
        }
        // Los textos de ejemplo de cada forma, para el mapa.
        for (patron, (_, texto, _)) in &pagina.enlaces {
            let p = self.patrones.entry(patron.clone()).or_default();
            if p.texto.is_empty() {
                p.texto = texto.clone();
            }
        }
        // Las páginas de la v1 que no salen como enlace: con el id de una
        // asignatura y una evaluación de su `<select name="trm">`.
        if let Some(asig) = asignatura_de_items(&final_) {
            if let Some(trm) = primera_evaluacion(&html) {
                let b = cliente::BASE;
                let lista = format!("{b}/assignatures/importar_notes_items.php?assig={asig}&id_aval={trm}");
                let plantilla = format!(
                    "{b}/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig={asig}&id_aval={trm}"
                );
                self.encolar(&lista, profundidad + 1, format!("de la v1, con la página {n}"));
                self.encolar(&plantilla, profundidad + 1, format!("de la v1, con la página {n}"));
            }
        }
        self.paginas.push(pagina);
        true
    }

    /// Las páginas de cada paso de entrar, como una página más (sin seguir
    /// sus enlaces): lo que contesta Clickedu al entrar.
    fn apuntar_pasos(&mut self) {
        for paso in self.cliente.pasos() {
            let n = self.paginas.len() + 1;
            let Ok(url) = Url::parse(&paso.url) else { continue };
            let mut pagina = Pagina {
                n,
                forma: self.tapar.url(&url, true),
                origen: format!("paso «{}» de entrar", paso.paso),
                estado: 200,
                tipo: "text/html".into(),
                bytes: paso.html.len(),
                codificacion: "(la de reqwest)",
                ..Default::default()
            };
            println!("{n:>4} {}  (paso «{}» de entrar)", pagina.forma, paso.paso);
            let _ = std::fs::write(self.carpeta.join("paginas").join(format!("{n:03}-{}.html", paso.paso)), &paso.html);
            if cliente::es_pagina_de_entrar(&paso.url, &paso.html) {
                pagina.nota = Some("es (o manda a) la página de entrar".into());
            }
            mirar(&self.tapar, &paso.html, &url, &mut pagina);
            self.paginas.push(pagina);
        }
    }

    /// Los ids de la página: los de los enlaces, las opciones de los
    /// `<select>` de ítems y evaluaciones, y los `evalObject_`.
    fn apuntar_ids(&mut self, n: usize, html: &str, enlaces: &[String]) {
        for d in enlaces {
            let Ok(u) = Url::parse(d) else { continue };
            for (k, v) in u.query_pairs() {
                if PARAMETROS_DE_ID.contains(&k.as_ref()) && !v.is_empty() {
                    self.conjuntos.entry(format!("{k}= en un enlace")).or_default().insert(v.into_owned());
                }
            }
        }
        let doc = Html::parse_document(html);
        for nombre in ["id_criteri_avaluacio", "trm"] {
            for o in doc.select(&selector(&format!(r#"select[name="{nombre}"] option[value]"#))) {
                let v = o.value().attr("value").unwrap_or_default();
                if !v.is_empty() {
                    self.conjuntos.entry(format!("opción de <select name={nombre}>")).or_default().insert(v.into());
                }
            }
        }
        let mut trios = Vec::new();
        let mut con_nota = 0;
        for i in doc.select(&selector(r#"input[name^="evalObject_"]"#)) {
            let partes: Vec<&str> = i.value().attr("name").unwrap_or_default().split('_').collect();
            if let [_, a, b, c] = partes[..] {
                if [a, b, c].iter().all(|x| es_numero(x)) {
                    trios.push([a.to_string(), b.to_string(), c.to_string()]);
                    if i.value().attr("value").is_some_and(|v| !v.trim().is_empty()) {
                        con_nota += 1;
                    }
                }
            }
        }
        if !trios.is_empty() {
            self.cuadriculas.push((n, trios, con_nota));
        }
    }

    fn marcar_abierta(&mut self, url: &Url, n: usize) {
        let patron = self.tapar.url(url, false);
        self.patrones.entry(patron).or_default().abiertas.push(n);
    }

    /// Lo que no es una página: no se guarda, salvo un Excel (para
    /// `revisar_plantillas`, en local).
    fn descarga(&mut self, tipo: &str, disposicion: &str, cuerpo: &[u8]) -> String {
        let nombre = disposicion.to_lowercase();
        let ext = if nombre.contains(".xlsx") || tipo.contains("spreadsheetml") {
            Some("xlsx")
        } else if nombre.contains(".xls") || tipo.contains("ms-excel") {
            Some("xls")
        } else {
            None
        };
        let fichero = nombre.rsplit('.').next().filter(|e| e.len() <= 5).map(|e| e.trim_matches(['"', '\'', ' ']));
        let mut nota = format!(
            "descarga: {} , {} bytes{}",
            tipo.split(';').next().unwrap_or("sin tipo"),
            cuerpo.len(),
            fichero.map(|e| format!(", fichero .{e}")).unwrap_or_default()
        );
        if let Some(ext) = ext {
            self.descargas += 1;
            let dir = self.carpeta.join("descargas");
            let _ = std::fs::create_dir_all(&dir);
            let ruta = dir.join(format!("descarga-{}.{ext}", self.descargas));
            if std::fs::write(&ruta, cuerpo).is_ok() {
                let _ = write!(nota, "; guardado en local como descargas\\descarga-{}.{ext}", self.descargas);
                if let Ok(p) = plantillas::leer(&ruta) {
                    let ids = self.conjuntos.entry("Id de la plantilla".into()).or_default();
                    ids.extend(p.alumnos.into_iter().map(|a| a.id).filter(|id| !id.is_empty()));
                }
            }
        }
        nota
    }

    /// Deja la sesión quieta cada pausa de `--sesion` y comprueba al final
    /// de cada una (comprobar la renueva: por eso no se comprueba a
    /// menudo). Para en cuanto se pierde.
    fn medir_sesion(&mut self) {
        let desde = self.dentro_desde.expect("se ha entrado");
        for minutos in self.opciones.sesion.clone() {
            println!("Sesión quieta {minutos} min…");
            std::thread::sleep(Duration::from_secs(minutos * 60));
            let total = desde.elapsed().as_secs() / 60;
            let (linea, perdida) = match bloquear(self.cliente.comprobar()) {
                Ok(()) => (format!("quieta {minutos} min (a los {total} min de entrar): sigue"), false),
                Err(f) => (format!("quieta {minutos} min (a los {total} min de entrar): **perdida** ({f})"), true),
            };
            println!("  {linea}");
            self.sesion.push(linea);
            if perdida {
                break;
            }
        }
    }

    fn informe(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "# Exploración de Clickedu\n");
        let _ = writeln!(
            s,
            "Sin nombres: números tapados (`▢`, o `▢5` con sus cifras) y de los textos solo las palabras de la \
             interfaz. {} páginas abiertas; tope {} páginas, profundidad {}, {} muestras por forma.\n",
            self.paginas.len(),
            self.opciones.paginas,
            self.opciones.profundidad,
            self.opciones.muestras
        );
        if let Some(p) = &self.parada {
            let _ = writeln!(s, "{p}\n");
        }
        if !self.sesion.is_empty() {
            let _ = writeln!(s, "## La sesión\n");
            for l in &self.sesion {
                let _ = writeln!(s, "- {l}");
            }
            s.push('\n');
        }

        if !self.cuadriculas.is_empty() || !self.conjuntos.is_empty() {
            s.push_str(&self.informe_ids());
        }

        let _ = writeln!(s, "## Cookies\n");
        let mut cookies: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for p in &self.paginas {
            for c in &p.cookies {
                cookies.entry(c).or_default().push(p.n);
            }
        }
        for (c, ns) in cookies {
            let _ = writeln!(s, "- `{c}` (páginas {})", lista_corta(&ns));
        }

        let _ = writeln!(s, "\n## Idioma y codificación\n");
        let mut idiomas: BTreeMap<String, usize> = BTreeMap::new();
        for p in self.paginas.iter().filter(|p| !p.codificacion.is_empty()) {
            *idiomas.entry(format!("lang=«{}», {}", p.lang, p.codificacion)).or_default() += 1;
        }
        for (k, v) in idiomas {
            let _ = writeln!(s, "- {k}: {v} páginas");
        }

        let _ = writeln!(s, "\n## Todas las formas de dirección\n");
        let _ = writeln!(s, "Las vistas en cualquier página (enlaces, marcos y direcciones en JS), por ruta.\n");
        let _ = writeln!(s, "| Forma | Veces | Abierta | Si no, por qué | Texto |");
        let _ = writeln!(s, "|---|---|---|---|---|");
        for (forma, p) in &self.patrones {
            let _ = writeln!(
                s,
                "| `{}` | {} | {} | {} | {} |",
                forma.replace('|', "\\|"),
                p.veces,
                lista_corta(&p.abiertas),
                if p.abiertas.is_empty() { p.no.as_deref().unwrap_or("") } else { "" },
                p.texto.replace('|', "\\|"),
            );
        }

        let _ = writeln!(s, "\n## Página a página\n");
        for p in &self.paginas {
            let _ = writeln!(s, "### {} · `{}`\n", p.n, p.forma);
            let _ = writeln!(
                s,
                "- {} · {} · {} bytes · profundidad {} · {}",
                p.estado,
                p.tipo.split(';').next().unwrap_or_default(),
                p.bytes,
                p.profundidad,
                p.origen
            );
            if let Some(f) = &p.final_ {
                let _ = writeln!(s, "- Acaba en `{f}`");
            }
            if let Some(nota) = &p.nota {
                let _ = writeln!(s, "- **{nota}**");
            }
            if !p.titulo.is_empty() {
                let _ = writeln!(s, "- Título: «{}»", p.titulo);
            }
            if !p.encabezados.is_empty() {
                let _ = writeln!(
                    s,
                    "- Encabezados: {}",
                    p.encabezados.iter().map(|e| format!("«{e}»")).collect::<Vec<_>>().join(", ")
                );
            }
            if !p.enlaces.is_empty() {
                let _ = writeln!(s, "- Enlaces ({} formas):", p.enlaces.len());
                for (forma, (n, texto, de)) in &p.enlaces {
                    let de: Vec<&str> = de.iter().copied().collect();
                    let _ = writeln!(s, "  - `{forma}` ×{n} [{}] «{texto}»", de.join(","));
                }
            }
            for f in &p.formularios {
                let _ = writeln!(s, "- Formulario: {f}");
            }
            for f in &p.scripts {
                let _ = writeln!(s, "- Script: {f}");
            }
            for (f, n) in &p.selects {
                let veces = if *n > 1 { format!(" (×{n})") } else { String::new() };
                let _ = writeln!(s, "- Select{veces}: {f}");
            }
            for t in &p.tablas {
                let _ = writeln!(s, "- Tabla: {t}");
            }
            if !p.clases.is_empty() {
                let clases: Vec<String> = p.clases.iter().map(|(c, n)| format!("`{c}`×{n}")).collect();
                let _ = writeln!(s, "- Clases: {}", clases.join(" "));
            }
            s.push('\n');
        }
        s
    }
}

impl Exploracion<'_> {
    /// Qué es cada número de `evalObject_▢_▢_▢`, y los cruces de ids: solo
    /// recuentos.
    fn informe_ids(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "## Los ids\n");
        for (nombre, valores) in &self.conjuntos {
            let _ = writeln!(s, "- {nombre}: {} distintos", valores.len());
        }
        for (n, trios, con_nota) in &self.cuadriculas {
            let _ = writeln!(s, "\n### `evalObject_A_B_C` en la página {n}\n");
            let _ = writeln!(s, "{} campos, {con_nota} con nota.\n", trios.len());
            for (i, letra) in ["A", "B", "C"].iter().enumerate() {
                let valores: HashSet<&String> = trios.iter().map(|t| &t[i]).collect();
                let _ = write!(s, "- {letra}: {} distintos", valores.len());
                for (nombre, conjunto) in &self.conjuntos {
                    let dentro = valores.iter().filter(|v| conjunto.contains(v.as_str())).count();
                    if dentro > 0 {
                        let _ = write!(s, "; {dentro} están en «{nombre}»");
                    }
                }
                s.push('\n');
            }
        }
        let _ = writeln!(s, "\n### Cruces\n");
        let nombres: Vec<&String> = self.conjuntos.keys().collect();
        for a in &nombres {
            for b in &nombres {
                if a >= b {
                    continue;
                }
                let (ca, cb) = (&self.conjuntos[*a], &self.conjuntos[*b]);
                let comunes = ca.intersection(cb).count();
                if comunes > 0 {
                    let _ = writeln!(s, "- «{a}» ({}) y «{b}» ({}): {comunes} en común", ca.len(), cb.len());
                }
            }
        }
        s.push('\n');
        s
    }
}

/// Los parámetros de los enlaces que son ids de algo: sus valores se
/// juntan para cruzarlos.
const PARAMETROS_DE_ID: &[&str] = &["alum", "alu", "it", "trm", "id_aval", "assig", "id", "criteri", "cgap", "av"];

/// El cuerpo, si es JSON (aunque diga que es HTML).
fn como_json(cuerpo: &[u8]) -> Option<serde_json::Value> {
    let primero = cuerpo.iter().find(|b| !b.is_ascii_whitespace())?;
    if !matches!(primero, b'{' | b'[') {
        return None;
    }
    serde_json::from_slice(cuerpo).ok()
}

/// La forma de un JSON: claves, tipos y cuántos; los textos, tapados.
fn forma_json(v: &serde_json::Value, tapar: &Tapar, hondo: usize) -> String {
    use serde_json::Value;
    if hondo > 5 {
        return "…".into();
    }
    match v {
        Value::Null => "null".into(),
        Value::Bool(_) => "bool".into(),
        Value::Number(_) => "núm".into(),
        Value::String(t) if t.trim().replace(',', ".").parse::<f64>().is_ok() => "\"núm\"".into(),
        Value::String(t) => format!("\"{}\"", recortar(&tapar.texto(t), 30)),
        Value::Array(a) => match a.first() {
            None => "[]".into(),
            Some(x) => format!("[{} × {}]", a.len(), forma_json(x, tapar, hondo + 1)),
        },
        Value::Object(o) => {
            if o.keys().all(|k| es_numero(k)) {
                return match o.values().next() {
                    None => "{}".into(),
                    Some(x) => format!("{{▢ ×{}: {}}}", o.len(), forma_json(x, tapar, hondo + 1)),
                };
            }
            let mut partes: Vec<String> = o
                .iter()
                .take(15)
                .map(|(k, x)| {
                    // Una clave que parece un identificador se deja; si no, se
                    // tapa como un texto (podría ser un nombre).
                    let es_id = k.contains('_') || k.chars().next().is_some_and(|c| c.is_ascii_lowercase());
                    let k = if es_id { tapar.codigo(k) } else { tapar.texto(k) };
                    format!("{k}: {}", forma_json(x, tapar, hondo + 1))
                })
                .collect();
            if o.len() > 15 {
                partes.push(format!("…+{}", o.len() - 15));
            }
            format!("{{{}}}", partes.join(", "))
        }
    }
}

fn lista_corta(ns: &[usize]) -> String {
    let mut v: Vec<String> = ns.iter().take(8).map(|n| n.to_string()).collect();
    if ns.len() > 8 {
        v.push(format!("… ({})", ns.len()));
    }
    v.join(", ")
}

/// UTF-8, o si no lo es, Latin-1 (las webs PHP antiguas).
fn decodificar(cuerpo: &[u8]) -> (String, &'static str) {
    match std::str::from_utf8(cuerpo) {
        Ok(s) => (s.to_string(), "utf-8"),
        Err(_) => (cuerpo.iter().map(|&b| b as char).collect(), "latin-1"),
    }
}

/// `go/Index.php?ctr=Subject/Items&do=Index&id=N` → `N`.
fn asignatura_de_items(url: &Url) -> Option<String> {
    let pares: HashMap<String, String> = url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    (pares.get("ctr").map(String::as_str) == Some("Subject/Items"))
        .then(|| pares.get("id").cloned())
        .flatten()
        .filter(|v| es_numero(v))
}

/// El primer valor numérico del `<select name="trm">`.
fn primera_evaluacion(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse(r#"select[name="trm"] option"#).ok()?;
    doc.select(&sel).filter_map(|o| o.value().attr("value")).find(|v| es_numero(v)).map(str::to_string)
}

fn selector(s: &str) -> Selector {
    Selector::parse(s).expect("selector fijo")
}

fn texto_de(e: ElementRef) -> String {
    e.text().collect::<Vec<_>>().join(" ")
}

/// Mira la página: apunta en `pagina` lo del informe y devuelve las
/// direcciones (enteras) de sus enlaces, marcos y JS.
fn mirar(tapar: &Tapar, html: &str, url: &Url, pagina: &mut Pagina) -> Vec<String> {
    let doc = Html::parse_document(html);
    pagina.lang = doc
        .select(&selector("html"))
        .next()
        .and_then(|h| h.value().attr("lang"))
        .map(|l| tapar.codigo(l))
        .unwrap_or_default();
    pagina.titulo = doc.select(&selector("title")).next().map(|t| tapar.texto(&texto_de(t))).unwrap_or_default();
    pagina.encabezados = doc
        .select(&selector("h1, h2, h3, legend"))
        .take(8)
        .map(|h| tapar.texto(&texto_de(h)))
        .filter(|t| !t.is_empty())
        .collect();

    // Los enlaces, los marcos y las direcciones dentro del JS.
    let mut direcciones = Vec::new();
    let mut apuntar = |crudo: &str, texto: String, de: &'static str, direcciones: &mut Vec<String>| {
        let crudo = crudo.trim();
        if crudo.is_empty() || crudo.starts_with('#') || crudo.to_lowercase().starts_with("mailto:") {
            return;
        }
        let Ok(mut u) = url.join(crudo) else { return };
        if !matches!(u.scheme(), "http" | "https") {
            return;
        }
        u.set_fragment(None);
        let forma = if u.host_str() == url.host_str() { tapar.url(&u, false) } else { "(fuera de Clickedu)".into() };
        let e = pagina.enlaces.entry(forma).or_insert_with(|| (0, String::new(), BTreeSet::new()));
        e.0 += 1;
        if e.1.is_empty() {
            e.1 = texto;
        }
        e.2.insert(de);
        direcciones.push(u.to_string());
    };
    for a in doc.select(&selector("a[href], area[href]")) {
        let href = a.value().attr("href").unwrap_or_default();
        let texto = tapar.texto(&texto_de(a));
        if href.trim_start().to_lowercase().starts_with("javascript:") {
            for d in urls_en_js(href) {
                apuntar(&d, texto.clone(), "js", &mut direcciones);
            }
        } else {
            apuntar(href, texto, "a", &mut direcciones);
        }
    }
    for f in doc.select(&selector("iframe[src], frame[src]")) {
        apuntar(f.value().attr("src").unwrap_or_default(), String::new(), "frame", &mut direcciones);
    }
    for e in doc.select(&selector("*")) {
        for (nombre, valor) in e.value().attrs() {
            if nombre.starts_with("on") || nombre.starts_with("data-") {
                let urls = urls_en_js(valor);
                if urls.is_empty() {
                    continue;
                }
                let texto = tapar.texto(&texto_de(e));
                for d in urls {
                    apuntar(&d, texto.clone(), "js", &mut direcciones);
                }
            }
        }
    }
    for s in doc.select(&selector("script")) {
        let js = s.text().collect::<String>();
        for d in urls_en_js(&js) {
            apuntar(&d, String::new(), "js", &mut direcciones);
        }
        for aviso in avisos_en_js(&js) {
            pagina.scripts.push(format!("alert «{}»", tapar.texto(&aviso)));
        }
        if js.contains("location") {
            for d in urls_en_js(&js) {
                if let Ok(u) = url.join(&d) {
                    pagina.scripts.push(format!("location → `{}`", tapar.url(&u, true)));
                }
            }
        }
    }

    // Los formularios: no se mandan; su forma.
    for f in doc.select(&selector("form")) {
        let v = f.value();
        let accion = v.attr("action").and_then(|a| url.join(a).ok()).map(|u| tapar.url(&u, false)).unwrap_or_default();
        let mut campos: BTreeMap<String, usize> = BTreeMap::new();
        for c in f.select(&selector("input[name], textarea[name], button[name], select[name]")) {
            let tipo = match c.value().name() {
                "input" => c.value().attr("type").unwrap_or("text").to_lowercase(),
                otro => otro.to_string(),
            };
            *campos
                .entry(format!("{} ({tipo})", tapar.codigo(c.value().attr("name").unwrap_or_default())))
                .or_default() += 1;
        }
        let campos: Vec<String> = campos.into_iter().map(|(c, n)| if n > 1 { format!("{c}×{n}") } else { c }).collect();
        pagina.formularios.push(format!(
            "`{}` {} {}{} — {}",
            accion,
            v.attr("method").unwrap_or("get").to_uppercase(),
            v.attr("enctype").unwrap_or_default(),
            v.attr("id").map(|i| format!(" #{}", tapar.codigo(i))).unwrap_or_default(),
            campos.join(", ")
        ));
    }
    for sel in doc.select(&selector("select")) {
        let opciones: Vec<ElementRef> = sel.select(&selector("option")).collect();
        let mut linea = format!(
            "`{}` {} opciones",
            tapar.codigo(sel.value().attr("name").or(sel.value().attr("id")).unwrap_or("(sin nombre)")),
            opciones.len()
        );
        if let Some(js) = sel.value().attr("onchange") {
            let _ = write!(linea, ", onchange `{}`", recortar(&tapar.codigo(js), 60));
        }
        if opciones.len() <= 15 {
            let textos: Vec<String> = opciones
                .iter()
                .map(|o| {
                    format!(
                        "{}={}",
                        tapar.codigo(o.value().attr("value").unwrap_or_default()),
                        tapar.texto(&texto_de(*o))
                    )
                })
                .collect();
            let _ = write!(linea, ": {}", textos.join(" | "));
        }
        match pagina.selects.iter_mut().find(|(l, _)| *l == linea) {
            Some((_, n)) => *n += 1,
            None => pagina.selects.push((linea, 1)),
        }
    }

    // Las tablas con cabecera o con filas: la cabecera, cuántas filas y la
    // forma de la primera fila de datos.
    for t in doc.select(&selector("table")).take(30) {
        let filas: Vec<ElementRef> = t
            .select(&selector("tr"))
            .filter(|tr| tr.ancestors().filter_map(ElementRef::wrap).find(|a| a.value().name() == "table") == Some(t))
            .collect();
        let cabecera: Vec<String> = t.select(&selector("th")).take(25).map(|th| tapar.texto(&texto_de(th))).collect();
        if cabecera.is_empty() && filas.len() < 3 {
            continue;
        }
        let datos = filas.iter().find(|tr| tr.select(&selector("td")).next().is_some());
        let primera: Vec<String> = datos
            .map(|tr| tr.select(&selector("td")).take(15).map(|td| recortar(&tapar.texto(&texto_de(td)), 30)).collect())
            .unwrap_or_default();
        let mut clases: BTreeSet<String> = BTreeSet::new();
        for td in t.select(&selector("td[class]")) {
            for c in td.value().classes() {
                clases.insert(tapar.codigo(c));
            }
        }
        let mut campos: BTreeSet<String> = BTreeSet::new();
        for c in t.select(&selector("input[name], select[name], textarea[name]")) {
            campos.insert(tapar.codigo(c.value().attr("name").unwrap_or_default()));
        }
        let mut linea = format!(
            "{}{} filas; cabecera [{}]; 1.ª fila [{}]",
            t.value()
                .attr("id")
                .map(|i| format!("#{} ", tapar.codigo(i)))
                .or(t.value().attr("class").map(|c| format!(".{} ", tapar.codigo(c))))
                .unwrap_or_default(),
            filas.len(),
            cabecera.join(" | "),
            primera.join(" | ")
        );
        if !clases.is_empty() {
            let _ = write!(linea, "; td: {}", clases.into_iter().take(12).collect::<Vec<_>>().join(" "));
        }
        if !campos.is_empty() {
            let _ = write!(linea, "; campos: {}", campos.into_iter().take(12).collect::<Vec<_>>().join(" "));
        }
        pagina.tablas.push(linea);
    }

    // Las clases CSS que más se repiten (las que llevan los datos).
    let mut clases: HashMap<String, usize> = HashMap::new();
    for e in doc.select(&selector("[class]")) {
        for c in e.value().classes() {
            if c.len() <= 40 {
                *clases.entry(format!("{}.{}", e.value().name(), tapar.codigo(c))).or_default() += 1;
            }
        }
    }
    let mut clases: Vec<(String, usize)> = clases.into_iter().filter(|(_, n)| *n >= 3).collect();
    clases.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    clases.truncate(25);
    pagina.clases = clases;

    direcciones
}

/// Los textos de los `alert('…')` de un trozo de JS.
fn avisos_en_js(js: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut resto = js;
    while let Some(i) = resto.find("alert(") {
        resto = &resto[i + 6..];
        let Some(comilla) = resto.trim_start().chars().next().filter(|c| matches!(c, '\'' | '"')) else { continue };
        let dentro = resto.trim_start()[1..].to_string();
        if let Some(fin) = dentro.find(comilla) {
            res.push(dentro[..fin].to_string());
        }
    }
    res
}

/// Las direcciones `….php…` dentro de un trozo de JS: desde donde empieza
/// la dirección hasta la primera comilla, espacio o paréntesis.
fn urls_en_js(js: &str) -> Vec<String> {
    let de_url = |c: char| c.is_ascii_alphanumeric() || "_./?=&%:-~+,;".contains(c);
    let bytes: Vec<char> = js.chars().collect();
    let mut res = Vec::new();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes[i..i + 4].iter().collect::<String>().eq_ignore_ascii_case(".php") {
            let mut ini = i;
            while ini > 0 && de_url(bytes[ini - 1]) {
                ini -= 1;
            }
            let mut fin = i + 4;
            while fin < bytes.len() && de_url(bytes[fin]) {
                fin += 1;
            }
            let d: String = bytes[ini..fin].iter().collect();
            let d = d.trim_end_matches([';', ',']).to_string();
            if !d.is_empty() && !res.contains(&d) {
                res.push(d);
            }
            i = fin;
        } else {
            i += 1;
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://colegio.clickedu.eu").unwrap()
    }

    fn abrir(d: &str) -> Result<(), String> {
        se_puede_abrir(&base().join(d).unwrap(), &base(), &[])
    }

    #[test]
    fn se_abre_lo_de_leer() {
        assert_eq!(abrir("/go/Index.php?ctr=Subjects/Subjects"), Ok(()));
        assert_eq!(abrir("/go/Index.php?ctr=Subject/Items&do=Index&id=12"), Ok(()));
        assert_eq!(abrir("/go/Index.php?ctr=Subject/Item/Item&do=See&it=99"), Ok(()));
        assert_eq!(abrir("/assignatures/importar_notes_items.php?assig=1&id_aval=2"), Ok(()));
        assert_eq!(
            abrir("/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig=1&id_aval=2"),
            Ok(())
        );
        assert_eq!(abrir("/sumari/index.php"), Ok(()));
        assert_eq!(abrir("/go/Index.php?ctr=Evaluate/EvaluateTermly&id=5"), Ok(()));
        assert_eq!(abrir("/go/Index.php?ctr=Evaluate/EvaluateTermly&id=5&trm=331"), Ok(()));
    }

    #[test]
    fn no_se_abre_lo_de_escribir() {
        for d in [
            "/assignatures/importar_notes_items.php?accio=importar_notes&assig=1&id_aval=2&id=3",
            "/assignatures/importar_notes_items.php?accio=esborrar_arxiu&assig=1&id_aval=2&id=3",
            "/assignatures/importar_notes_items.php?accio=carregar_plantilla&assig=1&id_aval=2",
            // La de la v1 con un parámetro de más ya no es la de la v1.
            "/assignatures/importar_notes_items.php?assig=1&id_aval=2&id=3",
            "/assignatures/importar_notes_items.php?assig=x&id_aval=2",
            "/go/Index.php?ctr=Subject/Item/Item&do=Edit&it=99",
            "/go/Index.php?ctr=Subject/Item/Delete&it=99",
            "/go/Index.php?ctr=Subject/Items&do=saveNotes&id=1",
            "/esborrar.php?id=3",
            "/notes/guardarNotes.php",
            "/user.php?action=logout",
            "/user.php?action=login",
            "/go/Index.php?ctr=Messages&do=Index&token=abc",
            "/x.php?accio=nouItem",
            "/x.php?action=Unknown",
            "/logout.php",
            "/x.php?op=del&id=1",
            // Vistos en el menú de verdad.
            "/",
            "/index.php",
            "/includes/ajax/ajax_assistencia_treballadors.php",
            "/go/Index.php?ctr=Subjects/LmsUsersSyncMySubjects",
            "/go/Index.php?ctr=Subjects/Subjects&do=Index&layout=2",
            "/go/Index.php?ctr=Messaging/Chat",
            "/go/Index.php?ctr=Communication/Communication",
            "/sumari/notify.php?type=Formation",
            "/gestio/oauth_llistat.php",
            "/assignatures/medicacions.php",
            "/behaviour/incidents.php",
            "/user.php?action=triarArxiuPas",
            "/assignatures/substituir.php?assig=1&sessio=2",
            "/assistencia/index.php?cgap=1&sessio=2&n=3",
            "/assignatures/fitxa_alumne.php?alum=1",
        ] {
            assert!(abrir(d).is_err(), "{d} se abriría");
        }
        assert!(abrir("https://otro.eu/index.php").is_err());
        assert!(abrir("http://colegio.clickedu.eu/index.php").is_err(), "otro esquema");
    }

    #[test]
    fn permitir_un_verbo() {
        let u = base().join("/x.php?do=Calendar").unwrap();
        assert!(se_puede_abrir(&u, &base(), &[]).is_err());
        assert_eq!(se_puede_abrir(&u, &base(), &["calendar".into()]), Ok(()));
    }

    #[test]
    fn avisos_de_los_scripts() {
        let js = "alert('No ha iniciado sesión');\nif (x) alert( \"Otro\" ); alert(variable);";
        assert_eq!(avisos_en_js(js), ["No ha iniciado sesión", "Otro"]);
    }

    #[test]
    fn la_forma_de_un_json() {
        let t = Tapar::new(["Lucía García"]);
        let j: serde_json::Value = serde_json::from_str(
            r#"{"grades": {"4321": {"nota": "7,5", "name": "García, Lucía"}, "4322": {"nota": 8, "name": "Pérez, Juan"}},
                "Lucía": 1, "ok": true, "items": [1, 2, 3]}"#,
        )
        .unwrap();
        let f = forma_json(&j, &t, 0);
        assert!(f.contains("grades: {▢ ×2: {"), "{f}");
        assert!(f.contains(r#"name: "▢, ▢""#), "{f}");
        assert!(f.contains("items: [3 × núm]"), "{f}");
        for nombre in ["Lucía", "García", "Juan", "4321"] {
            assert!(!f.contains(nombre), "{nombre} en {f}");
        }
    }

    #[test]
    fn direcciones_en_js() {
        let js = "javascript:obrir('/go/Index.php?ctr=Subject/Item/Item&do=See&it=5'); window.location='importar_notes_items.php?assig=' + id;";
        assert_eq!(
            urls_en_js(js),
            ["/go/Index.php?ctr=Subject/Item/Item&do=See&it=5", "importar_notes_items.php?assig="]
        );
    }

    #[test]
    fn mira_una_pagina_sin_nombres() {
        let t = Tapar::new(["Lucía García"]);
        let html = r#"<html lang="es"><head><title>Ítems - Matemáticas</title></head><body>
            <h1>Lucía García</h1>
            <select name="trm"><option value="11">1ª Evaluación</option><option value="12">2ª Evaluación</option></select>
            <table class="items"><tr><th>Nombre</th><th>Nota</th></tr>
              <tr><td class="nom">García, Lucía</td><td class="nota"><input name="nota_4321" value="7"></td></tr>
              <tr><td class="nom">Pérez, Juan</td><td class="nota"><input name="nota_4322" value="8"></td></tr></table>
            <a class="popupEvent" href="/go/Index.php?ctr=Subject/Item/Item&do=See&it=7">Examen tema 1</a>
            <form action="/go/Index.php?ctr=Subject/Items&do=Save" method="post"><input name="nota_1"></form>
            </body></html>"#;
        let url = base().join("/go/Index.php?ctr=Subject/Items&do=Index&id=3").unwrap();
        let mut p = Pagina::default();
        let dirs = mirar(&t, html, &url, &mut p);
        assert_eq!(dirs, ["https://colegio.clickedu.eu/go/Index.php?ctr=Subject/Item/Item&do=See&it=7"]);
        assert_eq!(p.titulo, "Ítems - Matemáticas");
        assert_eq!(p.encabezados, ["▢ ▢"]);
        assert_eq!(p.selects, [("`trm` 2 opciones: ▢=1ª Evaluación | ▢=2ª Evaluación".to_string(), 1)]);
        assert_eq!(p.tablas.len(), 1);
        assert!(p.tablas[0].contains("cabecera [Nombre | Nota]"), "{}", p.tablas[0]);
        assert!(p.tablas[0].contains("1.ª fila [▢, ▢ | ]"), "{}", p.tablas[0]);
        assert!(p.tablas[0].contains("campos: nota_▢"), "{}", p.tablas[0]);
        let todo = format!("{:?} {:?} {:?} {:?}", p.tablas, p.enlaces, p.formularios, p.clases);
        for nombre in ["Lucía", "García", "Juan", "Pérez", "4321"] {
            assert!(!todo.contains(nombre), "{nombre} en {todo}");
        }
        assert_eq!(asignatura_de_items(&url).as_deref(), Some("3"));
        assert_eq!(primera_evaluacion(html).as_deref(), Some("11"));
    }
}
