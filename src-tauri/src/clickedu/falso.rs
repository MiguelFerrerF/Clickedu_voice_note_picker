//! Un Clickedu falso para los tests (H4): un servidor HTTP en un hilo, en un
//! puerto libre de `127.0.0.1`.
//!
//! Las páginas son **inventadas**, de la forma que esperaba la v1 y que
//! confirmó Clickedu de verdad: el formulario del archivo con `id_usuari`,
//! `usrKO` en la dirección si el usuario está mal, «Archivo de seguridad
//! INCORRECTO», sin sesión un 200 con un aviso en JS… Sirve para entrar,
//! caducar y volver a entrar; lo que se lee de dentro se prueba en
//! [`super::fichas`] con páginas de verdad anonimizadas.

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
    /// Como `Normal`, pero la sesión caduca tras dos peticiones de dentro
    /// (la de comprobar y otra): la tercera contesta como sin sesión, hasta
    /// volver a entrar.
    Caduca,
    /// Como `Normal`, pero importar contesta sin «Importación correcta» y
    /// no importa nada.
    NoImporta,
    /// Como `Normal`, pero lo subido no sale en la tabla.
    NoGuardaSubido,
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
/// Tres materias: la 101 se trae bien; la 102 no tiene evaluaciones; la
/// 103, al bajarla, no da un Excel.
const ASIGNATURAS: &str = r#"<!DOCTYPE html><html><body>
<div class="panel panel-separador"><div class="title">Tercero de ESO</div></div>
<div class="bloc-assignatura"><a href="/go/Index.php?ctr=Subject/Subject&id=101">
  <div class="title">Matemáticas</div><div class="sub-title">3º E.S.O.-A</div></a></div>
<div class="bloc-assignatura"><a href="/go/Index.php?ctr=Subject/Subject&id=102">
  <div class="title">Música</div><div class="sub-title">3º E.S.O.-A</div></a></div>
<div class="bloc-assignatura"><a href="/go/Index.php?ctr=Subject/Subject&id=103">
  <div class="title">Tecnología</div><div class="sub-title">3º E.S.O.-B</div></a></div>
</body></html>"#;
const ITEMS_SIN_EVALUACIONES: &str = "<html><body><p>Sin evaluaciones</p></body></html>";
const ITEMS_103: &str =
    r#"<html><body><select name="trm"><option value="8">1ª evaluación</option></select></body></html>"#;
const NO_ES_EXCEL: &str = "<html><body><p>Error al generar la plantilla</p></body></html>";
/// La página de importar de la 101, en su evaluación: dos ítems, el
/// formulario de subir (con la acción relativa, como la de verdad) y la
/// tabla de lo subido (`{filas}`).
const IMPORTAR_101: &str = r#"<html><body>
<form action="importar_notes_items.php?accio=carregar_plantilla&amp;assig=101&amp;id_aval=7" enctype="multipart/form-data" method="POST">
<select name="id_criteri_avaluacio"><option value="">---</option>
<option value="9001">Examen 1</option><option value="9002">Trabajo</option></select>
<input type="file" name="v_arxiu_importar"><input type="submit" name="BotoGenerar" value="Cargar">
</form><table>{filas}</table></body></html>"#;
/// Lo puesto en la 101: un alumno en el Examen 1, otro en blanco, y la nota
/// final. Lo importado se pone encima ([`Estado`]).
type Fila = (&'static str, &'static str, Option<&'static str>, Option<&'static str>, i64);
const NOTAS_101: [Fila; 3] = [
    ("5001", "9001", None, Some("7.5"), 0),
    ("5002", "9001", Some(""), None, 0),
    ("5001", "101", Some("7"), None, 114),
];
const SUBIR_101: &str = "/assignatures/importar_notes_items.php?accio=carregar_plantilla&assig=101&id_aval=7";
const IMPORTAR_NOTAS_101: &str = "/assignatures/importar_notes_items.php?accio=importar_notes&assig=101&id_aval=7&id=";
pub const BAJAR_101: &str = "/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig=101&id_aval=7";
const BAJAR_103: &str = "/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig=103&id_aval=8";

/// La plantilla que da Clickedu, con dos alumnos inventados: la hoja
/// `Worksheet` y `Id | Alumnos | Nota`, como las de verdad. Con la fecha de
/// creación fija: si no, dos llamadas pueden dar bytes distintos.
pub fn plantilla_101() -> Vec<u8> {
    use rust_xlsxwriter::{DocProperties, ExcelDateTime};
    let mut libro = rust_xlsxwriter::Workbook::new();
    let creada = ExcelDateTime::from_ymd(2026, 11, 22).unwrap();
    libro.set_properties(&DocProperties::new().set_creation_datetime(&creada));
    let hoja = libro.add_worksheet().set_name("Worksheet").unwrap();
    for (c, t) in ["Id", "Alumnos", "Nota"].iter().enumerate() {
        hoja.write_string(0, c as u16, *t).unwrap();
    }
    for (f, (id, nombre)) in [(5001, "Castro Ruiz, Lucía"), (5002, "Ortega Gil, Mateo")].iter().enumerate() {
        hoja.write_number(f as u32 + 1, 0, *id as f64).unwrap();
        hoja.write_string(f as u32 + 1, 1, *nombre).unwrap();
    }
    libro.save_to_buffer().unwrap()
}

