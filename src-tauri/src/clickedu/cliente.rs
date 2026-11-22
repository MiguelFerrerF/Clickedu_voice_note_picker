//! Hablar con la web de Clickedu: entrar y comprobar que se ha entrado (H4).
//!
//! **Cómo se entra** (portado de la v1, `clickedu_client.py`):
//!
//! 1. Usuario y contraseña, por `POST` a `user.php?action=doLogin`. Si son
//!    buenos, la página que vuelve trae el formulario del archivo de
//!    seguridad con sus campos ocultos (entre ellos `id_usuari`).
//! 2. El archivo de seguridad, por `POST` *multipart* a
//!    `user.php?action=controlArxiuPas`, con esos campos.
//! 3. **Comprobar** (nuevo en la v2): pedir la página de las asignaturas. La
//!    v1 daba por entrado todo lo que no decía «INCORRECTO»; aquí, si
//!    Clickedu devuelve la página de entrar, no se ha entrado.
//!
//! Las cookies las lleva el `reqwest::Client` del [`Cliente`]: mientras el
//! cliente viva, la sesión sigue.
//!
//! **Lo que se reconoce es lo que vio la v1**: `usrKO` en la dirección,
//! «Incorrecto» en la página del usuario y «Archivo de seguridad
//! INCORRECTO» en la del archivo. Un archivo **caducado** no se distingue
//! de uno incorrecto: Clickedu contesta igual a los dos (probado el
//! 2026-11-20). Lo que no se entiende es [`Fallo::NoEntiendo`], con la
//! página, para guardarla y mirarla.
//!
//! Dentro, [`Cliente::abrir`] lee una página y, si Clickedu manda a entrar
//! (la sesión ha caducado), **vuelve a entrar solo** y la pide otra vez.

use std::sync::Mutex;
use std::time::Duration;

use reqwest::header::{ORIGIN, REFERER};
use reqwest::multipart::{Form, Part};
use scraper::{Html, Selector};
use serde::Serialize;

/// El Clickedu del colegio. La app la usa un solo colegio, como la v1.
pub const BASE: &str = "https://calasanz-pamplona-escolapiosemaus.clickedu.eu";
const ENTRAR: &str = "/user.php?action=doLogin";
const PAGINA_ENTRAR: &str = "/user.php?action=login";
const ARCHIVO: &str = "/user.php?action=controlArxiuPas";
pub const ASIGNATURAS: &str = "/go/Index.php?ctr=Subjects/Subjects";
/// Clickedu es una web para navegadores: se le habla como uno, como la v1.
const NAVEGADOR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0";

/// Por qué no se ha podido entrar. La interfaz dice qué hacer con cada uno.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Fallo {
    /// Sin red, o Clickedu no contesta.
    SinConexion { detalle: String },
    /// Clickedu contesta con un error (un 500, un 503…).
    Servidor { codigo: u16 },
    /// Usuario o contraseña incorrectos.
    Usuario,
    /// El archivo de seguridad no vale: incorrecto **o caducado**, que aún
    /// no se distinguen. Con la página, que se guarda en local: es la que
    /// hará falta para distinguirlos (H4, segunda parte).
    Archivo {
        #[serde(skip)]
        html: String,
    },
    /// La página no es la que se esperaba: Clickedu ha cambiado, o algo que
    /// la v1 no vio. `html` se guarda en local, nunca va a la interfaz.
    NoEntiendo {
        paso: String,
        #[serde(skip)]
        html: String,
    },
    /// Lo demás: el archivo no se puede leer, el Administrador de
    /// credenciales falla…
    Otro { detalle: String },
}

