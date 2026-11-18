//! Los ajustes de la app (H1): por ahora, solo dónde están las plantillas. Y
//! dónde van las notas exportadas (H2), que no se elige.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_y_vuelta_y_sin_fichero() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("app").join("ajustes.json");
        assert_eq!(leer(&ruta).unwrap(), Ajustes::default());

        let a = Ajustes { carpeta_plantillas: Some(PathBuf::from(r"D:\Clase\plantillas")) };
        guardar(&ruta, &a).unwrap();
        assert_eq!(leer(&ruta).unwrap(), a);
        assert!(!ruta.with_extension("json.tmp").exists());
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
        let c = carpeta_plantillas(&Ajustes { carpeta_plantillas: Some(elegida.clone()) }, docs.path());
        assert!(!c.por_defecto && !c.existe);
        assert!(!elegida.exists());
    }
}
