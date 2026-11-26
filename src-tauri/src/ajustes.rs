//! Los ajustes de la app (H1): dónde están las plantillas y, desde HUI, por
//! qué micrófono se dicta. Y dónde van las notas exportadas (H2), que no se
//! elige.
//!
//! Y, desde H9, lo que ya se ha visto de la ayuda: la bienvenida y el aviso
//! de cada pantalla. Desde el inicio (HUI, 2026-11-25), la última clase
//! abierta en Calificar, para seguir donde se dejó. Desde H11, si se
//! refresca solo lo de Clickedu y cada cuánto.
//!
//! Viven en `ajustes.json`, en la carpeta de la app (D4:
//! `%APPDATA%\es.gestornotas.app\`). **Nunca credenciales** aquí (D3): las de
//! Clickedu van al Administrador de credenciales de Windows.
//!
//! La carpeta de plantillas, si no se ha elegido otra, es
//! `Documentos\Gestor de Notas\plantillas`, y se crea sola. Una elegida a mano
//! **no se crea**: si ha desaparecido se dice, porque crearla vacía escondería
//! que las plantillas ya no están donde el profesor cree.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ajustes {
    /// `None`: la de por defecto.
    #[serde(default)]
    pub carpeta_plantillas: Option<PathBuf>,
    /// `None`: el micrófono de Windows.
    #[serde(default)]
    pub microfono: Option<MicrofonoElegido>,
    /// Lo que ya se ha visto de la ayuda (H9): `bienvenida` y una clave por
    /// pantalla. Sin el campo (unos ajustes de antes), nada visto.
    #[serde(default)]
    pub ayuda_vista: Vec<String>,
    /// La última clase abierta en Calificar, y su ítem (el inicio). Sin el
    /// campo, ninguna.
    #[serde(default)]
    pub ultima: Option<Ultima>,
    /// Refrescar lo de Clickedu solo (H11). Sin el campo, encendido y cada
    /// 30 minutos ([`Refresco::default`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresco: Option<Refresco>,
}

/// Cada cuánto se refresca lo de Clickedu, si se refresca (H11).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Refresco {
    pub encendido: bool,
    pub minutos: u32,
}

impl Refresco {
    /// Los minutos que se pueden elegir en Ajustes.
    pub const MINUTOS: [u32; 3] = [15, 30, 60];
}

impl Default for Refresco {
    fn default() -> Self {
        Refresco { encendido: true, minutos: 30 }
    }
}

/// La última clase abierta en Calificar: su fichero y, si se calificaba en
/// un ítem de Clickedu, cuál (es `ItemElegido` del lado de la ventana).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ultima {
    pub fichero: String,
    #[serde(default)]
    pub item: Option<ItemUltimo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemUltimo {
    pub evaluacion: String,
    pub item: String,
}

/// El micrófono elegido en Ajustes: su id de `cpal`, que es con lo que se
/// abre (dos pueden llamarse igual), y su nombre, para decir cuál falta si
/// no está conectado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicrofonoElegido {
    pub id: String,
    pub nombre: String,
}

/// Lee los ajustes. Si el fichero no existe, los de por defecto; si existe y
/// no se entiende, error: no se pisa en silencio lo que había.
pub fn leer(ruta: &Path) -> Result<Ajustes, String> {
    match std::fs::read_to_string(ruta) {
        Ok(texto) => serde_json::from_str(&texto)
            .map_err(|e| format!("los ajustes de {} están dañados: {e}", ruta.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Ajustes::default()),
        Err(e) => Err(format!("no se pueden leer los ajustes de {}: {e}", ruta.display())),
    }
}

/// Guarda los ajustes de una vez (ver [`guardar_json`]).
pub fn guardar(ruta: &Path, ajustes: &Ajustes) -> Result<(), String> {
    guardar_json(ruta, ajustes).map_err(|e| format!("no se pueden guardar los ajustes en {}: {e}", ruta.display()))
}

/// Escribe `valor` como JSON de una vez: a un temporal al lado y se renombra,
/// para que un corte a mitad no deje medio fichero. Lo usan los ajustes y la
/// sesión.
pub(crate) fn guardar_json<T: Serialize>(ruta: &Path, valor: &T) -> Result<(), String> {
    let carpeta = ruta.parent().ok_or("ruta sin carpeta")?;
    std::fs::create_dir_all(carpeta).map_err(|e| e.to_string())?;
    let texto = serde_json::to_string_pretty(valor).map_err(|e| e.to_string())?;
    let temporal = ruta.with_extension("json.tmp");
    let mut f = std::fs::File::create(&temporal).map_err(|e| e.to_string())?;
    f.write_all(texto.as_bytes()).and_then(|_| f.sync_all()).map_err(|e| e.to_string())?;
    drop(f);
    std::fs::rename(&temporal, ruta).map_err(|e| e.to_string())
}

/// La carpeta de plantillas que toca, y si es la de por defecto.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CarpetaPlantillas {
    pub ruta: String,
    pub por_defecto: bool,
    pub existe: bool,
}

