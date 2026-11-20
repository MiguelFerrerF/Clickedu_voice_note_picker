//! La cuenta de Clickedu guardada (H4, D3).
//!
//! - **Usuario y contraseña, en el Administrador de credenciales de
//!   Windows**, juntos en una entrada ([`LlaveroWindows`]). Nunca en un
//!   fichero: ya se filtraron una vez desde el `config.json` de la v1.
//! - **El archivo de seguridad**, copiado a la carpeta de la app
//!   (`%APPDATA%\es.gestornotas.app\clickedu\`) con su nombre: la v1 lo
//!   dejaba en una carpeta temporal y se perdía.
//! - `cuenta.json`, al lado, solo con el nombre del archivo y cuándo se
//!   eligió: nada que sirva para entrar.
//!
//! Se guarda **solo después de haber entrado** con ella (lo decide quien
//! llama, `commands::clickedu`): así nunca queda guardada una cuenta que no
//! funciona.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Dónde guarda Windows la entrada: «es.gestornotas.app/clickedu», como
/// la ve el Administrador de credenciales.
pub const SERVICIO: &str = "es.gestornotas.app/clickedu";
const ENTRADA: &str = "cuenta";
const FICHERO: &str = "cuenta.json";

/// Un archivo de seguridad es un texto corto. Uno de más de esto es otro
/// fichero elegido sin querer, y no se manda a Clickedu.
pub const MAXIMO_ARCHIVO: u64 = 64 * 1024;

/// Dónde se guarda el secreto. En la app, el Administrador de credenciales
/// ([`LlaveroWindows`]); en los tests, uno en memoria.
pub trait Llavero: Send + Sync {
    fn leer(&self) -> Result<Option<String>, String>;
    fn guardar(&self, secreto: &str) -> Result<(), String>;
    fn borrar(&self) -> Result<(), String>;
}

/// El Administrador de credenciales de Windows.
pub struct LlaveroWindows;

impl LlaveroWindows {
    fn entrada() -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICIO, ENTRADA)
            .map_err(|e| format!("no se puede abrir el Administrador de credenciales: {e}"))
    }
}

impl Llavero for LlaveroWindows {
    fn leer(&self) -> Result<Option<String>, String> {
        match Self::entrada()?.get_password() {
            Ok(s) => Ok(Some(s)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("no se puede leer la cuenta del Administrador de credenciales: {e}")),
        }
    }

    fn guardar(&self, secreto: &str) -> Result<(), String> {
        Self::entrada()?
            .set_password(secreto)
            .map_err(|e| format!("no se puede guardar la cuenta en el Administrador de credenciales: {e}"))
    }

    fn borrar(&self) -> Result<(), String> {
        match Self::entrada()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("no se puede borrar la cuenta del Administrador de credenciales: {e}")),
        }
    }
}

/// Lo que va en el Administrador de credenciales.
#[derive(Serialize, Deserialize)]
struct Secreto {
    usuario: String,
    contrasena: String,
}

/// `cuenta.json`: nada que sirva para entrar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Fichero {
    /// El nombre del archivo de seguridad, dentro de la carpeta.
    archivo: String,
    /// Cuándo se eligió, en segundos desde 1970.
    elegido: u64,
}

/// La cuenta guardada, con la contraseña: solo para entrar, nunca para la
/// interfaz (para eso, [`Guardada::vista`]).
#[derive(Clone)]
pub struct Guardada {
    pub usuario: String,
    pub contrasena: String,
    /// La copia del archivo de seguridad. `None` si falta: borrado a mano, o
    /// la cuenta se guardó y el archivo no llegó a copiarse.
    pub archivo: Option<PathBuf>,
    pub elegido: Option<u64>,
}

/// Lo que ve la interfaz: **sin la contraseña**.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vista {
    pub usuario: String,
    /// El nombre del archivo de seguridad; `None` si falta.
    pub archivo: Option<String>,
    pub elegido: Option<u64>,
}

impl Guardada {
    pub fn vista(&self) -> Vista {
        Vista {
            usuario: self.usuario.clone(),
            archivo: self.archivo.as_ref().and_then(|a| a.file_name()).map(|n| n.to_string_lossy().into_owned()),
            elegido: self.elegido,
        }
    }
}

