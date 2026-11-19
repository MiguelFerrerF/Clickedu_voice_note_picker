//! El motor de voz (D2): Whisper, en local, con `whisper-rs` (whisper.cpp).
//!
//! El modelo es un fichero `ggml-*.bin` que se carga una vez
//! ([`Whisper::cargar`], tarda) y luego transcribe cada frase
//! ([`Whisper::transcribir`]). Se le puede dar **contexto**: la lista de
//! alumnos de la clase, para que escriba bien los nombres.

use std::path::Path;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

use super::TASA;

/// Con la ventana recortada, los segundos de más que se le dejan: con 1 s
/// acertaba un 88 %; con 2 s, un 90–94 %, por 0,15 s más; con 3 o 4, nada
/// más (prueba de H3a).
const MARGEN: f32 = 2.0;

pub struct Whisper {
    estado: WhisperState,
    hilos: i32,
    /// Procesar solo lo que dura la frase, no la ventana de 30 s con la que
    /// se entrenó Whisper. Mucho más rápido con frases cortas; la prueba de
    /// H3a dice si se pierde algo.
    pub recortar: bool,
}

impl Whisper {
    pub fn cargar(modelo: &Path) -> Result<Self, String> {
        // Sin esto, whisper.cpp escribe su diario en la consola.
        whisper_rs::install_logging_hooks();
        let ruta = modelo.to_str().ok_or("la ruta del modelo no es texto válido")?;
        let contexto = WhisperContext::new_with_params(ruta, WhisperContextParameters::default())
            .map_err(|e| format!("no se puede cargar el modelo {}: {e}", modelo.display()))?;
        let estado = contexto.create_state().map_err(|e| format!("no se puede preparar el modelo: {e}"))?;
        Ok(Self { estado, hilos: hilos(), recortar: true })
    }

    /// `audio`: mono, a 16 kHz. `contexto`: texto que Whisper toma como lo
    /// dicho justo antes (los nombres de la clase), o nada.
    pub fn transcribir(&mut self, audio: &[f32], contexto: Option<&str>) -> Result<String, String> {
        // Buscar con haz (5) no acierta más y tarda un 30 % más (prueba de H3a).
        let mut p = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        p.set_language(Some("es"));
        p.set_n_threads(self.hilos);
        p.set_no_context(true);
        p.set_single_segment(true);
        p.set_no_timestamps(true);
        p.set_temperature(0.0);
        // Sin reintentos: si la frase le sale dudosa, whisper.cpp la vuelve a
        // decodificar subiendo la temperatura, hasta cinco veces más, y a
        // veces se queda repitiéndola. Una frase de nota cabe de sobra en 32
        // tokens.
        p.set_temperature_inc(0.0);
        p.set_max_tokens(32);
        p.set_print_special(false);
        p.set_print_progress(false);
        p.set_print_realtime(false);
        p.set_print_timestamps(false);
        if let Some(c) = contexto {
            p.set_initial_prompt(c);
        }

        // whisper.cpp no transcribe menos de un segundo: se rellena con
        // silencio.
        let minimo = TASA as usize * 11 / 10;
        let mut relleno;
        let audio = if audio.len() < minimo {
            relleno = audio.to_vec();
            relleno.resize(minimo, 0.0);
            &relleno[..]
        } else {
            audio
        };

        if self.recortar {
            // 1500 posiciones son 30 s: 50 por segundo, más el margen.
            let segundos = audio.len() as f32 / TASA as f32;
            p.set_audio_ctx((((segundos + MARGEN) * 50.0).ceil() as i32).min(1500));
        }

        self.estado.full(p, audio).map_err(|e| format!("Whisper no ha podido transcribir: {e}"))?;
        let mut texto = String::new();
        for segmento in self.estado.as_iter() {
            texto.push_str(&segmento.to_str_lossy().map_err(|e| e.to_string())?);
        }
        Ok(texto.trim().to_string())
    }
}

/// Un hilo por núcleo físico, que es donde whisper.cpp rinde más: la mitad
/// de los lógicos (con *hyperthreading*), y no más de 8.
fn hilos() -> i32 {
    let logicos = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    (logicos / 2).clamp(1, 8) as i32
}
