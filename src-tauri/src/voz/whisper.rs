//! El motor de voz (D2): Whisper, en local, con `whisper-rs` (whisper.cpp).
//!
//! El modelo es un fichero `ggml-*.bin` que se carga una vez
//! ([`Whisper::cargar`], tarda) y luego transcribe cada frase
//! ([`Whisper::transcribir`]). Se le puede dar **contexto**: la lista de
//! alumnos de la clase, para que escriba bien los nombres. Devuelve lo
//! [`Oido`]: el texto y lo seguro que estaba de cada letra.

use std::path::Path;

use serde::{Deserialize, Serialize};
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState, WhisperTokenId,
};

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
    /// El token de fin de texto: los especiales van de él en adelante.
    fin: WhisperTokenId,
}

impl Whisper {
    pub fn cargar(modelo: &Path) -> Result<Self, String> {
        // Sin esto, whisper.cpp escribe su diario en la consola.
        whisper_rs::install_logging_hooks();
        let ruta = modelo.to_str().ok_or("la ruta del modelo no es texto válido")?;
        let contexto = WhisperContext::new_with_params(ruta, WhisperContextParameters::default())
            .map_err(|e| format!("no se puede cargar el modelo {}: {e}", modelo.display()))?;
        let estado = contexto.create_state().map_err(|e| format!("no se puede preparar el modelo: {e}"))?;
        Ok(Self { estado, hilos: hilos(), recortar: true, fin: contexto.token_eot() })
    }

    /// `audio`: mono, a 16 kHz. `contexto`: texto que Whisper toma como lo
    /// dicho justo antes (los nombres de la clase), o nada.
    pub fn transcribir(&mut self, audio: &[f32], contexto: Option<&str>) -> Result<Oido, String> {
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
        // El texto se arma con los tokens, no con el del segmento, para saber
        // de qué token sale cada letra y con qué probabilidad. Un token puede
        // partir una letra con tilde en dos: se juntan los bytes y cada byte
        // se lleva la probabilidad de su token.
        let mut bytes = Vec::new();
        let mut p_byte = Vec::new();
        for segmento in self.estado.as_iter() {
            for i in 0..segmento.n_tokens() {
                let Some(token) = segmento.get_token(i) else { continue };
                // Los especiales ([_BEG_], [_TT_…], fin de texto) van detrás
                // del fin de texto en el vocabulario.
                if token.token_id() >= self.fin {
                    continue;
                }
                let b = token.to_bytes().map_err(|e| e.to_string())?;
                bytes.extend_from_slice(b);
                p_byte.extend(std::iter::repeat_n(token.token_probability(), b.len()));
            }
        }
        Ok(Oido::de_bytes(&bytes, &p_byte))
    }
}

/// Lo que oyó Whisper: el texto y, por cada letra, la probabilidad que le
/// dio al token del que sale (0–1).
///
/// **No sirve para decidir si preguntar** (prueba de H3b): Whisper reparte
/// la probabilidad entre maneras de escribir lo mismo («10» o «diez», «9,5»
/// o «9.5»), y el único nombre mal oído tenía la misma que los bien oídos.
/// Se guarda para volver a mirarlo con otro modelo u otro ordenador
/// (`examples/probar_entender`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Oido {
    pub texto: String,
    /// Una por cada `char` de `texto`.
    pub confianza: Vec<f32>,
}

impl Oido {
    /// Si no se ha dicho nada: ni una letra ni una cifra fuera de lo que
    /// Whisper pone entre corchetes o paréntesis cuando no oye voz
    /// («[Música]», «(Aplausos)»; pasó en la prueba a mano de H3a,
    /// 2026-11-19). Es silencio, no una frase sin entender.
    pub fn sin_palabras(&self) -> bool {
        let mut dentro = 0usize;
        for c in self.texto.chars() {
            match c {
                '[' | '(' => dentro += 1,
                ']' | ')' => dentro = dentro.saturating_sub(1),
                c if dentro == 0 && c.is_alphanumeric() => return false,
                _ => {}
            }
        }
        true
    }

    /// De los bytes de los tokens a letras: cada letra se queda con la menor
    /// probabilidad de sus bytes. Sin los espacios de los extremos.
    fn de_bytes(bytes: &[u8], p_byte: &[f32]) -> Self {
        let texto = String::from_utf8_lossy(bytes);
        let letras: Vec<(char, f32)> = if texto.len() == bytes.len() {
            texto
                .char_indices()
                .map(|(i, c)| (c, p_byte[i..i + c.len_utf8()].iter().copied().fold(1.0, f32::min)))
                .collect()
        } else {
            // Bytes que no son UTF-8 (no debería pasar): sin confianza fiable,
            // se toma la menor de toda la frase.
            let p = p_byte.iter().copied().fold(1.0, f32::min);
            texto.chars().map(|c| (c, p)).collect()
        };
        let inicio = letras.iter().position(|(c, _)| !c.is_whitespace()).unwrap_or(letras.len());
        let fin = letras.iter().rposition(|(c, _)| !c.is_whitespace()).map_or(inicio, |f| f + 1);
        let letras = &letras[inicio..fin];
        Self { texto: letras.iter().map(|(c, _)| c).collect(), confianza: letras.iter().map(|(_, p)| *p).collect() }
    }
}

/// Un hilo por núcleo físico, que es donde whisper.cpp rinde más: la mitad
/// de los lógicos (con *hyperthreading*), y no más de 8.
fn hilos() -> i32 {
    let logicos = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    (logicos / 2).clamp(1, 8) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn una_letra_partida_en_dos_tokens_se_queda_la_menor() {
        // « Martí» + «n,»: la «í» son dos bytes, cada uno de un token.
        let bytes = " Martín".as_bytes();
        let partida = 6; // a mitad de la «í»
        let p: Vec<f32> = (0..bytes.len()).map(|i| if i < partida { 0.9 } else { 0.4 }).collect();
        let o = Oido::de_bytes(bytes, &p);
        assert_eq!(o.texto, "Martín");
        assert_eq!(o.confianza, vec![0.9, 0.9, 0.9, 0.9, 0.4, 0.4]);
    }

    #[test]
    fn lo_que_whisper_pone_sin_voz_no_son_palabras() {
        let oido = |t: &str| Oido { texto: t.into(), confianza: vec![] };
        for t in ["", "  ", "[Música]", "(Aplausos)", "[Música] [Risas]", "♪", "..."] {
            assert!(oido(t).sin_palabras(), "{t}");
        }
        for t in ["Casals ocho", "[Música] Casals ocho", "7", "(Ruido) Lucía 9"] {
            assert!(!oido(t).sin_palabras(), "{t}");
        }
    }
}