impl std::fmt::Display for Fallo {
    /// Para el registro (D8): sin contraseñas ni páginas.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fallo::SinConexion { detalle } => write!(f, "Clickedu no contesta: {detalle}"),
            Fallo::Servidor { codigo } => write!(f, "Clickedu ha contestado {codigo}"),
            Fallo::Usuario => write!(f, "usuario o contraseña incorrectos"),
            Fallo::Archivo { .. } => write!(f, "archivo de seguridad incorrecto o caducado"),
            Fallo::NoEntiendo { paso, .. } => write!(f, "no se entiende la página de Clickedu al {paso}"),
            Fallo::Otro { detalle } => f.write_str(detalle),
        }
    }
}

impl From<String> for Fallo {
    fn from(detalle: String) -> Self {
        Fallo::Otro { detalle }
    }
}

/// Una sesión con Clickedu.
pub struct Cliente {
    http: reqwest::Client,
    base: String,
    /// Las páginas de la última vez que se entró, paso a paso: para
    /// diagnosticar en local ([`Cliente::pasos`]).
    pasos: Mutex<Vec<Paso>>,
    /// Con lo que se entró la última vez, para volver a entrar solo si la
    /// sesión caduca ([`Cliente::abrir`]). Solo en memoria, mientras viva el
    /// cliente: lo guardado está en el Administrador de credenciales (D3).
    credenciales: Mutex<Option<Credenciales>>,
}

#[derive(Clone)]
struct Credenciales {
    usuario: String,
    contrasena: String,
    archivo: (String, Vec<u8>),
}

/// Una página de un paso de entrar: `entrar`, `archivo` o `comprobar`.
#[derive(Clone)]
pub struct Paso {
    pub paso: &'static str,
    /// La dirección final, tras las redirecciones.
    pub url: String,
    pub html: String,
}