/// Resuelve la carpeta de plantillas. La de por defecto se crea si falta.
pub fn carpeta_plantillas(ajustes: &Ajustes, documentos: &Path) -> CarpetaPlantillas {
    let (ruta, por_defecto) = match &ajustes.carpeta_plantillas {
        Some(r) => (r.clone(), false),
        None => (por_defecto(documentos), true),
    };
    if por_defecto {
        // Si no se puede crear, `existe` lo dirá.
        let _ = std::fs::create_dir_all(&ruta);
    }
    CarpetaPlantillas { existe: ruta.is_dir(), ruta: ruta.display().to_string(), por_defecto }
}

/// `Documentos\Gestor de Notas\plantillas` (D4).
pub fn por_defecto(documentos: &Path) -> PathBuf {
    documentos.join("Gestor de Notas").join("plantillas")
}

/// `Documentos\Gestor de Notas\notas` (D4): donde se exporta. Siempre esta,
/// aunque las plantillas estén en otra carpeta: lo exportado es para subirlo
/// a Clickedu, y ha de estar a la vista.
pub fn carpeta_notas(documentos: &Path) -> PathBuf {
    documentos.join("Gestor de Notas").join("notas")
}

/// `Documentos\Gestor de Notas\informes` (D8): los informes del registro,
/// a la vista para mandarlos.
pub fn carpeta_informes(documentos: &Path) -> PathBuf {
    documentos.join("Gestor de Notas").join("informes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_y_vuelta_y_sin_fichero() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("app").join("ajustes.json");
        assert_eq!(leer(&ruta).unwrap(), Ajustes::default());

        let a = Ajustes {
            carpeta_plantillas: Some(PathBuf::from(r"D:\Clase\plantillas")),
            microfono: Some(MicrofonoElegido { id: "wasapi:{0.0.1}".into(), nombre: "Auriculares".into() }),
            ayuda_vista: vec!["bienvenida".into(), "calificar".into()],
            ultima: Some(Ultima {
                fichero: "biologia.xlsx".into(),
                item: Some(ItemUltimo { evaluacion: "2".into(), item: "42700".into() }),
            }),
            refresco: Some(Refresco { encendido: false, minutos: 60 }),
        };
        guardar(&ruta, &a).unwrap();
        assert_eq!(leer(&ruta).unwrap(), a);
        assert!(!ruta.with_extension("json.tmp").exists());
    }

    #[test]
    fn unos_ajustes_de_antes_de_la_ayuda_no_la_han_visto() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("ajustes.json");
        std::fs::write(&ruta, r#"{"carpetaPlantillas": null, "microfono": null}"#).unwrap();
        let a = leer(&ruta).unwrap();
        assert!(a.ayuda_vista.is_empty());
        assert_eq!(a.ultima, None);
        // Ni el refresco (H11): encendido y cada 30 minutos.
        assert_eq!(a.refresco.unwrap_or_default(), Refresco { encendido: true, minutos: 30 });
    }

    #[test]
    fn unos_ajustes_de_antes_del_microfono_se_leen() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("ajustes.json");
        std::fs::write(&ruta, r#"{"carpetaPlantillas": null}"#).unwrap();
        assert_eq!(leer(&ruta).unwrap(), Ajustes::default());
    }

    #[test]
    fn danados_es_error_y_no_se_pisan() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("ajustes.json");
        std::fs::write(&ruta, "{ esto no es json").unwrap();
        assert!(leer(&ruta).is_err());
        assert_eq!(std::fs::read_to_string(&ruta).unwrap(), "{ esto no es json");
    }

    #[test]
    fn la_de_por_defecto_se_crea_y_la_elegida_no() {
        let docs = tempfile::tempdir().unwrap();
        let c = carpeta_plantillas(&Ajustes::default(), docs.path());
        assert!(c.por_defecto && c.existe);
        assert!(docs.path().join("Gestor de Notas").join("plantillas").is_dir());

        let elegida = docs.path().join("no-existe");
        let c = carpeta_plantillas(
            &Ajustes { carpeta_plantillas: Some(elegida.clone()), ..Default::default() },
            docs.path(),
        );
        assert!(!c.por_defecto && !c.existe);
        assert!(!elegida.exists());
    }
}
