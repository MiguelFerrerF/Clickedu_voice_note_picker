//! La voz (H3): del micrófono al texto.
//!
//! - [`microfono`] — grabar mientras se mantiene ESPACIO.
//! - [`remuestreo`] — pasar lo grabado a 16 kHz, que es lo que pide Whisper.
//! - [`whisper`] — el motor (D2): Whisper **local**. La voz, que lleva
//!   nombres de menores, **no sale del ordenador**.
//!
//! Entender la frase («Elena Gómez siete con cinco» → alumno y nota) es de
//! H3b y llegará en su propio módulo.

pub mod microfono;
pub mod remuestreo;
pub mod whisper;

/// La tasa con la que trabaja Whisper: 16 000 muestras por segundo, mono.
pub const TASA: u32 = 16_000;
