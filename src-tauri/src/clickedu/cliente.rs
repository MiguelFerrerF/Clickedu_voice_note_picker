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
//! INCORRECTO» en la del archivo. Un archivo **caducado** aún no se
//! distingue de uno incorrecto (falta ver qué responde Clickedu, H4 segunda
//! parte): no se adivina. Lo que no se entiende es [`Fallo::NoEntiendo`],
//! con la página, para guardarla y mirarla.

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
pub(crate) const ASIGNATURAS: &str = "/go/Index.php?ctr=Subjects/Subjects";
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
        Ok(Self { http, base: base.trim_end_matches('/').to_string() })
    }

    fn url(&self, ruta: &str) -> String {
        format!("{}{ruta}", self.base)
    }

    /// Manda la petición y devuelve la dirección final (tras las
    /// redirecciones) y la página.
    async fn pagina(&self, peticion: reqwest::RequestBuilder) -> Result<(String, String), Fallo> {
        let r = peticion.send().await.map_err(sin_conexion)?;
        if !r.status().is_success() {
            return Err(Fallo::Servidor { codigo: r.status().as_u16() });
        }
        let url = r.url().to_string();
        let html = r.text().await.map_err(sin_conexion)?;
        Ok((url, html))
    }

    /// Entra con usuario, contraseña y el archivo de seguridad (nombre y
    /// contenido), y comprueba que se ha entrado.
    pub async fn entrar(&self, usuario: &str, contrasena: &str, archivo: (&str, &[u8])) -> Result<(), Fallo> {
        // 1. Usuario y contraseña.
        let peticion = self
            .http
            .post(self.url(ENTRAR))
            .header(ORIGIN, &self.base)
            .header(REFERER, self.url(PAGINA_ENTRAR))
            .form(&[("username", usuario), ("password", contrasena), ("button", "Inicia")]);
        let (url, html) = self.pagina(peticion).await?;
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
        let (url, html) = self.pagina(peticion).await?;
        if url.contains("usrKO") || html.to_lowercase().contains("archivo de seguridad incorrecto") {
            return Err(Fallo::Archivo { html });
        }

        // 3. Comprobarlo.
        self.comprobar().await
    }

    /// Si la sesión sigue: la página de las asignaturas no manda a entrar.
    pub async fn comprobar(&self) -> Result<(), Fallo> {
        let (url, html) = self.pagina(self.http.get(self.url(ASIGNATURAS)).header(REFERER, &self.base)).await?;
        if es_pagina_de_entrar(&url, &html) {
            return Err(Fallo::NoEntiendo { paso: "comprobar".into(), html });
        }
        Ok(())
    }
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

/// Clickedu ha devuelto la página de entrar: por la dirección o porque pide
/// una contraseña.
pub(crate) fn es_pagina_de_entrar(url: &str, html: &str) -> bool {
    if url.contains("action=login") || url.contains("usrKO") {
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
    fn la_pagina_de_entrar() {
        assert!(es_pagina_de_entrar("https://x/user.php?action=login", ""));
        assert!(es_pagina_de_entrar("https://x/otra", r#"<input type="PASSWORD" name="p">"#));
        assert!(!es_pagina_de_entrar("https://x/go/Index.php", "<div class='bloc-assignatura'></div>"));
    }
}