fn excel(bytes: Vec<u8>) -> Respuesta {
    Response::from_data(bytes)
        .with_header(
            Header::from_bytes("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                .unwrap(),
        )
        .with_header(
            Header::from_bytes("Content-Disposition", "attachment; filename=\"evaluate_20261122_101500.xlsx\"")
                .unwrap(),
        )
}

const SIN_SESION: &str = "<html><body class='home'><script>alert('No ha iniciado sesión');
window.location.href='/user.php?action=login';</script></body></html>";

/// Arranca el Clickedu falso. Se para al soltarlo.
pub fn arrancar(modo: Modo) -> Falso {
    let servidor = Arc::new(Server::http("127.0.0.1:0").expect("servidor falso"));
    let puerto = servidor.server_addr().to_ip().expect("puerto").port();
    let pedidas = Arc::new(Mutex::new(Vec::new()));
    let (s, p) = (servidor.clone(), pedidas.clone());
    // Las páginas de dentro pedidas desde la última vez que se entró.
    let mut de_dentro = 0;
    let mut estado = Estado::default();
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
            let respuesta = if (metodo.as_str(), ruta.as_str()) == ("POST", SUBIR_101) && cookies.contains(DENTRO) {
                pagina(&estado.subir(modo, &cuerpo))
            } else {
                let texto = String::from_utf8_lossy(&cuerpo).into_owned();
                responder(modo, &metodo, &ruta, &cookies, &texto, &mut de_dentro, &mut estado)
            };
            let _ = peticion.respond(respuesta);
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

/// Los ítems, en Latin-1 y sin decirlo, como casi todo Clickedu: «1ª
/// evaluación».
const ITEMS_LATIN1: &[u8] = b"<html><body><select name=\"trm\"><option value=\"7\">1\xaa evaluaci\xf3n</option>\
</select></body></html>";

/// Lo subido a la 101 y lo importado.
#[derive(Default)]
struct Estado {
    /// Lo subido: su número, su ítem, sus notas (por `Id`) y si se ha
    /// importado.
    subidos: Vec<(u32, String, Vec<(String, String)>, bool)>,
    /// Lo importado, por alumno e ítem: va en `nota_mod`.
    importadas: std::collections::BTreeMap<(String, String), String>,
}

impl Estado {
    /// El `POST` de subir: el ítem y el Excel del *multipart*.
    fn subir(&mut self, modo: Modo, cuerpo: &[u8]) -> String {
        use calamine::Reader;
        let item = campo(cuerpo, "id_criteri_avaluacio").map(|v| String::from_utf8_lossy(v).trim().to_string());
        let (Some(item), Some(fichero)) = (item, campo(cuerpo, "v_arxiu_importar")) else {
            return r#"<html><script>alert("Faltan datos")</script></html>"#.into();
        };
        let Ok(mut libro) = calamine::Xlsx::new(std::io::Cursor::new(fichero.to_vec())) else {
            return r#"<html><script>alert("El fichero no es correcto")</script></html>"#.into();
        };
        let (hoja, rango) = libro.worksheets().remove(0);
        let plantilla = crate::plantillas::leer_rango(hoja, &rango).expect("el Excel subido se lee");
        let notas = plantilla
            .alumnos
            .iter()
            .filter_map(|a| match a.nota {
                crate::plantillas::Nota::Valor(v) => Some((a.id.clone(), format!("{v}"))),
                _ => None,
            })
            .collect();
        if modo != Modo::NoGuardaSubido {
            self.subidos.push((70 + self.subidos.len() as u32, item, notas, false));
        }
        r#"<html><script>alert("Archivo subido correctamente.");location.href='importar_notes_items.php';</script></html>"#
            .into()
    }

    /// La tabla de lo subido: el enlace de importar, hasta importarlo.
    fn filas(&self) -> String {
        self.subidos
            .iter()
            .map(|(id, _, _, importado)| {
                let importar = if *importado {
                    String::new()
                } else {
                    format!(
                        r#"<a href="importar_notes_items.php?accio=importar_notes&amp;assig=101&amp;id_aval=7&amp;id={id}">i</a>"#
                    )
                };
                format!(
                    r##"<tr><td>{importar}<a href="#a" class="esborrar BotoEsborrarImportacio" cgap="101" ind="{id}" aval="7">e</a></td></tr>"##
                )
            })
            .collect()
    }

    fn importar(&mut self, modo: Modo, id: &str) -> String {
        let Some(s) = self.subidos.iter_mut().find(|s| s.0.to_string() == id && !s.3) else {
            return r#"<html><script>alert("No existe")</script></html>"#.into();
        };
        if modo == Modo::NoImporta {
            return "<html><body></body></html>".into();
        }
        s.3 = true;
        for (alumno, nota) in &s.2 {
            self.importadas.insert((alumno.clone(), s.1.clone()), nota.clone());
        }
        r#"<html><script>alert("Importación correcta")</script></html>"#.into()
    }

    /// `GetGrades` de la 101: lo puesto y, encima, lo importado.
    fn notas(&self) -> String {
        use serde_json::json;
        let mut importadas = self.importadas.clone();
        let mut filas = Vec::new();
        for (alumno, criterio, nota, nota_mod, tipo) in NOTAS_101 {
            let puesta = importadas.remove(&(alumno.to_string(), criterio.to_string()));
            let nota_mod = puesta.or(nota_mod.map(String::from));
            filas.push(
                json!({"id_alumne": alumno, "id_criteri": criterio, "nota": nota, "nota_mod": nota_mod, "type": tipo}),
            );
        }
        for ((alumno, criterio), nota) in importadas {
            filas.push(json!({"id_alumne": alumno, "id_criteri": criterio, "nota": null, "nota_mod": nota, "type": 0}));
        }
        serde_json::Value::Array(filas).to_string()
    }
}

/// El valor del campo `nombre` de un *multipart*, tal cual.
fn campo<'a>(cuerpo: &'a [u8], nombre: &str) -> Option<&'a [u8]> {
    let buscar =
        |aguja: &[u8], desde: usize| cuerpo[desde..].windows(aguja.len()).position(|w| w == aguja).map(|i| i + desde);
    let i = buscar(format!("name=\"{nombre}\"").as_bytes(), 0)?;
    let inicio = buscar(b"\r\n\r\n", i)? + 4;
    let fin = buscar(b"\r\n--", inicio)?;
    Some(&cuerpo[inicio..fin])
}