impl Cliente {
    /// Contra el Clickedu del colegio, [`BASE`]. Los tests le pasan uno falso.
    pub fn new(base: &str) -> Result<Self, Fallo> {
        let http = reqwest::Client::builder()
            .user_agent(NAVEGADOR)
            .cookie_store(true)
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("no se puede preparar la conexión: {e}"))?;
        Ok(Self {
            http,
            base: base.trim_end_matches('/').to_string(),
            pasos: Mutex::new(Vec::new()),
            credenciales: Mutex::new(None),
        })
    }

    fn url(&self, ruta: &str) -> String {
        format!("{}{ruta}", self.base)
    }

    /// Las páginas de la última vez que se entró, paso a paso. Pueden
    /// llevar el nombre del profesor: solo para mirarlas en local.
    pub fn pasos(&self) -> Vec<Paso> {
        self.pasos.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Manda la petición del `paso` y devuelve la dirección final (tras las
    /// redirecciones) y la página.
    async fn pagina(&self, paso: &'static str, peticion: reqwest::RequestBuilder) -> Result<(String, String), Fallo> {
        let r = peticion.send().await.map_err(sin_conexion)?;
        if !r.status().is_success() {
            return Err(Fallo::Servidor { codigo: r.status().as_u16() });
        }
        let url = r.url().to_string();
        let html = r.text().await.map_err(sin_conexion)?;
        if let Ok(mut p) = self.pasos.lock() {
            p.push(Paso { paso, url: url.clone(), html: html.clone() });
        }
        Ok((url, html))
    }

    /// Entra con usuario, contraseña y el archivo de seguridad (nombre y
    /// contenido), y comprueba que se ha entrado.
    pub async fn entrar(&self, usuario: &str, contrasena: &str, archivo: (&str, &[u8])) -> Result<(), Fallo> {
        if let Ok(mut p) = self.pasos.lock() {
            p.clear();
        }
        // 1. Usuario y contraseña.
        let peticion = self
            .http
            .post(self.url(ENTRAR))
            .header(ORIGIN, &self.base)
            .header(REFERER, self.url(PAGINA_ENTRAR))
            .form(&[("username", usuario), ("password", contrasena), ("button", "Inicia")]);
        let (url, html) = self.pagina("entrar", peticion).await?;
        let campos = campos_del_formulario(&html);
        if !campos.iter().any(|(n, _)| n == "id_usuari") {
            if url.contains("usrKO") || html.to_lowercase().contains("incorrect") {
                return Err(Fallo::Usuario);
            }
            return Err(Fallo::NoEntiendo { paso: "entrar".into(), html });
        }

        // 2. El archivo de seguridad, con los campos de la página.
        let parte = Part::bytes(archivo.1.to_vec())
            .file_name(archivo.0.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| e.to_string())?;
        let formulario = campos.into_iter().fold(Form::new(), |f, (n, v)| f.text(n, v)).part("userfile", parte);
        let peticion = self
            .http
            .post(self.url(ARCHIVO))
            .header(ORIGIN, &self.base)
            .header(REFERER, self.url(ENTRAR))
            .multipart(formulario);
        let (url, html) = self.pagina("archivo", peticion).await?;
        if url.contains("usrKO") || html.to_lowercase().contains("archivo de seguridad incorrecto") {
            return Err(Fallo::Archivo { html });
        }

        // 3. Comprobarlo.
        self.comprobar().await?;
        if let Ok(mut c) = self.credenciales.lock() {
            *c = Some(Credenciales {
                usuario: usuario.to_string(),
                contrasena: contrasena.to_string(),
                archivo: (archivo.0.to_string(), archivo.1.to_vec()),
            });
        }
        Ok(())
    }

    /// Abre una página **de dentro** con `GET` y la devuelve como texto (en
    /// UTF-8 o en Latin-1, que es como llegan casi todas, [`decodificar`]).
    /// Si Clickedu manda a entrar, la sesión ha caducado: **vuelve a entrar
    /// solo**, con lo de la última vez, y la pide otra vez. Solo para leer:
    /// lo que escribe no se repite así.
    pub async fn abrir(&self, direccion: &str) -> Result<Abierta, Fallo> {
        let primera = self.abrir_una_vez(direccion).await?;
        if !es_pagina_de_entrar(&primera.url, &primera.texto) {
            return Ok(primera);
        }
        self.volver_a_entrar().await?;
        let segunda = self.abrir_una_vez(direccion).await?;
        if es_pagina_de_entrar(&segunda.url, &segunda.texto) {
            // Recién entrado y vuelve a mandar a entrar: no es la sesión.
            return Err(Fallo::NoEntiendo { paso: "abrir".into(), html: segunda.texto });
        }
        Ok(segunda)
    }

    /// Baja un fichero **de dentro** con `GET` y lo devuelve tal cual, como
    /// [`Cliente::leer`]; si en vez del fichero llega la página de entrar,
    /// **vuelve a entrar solo** y lo pide otra vez, como [`Cliente::abrir`].
    /// Lo que llegue (un Excel o no) lo juzga quien lo pide.
    pub async fn bajar(&self, direccion: &str) -> Result<Leida, Fallo> {
        let manda_a_entrar =
            |l: &Leida| !l.cuerpo.starts_with(b"PK") && es_pagina_de_entrar(&l.url, &decodificar(&l.cuerpo));
        let primera = self.leer(direccion).await?;
        if !manda_a_entrar(&primera) {
            return Ok(primera);
        }
        self.volver_a_entrar().await?;
        let segunda = self.leer(direccion).await?;
        if manda_a_entrar(&segunda) {
            return Err(Fallo::NoEntiendo { paso: "bajar".into(), html: decodificar(&segunda.cuerpo) });
        }
        Ok(segunda)
    }

    /// Vuelve a entrar con lo de la última vez: la sesión ha caducado.
    async fn volver_a_entrar(&self) -> Result<(), Fallo> {
        let credenciales = self.credenciales.lock().ok().and_then(|c| c.clone());
        let Some(c) = credenciales else {
            return Err("no se ha entrado en Clickedu".to_string().into());
        };
        self.entrar(&c.usuario, &c.contrasena, (&c.archivo.0, &c.archivo.1)).await
    }

    async fn abrir_una_vez(&self, direccion: &str) -> Result<Abierta, Fallo> {
        let leida = self.leer(direccion).await?;
        if !(200..300).contains(&leida.estado) {
            return Err(Fallo::Servidor { codigo: leida.estado });
        }
        Ok(Abierta { url: leida.url, texto: decodificar(&leida.cuerpo) })
    }

    /// Pide con `GET` una dirección **de este Clickedu** (`/ruta?…` o la
    /// dirección entera) y la devuelve tal cual, sin juzgarla: el código, el
    /// tipo y el cuerpo. Una dirección de fuera no se pide.
    pub async fn leer(&self, direccion: &str) -> Result<Leida, Fallo> {
        let url = if direccion.starts_with('/') {
            self.url(direccion)
        } else if direccion.strip_prefix(self.base.as_str()).is_some_and(|r| r.is_empty() || r.starts_with('/')) {
            direccion.to_string()
        } else {
            return Err(format!("{direccion} no es de Clickedu").into());
        };
        let r = self.http.get(url).header(REFERER, &self.base).send().await.map_err(sin_conexion)?;
        let cabecera = |nombre| r.headers().get(nombre).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
        let tipo = cabecera(reqwest::header::CONTENT_TYPE);
        let disposicion = cabecera(reqwest::header::CONTENT_DISPOSITION);
        let cookies = r
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(cookie_sin_valor)
            .collect();
        let (url, estado) = (r.url().to_string(), r.status().as_u16());
        let cuerpo = r.bytes().await.map_err(sin_conexion)?.to_vec();
        Ok(Leida { url, estado, tipo, disposicion, cookies, cuerpo })
    }

    /// Si la sesión sigue: la página de las asignaturas no manda a entrar.
    pub async fn comprobar(&self) -> Result<(), Fallo> {
        let (url, html) =
            self.pagina("comprobar", self.http.get(self.url(ASIGNATURAS)).header(REFERER, &self.base)).await?;
        if es_pagina_de_entrar(&url, &html) {
            return Err(Fallo::NoEntiendo { paso: "comprobar".into(), html });
        }
        Ok(())
    }
}

/// Lo que devuelve [`Cliente::abrir`]: una página de dentro, ya como texto.
#[derive(Debug)]
pub struct Abierta {
    /// La dirección final, tras las redirecciones.
    pub url: String,
    pub texto: String,
}

/// El cuerpo como texto: UTF-8 si lo es y, si no, **ISO-8859-15**, que es
/// lo que dice el pie de sus páginas. Casi todo Clickedu llega así sin
/// decirlo en la cabecera (270 de 288 páginas, `docs/03`): leído como
/// UTF-8, «Evaluación» saldría rota. Es Latin-1 salvo ocho letras (`€`,
/// `Š`, `Œ`…).
pub fn decodificar(cuerpo: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(cuerpo) {
        return s.to_string();
    }
    cuerpo
        .iter()
        .map(|&b| match b {
            0xA4 => '€',
            0xA6 => 'Š',
            0xA8 => 'š',
            0xB4 => 'Ž',
            0xB8 => 'ž',
            0xBC => 'Œ',
            0xBD => 'œ',
            0xBE => 'Ÿ',
            _ => b as char,
        })
        .collect()
}

/// Lo que devuelve [`Cliente::leer`].
pub struct Leida {
    /// La dirección final, tras las redirecciones.
    pub url: String,
    pub estado: u16,
    /// `Content-Type`.
    pub tipo: String,
    /// `Content-Disposition`: si viene, es un fichero para descargar.
    pub disposicion: String,
    /// Las cookies que pone, **sin su valor**: `PHPSESSID; path=/; HttpOnly`.
    pub cookies: Vec<String>,
    pub cuerpo: Vec<u8>,
}

/// Una cabecera `Set-Cookie` sin el valor, que es la sesión: el nombre y lo
/// demás.
fn cookie_sin_valor(c: &str) -> String {
    let mut partes = c.split(';').map(str::trim);
    let nombre = partes.next().unwrap_or_default().split('=').next().unwrap_or_default();
    std::iter::once(nombre).chain(partes).collect::<Vec<_>>().join("; ")
}

/// El fallo de red, con su causa: «error sending request» solo no dice nada.
fn sin_conexion(e: reqwest::Error) -> Fallo {
    let e = e.without_url();
    let mut detalle = e.to_string();
    let mut causa = std::error::Error::source(&e);
    while let Some(c) = causa {
        detalle.push_str(": ");
        detalle.push_str(&c.to_string());
        causa = c.source();
    }
    Fallo::SinConexion { detalle }
}

/// Los campos de los formularios de la página que se mandan tal cual: los
/// `<input>` (menos los de fichero) y los `<button>` con nombre, con su
/// valor. Como `extract_hidden_fields` de la v1: un nombre repetido se queda
/// en su sitio con el último valor.
pub(crate) fn campos_del_formulario(html: &str) -> Vec<(String, String)> {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("input[name], button[name]").expect("selector fijo");
    let mut campos: Vec<(String, String)> = Vec::new();
    for el in doc.select(&selector) {
        let e = el.value();
        if e.name() == "input" && e.attr("type").is_some_and(|t| t.eq_ignore_ascii_case("file")) {
            continue;
        }
        let nombre = e.attr("name").unwrap_or_default().to_string();
        let valor = e.attr("value").unwrap_or_default().to_string();
        match campos.iter_mut().find(|(n, _)| *n == nombre) {
            Some(c) => c.1 = valor,
            None => campos.push((nombre, valor)),
        }
    }
    campos
}

/// Clickedu ha devuelto la página de entrar: por la dirección, porque
/// manda a ella con JS (sin sesión, Clickedu contesta 200 con un
/// `alert('No ha iniciado sesión')` y `window.location` a la de entrar:
/// visto el 2026-11-20) o porque pide una contraseña.
pub fn es_pagina_de_entrar(url: &str, html: &str) -> bool {
    if url.contains("action=login") || url.contains("usrKO") {
        return true;
    }
    if html.contains("location") && html.contains("user.php?action=login") {
        return true;
    }
    let selector = Selector::parse(r#"input[type="password" i]"#).expect("selector fijo");
    Html::parse_document(html).select(&selector).next().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickedu::falso::{self, Modo, ARCHIVO_BUENO, CONTRASENA, USUARIO};

    fn entrar(modo: Modo, usuario: &str, contrasena: &str, archivo: &[u8]) -> (Result<(), Fallo>, Vec<String>) {
        let f = falso::arrancar(modo);
        let cliente = Cliente::new(&f.base).unwrap();
        let hecho =
            tauri::async_runtime::block_on(cliente.entrar(usuario, contrasena, ("archivo_seguridad_1.txt", archivo)));
        (hecho, f.pedidas())
    }

    #[test]
    fn entra_con_todo_bien() {
        let (hecho, pedidas) = entrar(Modo::Normal, USUARIO, CONTRASENA, ARCHIVO_BUENO);
        assert_eq!(hecho, Ok(()));
        assert_eq!(
            pedidas,
            [
                "POST /user.php?action=doLogin",
                "GET /user.php?action=arxiuPas",
                "POST /user.php?action=controlArxiuPas",
                "GET /index.php",
                "GET /go/Index.php?ctr=Subjects/Subjects",
            ]
        );
    }

    #[test]
    fn contrasena_mala() {
        let (hecho, pedidas) = entrar(Modo::Normal, USUARIO, "otra", ARCHIVO_BUENO);
        assert_eq!(hecho, Err(Fallo::Usuario));
        // Sin la contraseña buena no se manda el archivo.
        assert!(!pedidas.iter().any(|p| p.contains("controlArxiuPas")));
    }

    #[test]
    fn archivo_malo() {
        let (hecho, _) = entrar(Modo::Normal, USUARIO, CONTRASENA, b"OTRA-CLAVE");
        match hecho {
            Err(Fallo::Archivo { html }) => assert!(html.contains("INCORRECTO")),
            otro => panic!("{otro:?}"),
        }
    }

    #[test]
    fn clickedu_ha_cambiado() {
        let (hecho, _) = entrar(Modo::Cambiado, USUARIO, CONTRASENA, ARCHIVO_BUENO);
        match hecho {
            Err(Fallo::NoEntiendo { paso, html }) => {
                assert_eq!(paso, "entrar");
                assert!(html.contains("nuevo Clickedu"));
            }
            otro => panic!("{otro:?}"),
        }
    }

    #[test]
    fn sin_incorrecto_pero_sin_entrar() {
        // Acepta el archivo sin decir nada y no deja pasar: la v1 lo daba
        // por bueno.
        let (hecho, _) = entrar(Modo::NoDejaPasar, USUARIO, CONTRASENA, ARCHIVO_BUENO);
        assert!(matches!(hecho, Err(Fallo::NoEntiendo { ref paso, .. }) if paso == "comprobar"), "{hecho:?}");
    }

    #[test]
    fn clickedu_caido() {
        let (hecho, _) = entrar(Modo::Caido, USUARIO, CONTRASENA, ARCHIVO_BUENO);
        assert_eq!(hecho, Err(Fallo::Servidor { codigo: 503 }));
    }

    #[test]
    fn sin_conexion() {
        // Un puerto que nadie escucha.
        let puerto = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let cliente = Cliente::new(&format!("http://127.0.0.1:{puerto}")).unwrap();
        let hecho = tauri::async_runtime::block_on(cliente.entrar(USUARIO, CONTRASENA, ("a.txt", ARCHIVO_BUENO)));
        assert!(matches!(hecho, Err(Fallo::SinConexion { .. })), "{hecho:?}");
    }

    #[test]
    fn el_fallo_no_lleva_la_pagina() {
        let f = Fallo::NoEntiendo { paso: "entrar".into(), html: "<p>Lucía Castro</p>".into() };
        assert_eq!(serde_json::to_string(&f).unwrap(), r#"{"tipo":"no_entiendo","paso":"entrar"}"#);
        assert!(!f.to_string().contains("Lucía"));
        let f = Fallo::Archivo { html: "<p>Lucía Castro</p>".into() };
        assert_eq!(serde_json::to_string(&f).unwrap(), r#"{"tipo":"archivo"}"#);
    }

    #[test]
    fn campos_como_la_v1() {
        let html = r#"<form>
            <input type="hidden" name="id_usuari" value="4242">
            <input type="hidden" name="token" value="a">
            <input type="file" name="userfile">
            <input type="hidden" name="token" value="b">
            <input type="text" name="sin_valor">
            <button name="enviar" value="Enviar">Enviar</button>
            <button>sin nombre</button>
        </form>"#;
        assert_eq!(
            campos_del_formulario(html),
            [
                ("id_usuari".into(), "4242".into()),
                ("token".into(), "b".into()),
                ("sin_valor".into(), String::new()),
                ("enviar".into(), "Enviar".into()),
            ]
        );
    }

    #[test]
    fn leer_solo_dentro_de_clickedu() {
        let f = falso::arrancar(Modo::Normal);
        let cliente = Cliente::new(&f.base).unwrap();
        let leida = tauri::async_runtime::block_on(cliente.leer("/user.php?action=login")).unwrap();
        assert_eq!(leida.estado, 200);
        assert!(String::from_utf8_lossy(&leida.cuerpo).contains("password"));
        let fuera = tauri::async_runtime::block_on(cliente.leer("https://example.com/"));
        assert!(matches!(fuera, Err(Fallo::Otro { .. })), "{:?}", fuera.err());
        // Una dirección que empieza como la base pero es otro host.
        let fuera = tauri::async_runtime::block_on(cliente.leer(&format!("{}.otro.eu/", f.base)));
        assert!(fuera.is_err());
        assert_eq!(f.pedidas(), ["GET /user.php?action=login"]);
    }

    const ITEMS: &str = "/go/Index.php?ctr=Subject/Items&do=Index&id=101";

    #[test]
    fn abre_en_latin1() {
        let f = falso::arrancar(Modo::Normal);
        let cliente = Cliente::new(&f.base).unwrap();
        tauri::async_runtime::block_on(cliente.entrar(USUARIO, CONTRASENA, ("a.txt", ARCHIVO_BUENO))).unwrap();
        let abierta = tauri::async_runtime::block_on(cliente.abrir(ITEMS)).unwrap();
        assert!(abierta.texto.contains("1ª evaluación"), "{}", abierta.texto);
    }

    #[test]
    fn vuelve_a_entrar_si_caduca() {
        let f = falso::arrancar(Modo::Caduca);
        let cliente = Cliente::new(&f.base).unwrap();
        tauri::async_runtime::block_on(cliente.entrar(USUARIO, CONTRASENA, ("a.txt", ARCHIVO_BUENO))).unwrap();
        // La segunda de dentro (tras comprobar), aún con sesión.
        assert!(tauri::async_runtime::block_on(cliente.abrir(ITEMS)).is_ok());
        // La tercera, caducada: vuelve a entrar solo y la trae.
        let abierta = tauri::async_runtime::block_on(cliente.abrir(ITEMS)).unwrap();
        assert!(abierta.texto.contains("evaluación"));
        let pedidas = f.pedidas();
        let tras = &pedidas[pedidas.iter().rposition(|p| p.contains("Items")).unwrap() - 6..];
        assert_eq!(
            tras,
            [
                "GET /go/Index.php?ctr=Subject/Items&do=Index&id=101",
                "POST /user.php?action=doLogin",
                "GET /user.php?action=arxiuPas",
                "POST /user.php?action=controlArxiuPas",
                "GET /index.php",
                "GET /go/Index.php?ctr=Subjects/Subjects",
                "GET /go/Index.php?ctr=Subject/Items&do=Index&id=101",
            ]
        );
    }

    #[test]
    fn sin_haber_entrado_no_entra_solo() {
        let f = falso::arrancar(Modo::Normal);
        let cliente = Cliente::new(&f.base).unwrap();
        let hecho = tauri::async_runtime::block_on(cliente.abrir(ITEMS));
        assert!(matches!(hecho, Err(Fallo::Otro { .. })), "{hecho:?}");
        assert_eq!(f.pedidas(), [format!("GET {ITEMS}")]);
    }

    #[test]
    fn decodifica_iso_8859_15() {
        assert_eq!(decodificar("Evaluación".as_bytes()), "Evaluación");
        assert_eq!(decodificar(b"Evaluaci\xf3n 1\xaa \xa4 \xbd"), "Evaluación 1ª € œ");
    }

    #[test]
    fn la_cookie_sin_su_valor() {
        assert_eq!(cookie_sin_valor("PHPSESSID=abc123; path=/; HttpOnly"), "PHPSESSID; path=/; HttpOnly");
        assert_eq!(cookie_sin_valor("x=1"), "x");
    }

    #[test]
    fn la_pagina_de_entrar() {
        assert!(es_pagina_de_entrar("https://x/user.php?action=login", ""));
        assert!(es_pagina_de_entrar("https://x/otra", r#"<input type="PASSWORD" name="p">"#));
        assert!(!es_pagina_de_entrar("https://x/go/Index.php", "<div class='bloc-assignatura'></div>"));
        // La de verdad sin sesión, tal cual.
        let sin_sesion = "<html><body class='home'><script>\nalert('No ha iniciado sesión');\n\
            window.location.href='https://colegio.clickedu.eu/user.php?action=login';\n</script></body></html>";
        assert!(es_pagina_de_entrar("https://colegio.clickedu.eu/go/Index.php?ctr=Subjects/Subjects", sin_sesion));
    }
}
