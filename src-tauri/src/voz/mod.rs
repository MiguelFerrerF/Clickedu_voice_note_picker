//! La voz (H3): del micrófono al texto.
//!
//! - [`microfono`] — grabar mientras se mantiene ESPACIO.
//! - [`remuestreo`] — pasar lo grabado a 16 kHz, que es lo que pide Whisper.
//! - [`whisper`] — el motor (D2): Whisper **local**. La voz, que lleva
//!   nombres de menores, **no sale del ordenador**.
//! - [`modelo`] — el modelo de Whisper: dónde está y cómo se comprueba al
//!   descargarlo.
//! - [`entender`] — de lo oído a alumno y nota (H3b), con el
//!   [`interprete`] (la nota y las palabras del nombre) y [`emparejar`]
//!   (quién de la clase es).

pub mod emparejar;
pub mod entender;
pub mod interprete;
pub mod microfono;
pub mod modelo;
pub mod remuestreo;
pub mod whisper;

/// La tasa con la que trabaja Whisper: 16 000 muestras por segundo, mono.
pub const TASA: u32 = 16_000;