/// La cuenta guardada, o `None` si no hay.
pub fn leer(carpeta: &Path, llavero: &dyn Llavero) -> Result<Option<Guardada>, String> {
    let Some(texto) = llavero.leer()? else {
        return Ok(None);
    };
    let s: Secreto = serde_json::from_str(&texto).map_err(|_| {
        "la cuenta guardada en el Administrador de credenciales está dañada: olvídala y vuelve a entrar".to_string()
    })?;
    let fichero = leer_fichero(carpeta);
    let archivo = fichero.as_ref().map(|f| carpeta.join(&f.archivo)).filter(|a| a.is_file());
    Ok(Some(Guardada {
        usuario: s.usuario,
        contrasena: s.contrasena,
        elegido: fichero.filter(|_| archivo.is_some()).map(|f| f.elegido),
        archivo,
    }))
}

fn leer_fichero(carpeta: &Path) -> Option<Fichero> {
    serde_json::from_str(&std::fs::read_to_string(carpeta.join(FICHERO)).ok()?).ok()
}

/// Lee el archivo de seguridad que se va a mandar, con su nombre.
pub fn leer_archivo(ruta: &Path) -> Result<(String, Vec<u8>), String> {
    let nombre = ruta
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| format!("{} no es un archivo", ruta.display()))?;
    let datos =
        std::fs::metadata(ruta).map_err(|e| format!("no se puede leer el archivo de seguridad {nombre}: {e}"))?;
    if !datos.is_file() {
        return Err(format!("{nombre} no es un archivo"));
    }
    if datos.len() > MAXIMO_ARCHIVO {
        return Err(format!("{nombre} no parece un archivo de seguridad de Clickedu: es demasiado grande"));
    }
    let bytes = std::fs::read(ruta).map_err(|e| format!("no se puede leer el archivo de seguridad {nombre}: {e}"))?;
    Ok((nombre, bytes))
}

/// Guarda la cuenta con la que **ya se ha entrado**. Con `archivo`, el de
/// seguridad nuevo (nombre y contenido), que sustituye al de antes; sin él,
/// se queda el que había.
pub fn guardar(
    carpeta: &Path,
    llavero: &dyn Llavero,
    usuario: &str,
    contrasena: &str,
    archivo: Option<(&str, &[u8])>,
    ahora: u64,
) -> Result<Guardada, String> {
    std::fs::create_dir_all(carpeta).map_err(|e| format!("no se puede crear {}: {e}", carpeta.display()))?;
    if let Some((nombre, bytes)) = archivo {
        let nombre = nombre_seguro(nombre);
        let anterior = leer_fichero(carpeta);
        let destino = carpeta.join(&nombre);
        let temporal = carpeta.join(format!("{nombre}.parcial"));
        let escribir =
            |e: std::io::Error| format!("no se puede guardar el archivo de seguridad en {}: {e}", carpeta.display());
        std::fs::write(&temporal, bytes).map_err(escribir)?;
        std::fs::rename(&temporal, &destino).map_err(escribir)?;
        crate::ajustes::guardar_json(&carpeta.join(FICHERO), &Fichero { archivo: nombre.clone(), elegido: ahora })?;
        if let Some(a) = anterior.filter(|a| a.archivo != nombre) {
            let _ = std::fs::remove_file(carpeta.join(a.archivo));
        }
    }
    let secreto = serde_json::to_string(&Secreto { usuario: usuario.into(), contrasena: contrasena.into() })
        .map_err(|e| e.to_string())?;
    llavero.guardar(&secreto)?;
    leer(carpeta, llavero)?.ok_or_else(|| "la cuenta no ha quedado guardada".to_string())
}

/// Olvida la cuenta: la entrada del Administrador de credenciales y la
/// carpeta entera (el archivo de seguridad y las páginas guardadas).
pub fn olvidar(carpeta: &Path, llavero: &dyn Llavero) -> Result<(), String> {
    llavero.borrar()?;
    match std::fs::remove_dir_all(carpeta) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("no se puede borrar {}: {e}", carpeta.display())),
    }
}