fn responder(
    modo: Modo,
    metodo: &str,
    ruta: &str,
    cookies: &str,
    cuerpo: &str,
    de_dentro: &mut u32,
    estado: &mut Estado,
) -> Respuesta {
    if modo == Modo::Caido {
        return pagina("Service Unavailable").with_status_code(503);
    }
    let de_dentro_es = ruta.starts_with("/go/") || ruta.starts_with("/assignatures/");
    if de_dentro_es && cookies.contains(DENTRO) {
        *de_dentro += 1;
        if modo == Modo::Caduca && *de_dentro > 2 {
            return pagina(SIN_SESION);
        }
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
                (true, _) => {
                    *de_dentro = 0;
                    ir_a("/index.php", Some(DENTRO))
                }
            }
        }
        ("GET", "/index.php") => pagina(INICIO),
        ("GET", "/go/Index.php?ctr=Subjects/Subjects") if cookies.contains(DENTRO) => pagina(ASIGNATURAS),
        ("GET", "/go/Index.php?ctr=Subject/Items&do=Index&id=101") if cookies.contains(DENTRO) => {
            Response::from_data(ITEMS_LATIN1).with_header(Header::from_bytes("Content-Type", "text/html").unwrap())
        }
        ("GET", "/go/Index.php?ctr=Subject/Items&do=Index&id=102") if cookies.contains(DENTRO) => {
            pagina(ITEMS_SIN_EVALUACIONES)
        }
        ("GET", "/go/Index.php?ctr=Subject/Items&do=Index&id=103") if cookies.contains(DENTRO) => pagina(ITEMS_103),
        ("GET", BAJAR_101) if cookies.contains(DENTRO) => excel(plantilla_101()),
        ("GET", BAJAR_103) if cookies.contains(DENTRO) => pagina(NO_ES_EXCEL),
        ("GET", "/assignatures/importar_notes_items.php?assig=101&id_aval=7") if cookies.contains(DENTRO) => {
            pagina(&IMPORTAR_101.replace("{filas}", &estado.filas()))
        }
        ("GET", r) if r.starts_with(IMPORTAR_NOTAS_101) && cookies.contains(DENTRO) => {
            pagina(&estado.importar(modo, &r[IMPORTAR_NOTAS_101.len()..]))
        }
        ("GET", "/go/Index.php?ctr=Evaluate/EvaluateTermly&do=GetGrades&id=101&trm=7") if cookies.contains(DENTRO) => {
            pagina(&estado.notas())
        }
        // Sin sesión, el de verdad no redirige: contesta 200 con un aviso y
        // manda a entrar con JS (visto el 2026-11-20).
        ("GET", r) if r.starts_with("/go/") || r.starts_with("/assignatures/") => pagina(SIN_SESION),
        _ => ir_a("/user.php?action=login", None),
    }
}
