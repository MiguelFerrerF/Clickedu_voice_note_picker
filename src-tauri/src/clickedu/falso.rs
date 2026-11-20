//! Un Clickedu falso para los tests (H4): un servidor HTTP en un hilo, en un
//! puerto libre de `127.0.0.1`.
//!
//! Las páginas son **inventadas**, de la forma que esperaba la v1: el
//! formulario del archivo con `id_usuari`, `usrKO` en la dirección si el
//! usuario está mal, «Archivo de seguridad INCORRECTO»… Cuando haya páginas
//! de verdad (anonimizadas, H4 segunda parte), los tests se pasan también
//! contra ellas.

use std::sync::{Arc, Mutex};

use tiny_http::{Header, Response, Server};

pub const USUARIO: &str = "profe";
pub const CONTRASENA: &str = "secreta";
pub const ARCHIVO_BUENO: &[u8] = b"CLAVE-BUENA-3062";

/// Cómo se porta el Clickedu falso.
#[derive(Clone, Copy, PartialEq)]
pub enum Modo {
    /// Como esperaba la v1.
    Normal,
    /// Tras el usuario y la contraseña, una página que no se parece a nada.
    Cambiado,
    /// Acepta el archivo sin decir «INCORRECTO», pero no deja pasar.
    NoDejaPasar,
    /// Todo, un 503.
    Caido,
}

pub struct Falso {
    /// `http://127.0.0.1:<puerto>`.
    pub base: String,
    pedidas: Arc<Mutex<Vec<String>>>,
    servidor: Arc<Server>,
}

impl Falso {
    /// Lo pedido, en orden: «POST /user.php?action=doLogin».
    pub fn pedidas(&self) -> Vec<String> {
        self.pedidas.lock().unwrap().clone()
    }
}

impl Drop for Falso {
    fn drop(&mut self) {
        self.servidor.unblock();
    }
}

const SESION: &str = "PHPSESSID=falsa";
const DENTRO: &str = "dentro=si";

const ENTRAR: &str = r#"<!DOCTYPE html><html><body>
<form action="/user.php?action=doLogin" method="post">
  <input type="text" name="username"><input type="password" name="password">
  <button name="button" value="Inicia">Inicia</button>
</form>{error}</body></html>"#;

const ARXIU: &str = r#"<!DOCTYPE html><html><body>
<form action="/user.php?action=controlArxiuPas" method="post" enctype="multipart/form-data">
  <input type="hidden" name="id_usuari" value="4242">
  <input type="hidden" name="token" value="t0k3n">
  <input type="file" name="userfile">
  <button name="enviar" value="Enviar">Enviar</button>
</form></body></html>"#;

const ARCHIVO_INCORRECTO: &str = "<!DOCTYPE html><html><body><p>Archivo de seguridad INCORRECTO</p></body></html>";
const CAMBIADO: &str = "<!DOCTYPE html><html><body><p>Bienvenido al nuevo Clickedu</p></body></html>";
const INICIO: &str = "<!DOCTYPE html><html><body><p>Inicio</p></body></html>";
const ASIGNATURAS: &str = r#"<!DOCTYPE html><html><body>
<div class="bloc-assignatura"><a href="/go/Index.php?ctr=Subject/Items&id=101">
  <div class="title">Matemáticas</div><div class="sub-title">3.º ESO A</div></a></div>
</body></html>"#;

/// Arranca el Clickedu falso. Se para al soltarlo.
pub fn arrancar(modo: Modo) -> Falso {
    let servidor = Arc::new(Server::http("127.0.0.1:0").expect("servidor falso"));
    let puerto = servidor.server_addr().to_ip().expect("puerto").port();
    let pedidas = Arc::new(Mutex::new(Vec::new()));
    let (s, p) = (servidor.clone(), pedidas.clone());
    std::thread::spawn(move || {
        for mut peticion in s.incoming_requests() {
            let metodo = peticion.method().as_str().to_string();
            let ruta = peticion.url().to_string();
            p.lock().unwrap().push(format!("{metodo} {ruta}"));
            let cookies: String = peticion
                .headers()
                .iter()
                .filter(|h| h.field.equiv("Cookie"))
                .map(|h| h.value.as_str().to_string())
                .collect::<Vec<_>>()
                .join("; ");
            let mut cuerpo = Vec::new();
            let _ = peticion.as_reader().read_to_end(&mut cuerpo);
            let cuerpo = String::from_utf8_lossy(&cuerpo).into_owned();
            let _ = peticion.respond(responder(modo, &metodo, &ruta, &cookies, &cuerpo));
        }
    });
    Falso { base: format!("http://127.0.0.1:{puerto}"), pedidas, servidor }
}

type Respuesta = Response<std::io::Cursor<Vec<u8>>>;

fn pagina(html: &str) -> Respuesta {
    Response::from_string(html).with_header(Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap())
}

fn ir_a(a: &str, cookie: Option<&str>) -> Respuesta {
    let mut r = Response::from_string("").with_status_code(302).with_header(Header::from_bytes("Location", a).unwrap());
    if let Some(c) = cookie {
        r = r.with_header(Header::from_bytes("Set-Cookie", format!("{c}; Path=/")).unwrap());
    }
    r
}

fn responder(modo: Modo, metodo: &str, ruta: &str, cookies: &str, cuerpo: &str) -> Respuesta {
    if modo == Modo::Caido {
        return pagina("Service Unavailable").with_status_code(503);
    }
    match (metodo, ruta) {
        ("POST", "/user.php?action=doLogin") => {
            let bien =
                cuerpo.contains(&format!("username={USUARIO}")) && cuerpo.contains(&format!("password={CONTRASENA}"));
            match (bien, modo) {
                (false, _) => ir_a("/user.php?action=login&error=usrKO", None),
                (true, Modo::Cambiado) => pagina(CAMBIADO),
                (true, _) => ir_a("/user.php?action=arxiuPas", Some(SESION)),
            }
        }
        ("GET", r) if r.starts_with("/user.php?action=login") => {
            let error = if r.contains("usrKO") { "<p>Usuario o contraseña Incorrecto</p>" } else { "" };
            pagina(&ENTRAR.replace("{error}", error))
        }
        ("GET", "/user.php?action=arxiuPas") if cookies.contains(SESION) => pagina(ARXIU),
        ("POST", "/user.php?action=controlArxiuPas") => {
            let campos = cookies.contains(SESION)
                && cuerpo.contains("name=\"id_usuari\"\r\n\r\n4242")
                && cuerpo.contains("name=\"token\"\r\n\r\nt0k3n")
                && cuerpo.contains("name=\"userfile\"; filename=\"");
            let archivo = cuerpo.contains(std::str::from_utf8(ARCHIVO_BUENO).unwrap());
            match (campos && archivo, modo) {
                (false, _) => pagina(ARCHIVO_INCORRECTO),
                (true, Modo::NoDejaPasar) => pagina(INICIO),
                (true, _) => ir_a("/index.php", Some(DENTRO)),
            }
        }
        ("GET", "/index.php") => pagina(INICIO),
        ("GET", "/go/Index.php?ctr=Subjects/Subjects") if cookies.contains(DENTRO) => pagina(ASIGNATURAS),
        _ => ir_a("/user.php?action=login", None),
    }
}
