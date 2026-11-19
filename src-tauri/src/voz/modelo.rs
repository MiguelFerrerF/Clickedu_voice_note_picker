//! El modelo de Whisper (D2): `small` `q5_1`, 181 MB.
//!
//! No va en el instalador, que así sigue pequeño: **se descarga la primera
//! vez que se usa la voz** (el usuario, 2026-11-19), a
//! `%APPDATA%\es.gestornotas.app\modelos`, la misma carpeta que usa la
//! prueba de H3a (`examples/probar_voz`).
//!
//! Se baja de Hugging Face, de los modelos de whisper.cpp, **fijado a un
//! commit**: si allí cambian el fichero, se sigue bajando este. Y antes de
//! dejarlo en su sitio se comprueba que ha llegado entero y es ese
//! ([`Comprobar`]): un modelo a medias no se carga nunca.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// El nombre del fichero, como lo llama whisper.cpp.
pub const FICHERO: &str = "ggml-small-q5_1.bin";
/// El commit `5359861` de `ggerganov/whisper.cpp` en Hugging Face.
pub const URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/\
                       5359861c739e955e79d9a303bcbc70fb988958b1/ggml-small-q5_1.bin";
pub const BYTES: u64 = 190_085_487;
/// El que da Hugging Face, y el del modelo con el que se hizo la prueba de
/// H3a (comprobado el 2026-11-19).
pub const SHA256: &str = "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb";

/// Dónde está, o estará, el modelo.
pub fn ruta(carpeta: &Path) -> PathBuf {
    carpeta.join(FICHERO)
}

/// Si el modelo ya está. Basta con el tamaño: solo se deja en su sitio
/// después de comprobarlo entero.
pub fn esta(carpeta: &Path) -> bool {
    std::fs::metadata(ruta(carpeta)).is_ok_and(|m| m.len() == BYTES)
}

/// Comprueba, mientras llega, que lo descargado es el modelo: tamaño y
/// SHA-256.
pub struct Comprobar {
    hash: Sha256,
    bytes: u64,
    esperados: u64,
    sha256: &'static str,
}

impl Comprobar {
    /// Para el modelo de la app.
    pub fn modelo() -> Self {
        Self::new(BYTES, SHA256)
    }

    pub fn new(esperados: u64, sha256: &'static str) -> Self {
        Self { hash: Sha256::new(), bytes: 0, esperados, sha256 }
    }

    pub fn anadir(&mut self, trozo: &[u8]) {
        self.hash.update(trozo);
        self.bytes += trozo.len() as u64;
    }

    pub fn terminar(self) -> Result<(), String> {
        if self.bytes != self.esperados {
            return Err(format!("ha llegado incompleto: {} de {} bytes", self.bytes, self.esperados));
        }
        let hecho: String = self.hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if hecho != self.sha256 {
            return Err("ha llegado, pero no es el que tenía que ser (no coincide su huella SHA-256)".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SHA-256 de «abc» (FIPS 180-2).
    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn entero_y_el_que_es() {
        let mut c = Comprobar::new(3, ABC);
        c.anadir(b"a");
        c.anadir(b"bc");
        assert_eq!(c.terminar(), Ok(()));
    }

    #[test]
    fn a_medias_no() {
        let mut c = Comprobar::new(3, ABC);
        c.anadir(b"ab");
        assert!(c.terminar().unwrap_err().contains("incompleto"));
    }

    #[test]
    fn otro_del_mismo_tamano_no() {
        let mut c = Comprobar::new(3, ABC);
        c.anadir(b"abd");
        assert!(c.terminar().unwrap_err().contains("SHA-256"));
    }

    #[test]
    fn esta_mira_el_tamano() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!esta(dir.path()));
        std::fs::write(ruta(dir.path()), b"a medias").unwrap();
        assert!(!esta(dir.path()));
    }
}