/// El nombre del archivo, sin nada que lo saque de la carpeta ni lo
/// confunda con `cuenta.json`.
fn nombre_seguro(nombre: &str) -> String {
    let limpio: String = Path::new(nombre).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if limpio.is_empty() || limpio.eq_ignore_ascii_case(FICHERO) || limpio.ends_with(".parcial") {
        "archivo_seguridad.txt".into()
    } else {
        limpio
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Mutex;

    use super::*;

    /// El llavero de los tests: en memoria.
    #[derive(Default)]
    pub(crate) struct LlaveroMemoria(pub Mutex<Option<String>>);

    impl Llavero for LlaveroMemoria {
        fn leer(&self) -> Result<Option<String>, String> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn guardar(&self, secreto: &str) -> Result<(), String> {
            *self.0.lock().unwrap() = Some(secreto.into());
            Ok(())
        }
        fn borrar(&self) -> Result<(), String> {
            *self.0.lock().unwrap() = None;
            Ok(())
        }
    }

    #[test]
    fn sin_cuenta() {
        let dir = tempfile::tempdir().unwrap();
        assert!(leer(dir.path(), &LlaveroMemoria::default()).unwrap().is_none());
    }

    #[test]
    fn guardada_y_leida_sin_contrasena_en_ficheros() {
        let dir = tempfile::tempdir().unwrap();
        let carpeta = dir.path().join("clickedu");
        let llavero = LlaveroMemoria::default();
        let g =
            guardar(&carpeta, &llavero, "profe", "secreta", Some(("clave_3062.txt", b"CLAVE")), 1_800_000_000).unwrap();
        assert_eq!(g.contrasena, "secreta");
        assert_eq!(
            g.vista(),
            Vista { usuario: "profe".into(), archivo: Some("clave_3062.txt".into()), elegido: Some(1_800_000_000) }
        );
        assert_eq!(std::fs::read(carpeta.join("clave_3062.txt")).unwrap(), b"CLAVE");
        // Ni el usuario ni la contraseña en ningún fichero.
        for f in std::fs::read_dir(&carpeta).unwrap() {
            let texto = String::from_utf8_lossy(&std::fs::read(f.unwrap().path()).unwrap()).into_owned();
            assert!(!texto.contains("secreta") && !texto.contains("profe"), "{texto}");
        }
    }

    #[test]
    fn otra_contrasena_conserva_el_archivo() {
        let dir = tempfile::tempdir().unwrap();
        let llavero = LlaveroMemoria::default();
        guardar(dir.path(), &llavero, "profe", "vieja", Some(("clave.txt", b"CLAVE")), 1).unwrap();
        let g = guardar(dir.path(), &llavero, "profe", "nueva", None, 2).unwrap();
        assert_eq!(g.contrasena, "nueva");
        assert_eq!(g.vista().archivo.as_deref(), Some("clave.txt"));
        assert_eq!(g.elegido, Some(1));
    }

    #[test]
    fn otro_archivo_sustituye_al_de_antes() {
        let dir = tempfile::tempdir().unwrap();
        let llavero = LlaveroMemoria::default();
        guardar(dir.path(), &llavero, "profe", "s", Some(("vieja.txt", b"A")), 1).unwrap();
        let g = guardar(dir.path(), &llavero, "profe", "s", Some(("nueva.txt", b"B")), 2).unwrap();
        assert_eq!(g.vista().archivo.as_deref(), Some("nueva.txt"));
        assert!(!dir.path().join("vieja.txt").exists());
    }

    #[test]
    fn si_falta_el_archivo_se_dice() {
        let dir = tempfile::tempdir().unwrap();
        let llavero = LlaveroMemoria::default();
        guardar(dir.path(), &llavero, "profe", "s", Some(("clave.txt", b"A")), 1).unwrap();
        std::fs::remove_file(dir.path().join("clave.txt")).unwrap();
        let g = leer(dir.path(), &llavero).unwrap().unwrap();
        assert_eq!(g.vista(), Vista { usuario: "profe".into(), archivo: None, elegido: None });
    }

    #[test]
    fn un_nombre_no_sale_de_la_carpeta() {
        assert_eq!(nombre_seguro(r"..\..\fuera.txt"), "fuera.txt");
        assert_eq!(nombre_seguro("cuenta.json"), "archivo_seguridad.txt");
        assert_eq!(nombre_seguro(""), "archivo_seguridad.txt");
    }

    #[test]
    fn olvidar_lo_borra_todo() {
        let dir = tempfile::tempdir().unwrap();
        let carpeta = dir.path().join("clickedu");
        let llavero = LlaveroMemoria::default();
        guardar(&carpeta, &llavero, "profe", "s", Some(("clave.txt", b"A")), 1).unwrap();
        olvidar(&carpeta, &llavero).unwrap();
        assert!(!carpeta.exists());
        assert!(leer(&carpeta, &llavero).unwrap().is_none());
        // Dos veces no es un error.
        olvidar(&carpeta, &llavero).unwrap();
    }

    #[test]
    fn un_archivo_grande_no_se_manda() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("foto.jpg");
        std::fs::write(&ruta, vec![0u8; MAXIMO_ARCHIVO as usize + 1]).unwrap();
        assert!(leer_archivo(&ruta).unwrap_err().contains("demasiado grande"));
    }
}
