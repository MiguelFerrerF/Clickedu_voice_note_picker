//! El micrófono, para grabar mientras se mantiene ESPACIO.
//!
//! Abrir el micrófono de Windows tarda de 0,2 a 0,4 s: si se abriera al
//! pulsar ESPACIO, se perdería el principio del nombre. Por eso
//! [`Microfono::abrir`] lo deja abierto, guardando **solo los últimos
//! 0,3 s** en memoria, y [`Microfono::empezar`] empieza la grabación con ese
//! trocito de antes. Nada se escribe en disco, y lo que no se graba se tira.
//!
//! El *stream* de `cpal` no se puede pasar entre hilos: vive en un hilo
//! propio hasta que se suelta el [`Microfono`].

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};

use super::remuestreo::remuestrear;
use super::TASA;

/// Lo que se guarda de antes de pulsar ESPACIO, en segundos.
const ANTES: f32 = 0.3;

/// El micrófono abierto.
pub struct Microfono {
    estado: Arc<Mutex<Estado>>,
    tasa: u32,
    cerrar: Option<mpsc::Sender<()>>,
    hilo: Option<JoinHandle<()>>,
    /// El nombre del micrófono, para decir cuál se está usando.
    pub nombre: String,
}

#[derive(Default)]
struct Estado {
    /// Mono, a la tasa del micrófono.
    muestras: Vec<f32>,
    grabando: bool,
    /// Cuántas muestras de antes se guardan.
    antes: usize,
}

impl Estado {
    fn recibir(&mut self, mono: impl Iterator<Item = f32>) {
        self.muestras.extend(mono);
        // Sin grabar, solo lo último; se recorta de vez en cuando, no en cada
        // paquete.
        if !self.grabando && self.muestras.len() > self.antes * 2 {
            let sobra = self.muestras.len() - self.antes;
            self.muestras.drain(..sobra);
        }
    }
}

impl Microfono {
    /// Abre el micrófono por defecto de Windows. Vuelve cuando ya escucha, o
    /// con el motivo si no puede (no hay micrófono, Windows no deja usarlo).
    pub fn abrir() -> Result<Self, String> {
        let estado: Arc<Mutex<Estado>> = Arc::default();
        let (cerrar, al_cerrar) = mpsc::channel::<()>();
        let (listo, al_listo) = mpsc::channel::<Result<(u32, String), String>>();

        let para_el_hilo = estado.clone();
        let hilo = std::thread::spawn(move || match abrir(para_el_hilo) {
            Ok((stream, tasa, nombre)) => {
                let _ = listo.send(Ok((tasa, nombre)));
                // Abierto hasta que se suelte el `Microfono`.
                let _ = al_cerrar.recv();
                drop(stream);
            }
            Err(e) => {
                let _ = listo.send(Err(e));
            }
        });

        match al_listo.recv() {
            Ok(Ok((tasa, nombre))) => Ok(Self { estado, tasa, cerrar: Some(cerrar), hilo: Some(hilo), nombre }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("el hilo del micrófono se ha caído al abrirlo".into()),
        }
    }

    /// Empieza a grabar, con los últimos 0,3 s de antes.
    pub fn empezar(&self) {
        let mut e = self.estado.lock().unwrap_or_else(|p| p.into_inner());
        let sobra = e.muestras.len().saturating_sub(e.antes);
        e.muestras.drain(..sobra);
        e.grabando = true;
    }

    /// Para y devuelve lo grabado desde [`empezar`](Self::empezar): mono, a
    /// 16 kHz, de -1 a 1.
    pub fn terminar(&self) -> Vec<f32> {
        let grabado = {
            let mut e = self.estado.lock().unwrap_or_else(|p| p.into_inner());
            e.grabando = false;
            std::mem::take(&mut e.muestras)
        };
        remuestrear(&grabado, self.tasa, TASA)
    }
}

impl Drop for Microfono {
    fn drop(&mut self) {
        drop(self.cerrar.take());
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

/// Abre el micrófono por defecto con su formato por defecto: no se le pide
/// 16 kHz porque muchos no lo dan; se remuestrea después.
fn abrir(estado: Arc<Mutex<Estado>>) -> Result<(cpal::Stream, u32, String), String> {
    let host = cpal::default_host();
    let dispositivo = host.default_input_device().ok_or("no hay ningún micrófono")?;
    let nombre = dispositivo.description().map(|d| d.to_string()).unwrap_or_else(|_| "micrófono".into());
    let config = dispositivo
        .default_input_config()
        .map_err(|e| format!("no se puede usar el micrófono «{nombre}»: {e}"))?;
    let tasa = config.sample_rate();
    let canales = config.channels() as usize;
    let formato = config.sample_format();
    let config: cpal::StreamConfig = config.into();
    estado.lock().map_err(|e| e.to_string())?.antes = (tasa as f32 * ANTES) as usize;

    let stream = match formato {
        SampleFormat::F32 => construir::<f32>(&dispositivo, &config, canales, estado),
        SampleFormat::I16 => construir::<i16>(&dispositivo, &config, canales, estado),
        SampleFormat::I32 => construir::<i32>(&dispositivo, &config, canales, estado),
        SampleFormat::U16 => construir::<u16>(&dispositivo, &config, canales, estado),
        otro => return Err(format!("el micrófono «{nombre}» graba en un formato que no se conoce ({otro})")),
    }
    .map_err(|e| format!("no se puede abrir el micrófono «{nombre}»: {e}"))?;
    stream.play().map_err(|e| format!("el micrófono «{nombre}» no arranca: {e}"))?;
    Ok((stream, tasa, nombre))
}

/// El *stream* que junta los canales en uno (la media) y lo va guardando.
fn construir<T>(
    dispositivo: &cpal::Device,
    config: &cpal::StreamConfig,
    canales: usize,
    estado: Arc<Mutex<Estado>>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    dispositivo.build_input_stream(
        config.clone(),
        move |datos: &[T], _: &_| {
            if let Ok(mut e) = estado.lock() {
                e.recibir(
                    datos
                        .chunks(canales)
                        .map(|trama| trama.iter().map(|&m| f32::from_sample(m)).sum::<f32>() / canales as f32),
                );
            }
        },
        |e| eprintln!("micrófono: {e}"),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_grabar_solo_guarda_lo_ultimo() {
        let mut e = Estado { antes: 100, ..Default::default() };
        for _ in 0..50 {
            e.recibir((0..48).map(|i| i as f32));
        }
        assert!(e.muestras.len() <= 200);
        e.grabando = true;
        for _ in 0..50 {
            e.recibir((0..48).map(|i| i as f32));
        }
        assert!(e.muestras.len() >= 50 * 48, "grabando no se tira nada");
    }

    /// Necesita un micrófono: `cargo test -- --ignored microfono`.
    #[test]
    #[ignore]
    fn graba_un_segundo_mas_lo_de_antes() {
        let m = Microfono::abrir().expect("no se abre el micrófono");
        std::thread::sleep(std::time::Duration::from_millis(500));
        m.empezar();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let audio = m.terminar();
        // 1 s + 0,3 s de antes, con lo que tarde en llegar el último paquete.
        assert!((19_000..22_500).contains(&audio.len()), "{} muestras", audio.len());
    }
}
