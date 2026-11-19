//! Lo que comparten `grabar_frases` y `probar_voz`: el corpus de la prueba
//! de voz (H3a) y dónde van las grabaciones.
//!
//! El corpus vive en `tests/datos/voz/`: una clase **inventada**
//! (`clase.txt`) y 50 frases dichas a esa clase (`frases.tsv`). Las
//! grabaciones **no** van al repo: van a `Documentos\Gestor de Notas\prueba-voz`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub struct Frase {
    pub n: u32,
    /// El alumno de la clase al que va la nota, con el nombre completo.
    pub alumno: String,
    /// Lo que se dice de su nombre («Lucía Martínez», «Popescu»).
    pub dicho: String,
    pub nota: f64,
    /// La frase entera, tal como hay que decirla.
    pub frase: String,
}

fn datos() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datos/voz")
}

pub fn frases() -> Vec<Frase> {
    let texto = std::fs::read_to_string(datos().join("frases.tsv")).expect("no se puede leer frases.tsv");
    texto
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            assert_eq!(c.len(), 5, "frases.tsv: línea mal formada: {l}");
            Frase {
                n: c[0].parse().expect("n"),
                alumno: c[1].into(),
                dicho: c[2].into(),
                nota: c[3].parse().expect("nota"),
                frase: c[4].into(),
            }
        })
        .collect()
}

pub fn clase() -> Vec<String> {
    std::fs::read_to_string(datos().join("clase.txt"))
        .expect("no se puede leer clase.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// La carpeta de las grabaciones: la que se pase, o
/// `Documentos\Gestor de Notas\prueba-voz`.
pub fn carpeta(argumento: Option<&str>) -> PathBuf {
    match argumento {
        Some(c) => PathBuf::from(c),
        None => dirs::document_dir()
            .expect("no se encuentra la carpeta Documentos")
            .join("Gestor de Notas")
            .join("prueba-voz"),
    }
}

pub fn wav(carpeta: &Path, n: u32) -> PathBuf {
    carpeta.join(format!("{n:02}.wav"))
}

/// Una grabación: mono, a 16 kHz, como las deja `grabar_frases`.
pub fn leer_wav(ruta: &Path) -> Option<Vec<f32>> {
    let mut r = hound::WavReader::open(ruta).ok()?;
    let spec = r.spec();
    assert_eq!((spec.channels, spec.sample_rate), (1, 16_000), "{}: no es mono a 16 kHz", ruta.display());
    Some(r.samples::<i16>().filter_map(Result::ok).map(|m| m as f32 / i16::MAX as f32).collect())
}
