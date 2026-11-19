//! El micrófono, para grabar mientras se mantiene ESPACIO.
//!
//! Abrir el micrófono de Windows tarda de 0,2 a 0,4 s: si se abriera al
//! pulsar ESPACIO, se perdería el principio del nombre. Por eso
//! [`Microfono::abrir`] lo deja abierto, guardando **solo los últimos
//! 0,3 s** en memoria, y [`Microfono::empezar`] empieza la grabación con ese
//! trocito de antes. Nada se escribe en disco, y lo que no se graba se tira.
//!
//! Mientras graba, avisa del **nivel** 20 veces por segundo (HUI): la
//! interfaz enseña que el micrófono oye, y si no se mueve, es que no llega.
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

/// Por debajo de este pico, lo grabado es silencio. En las 100 grabaciones
/// de H3a, el ruido de antes de hablar llega a 0,0002 de pico (mediana) y la
/// frase más baja, a 0,045: Whisper, con silencio, se inventa frases
/// («Subtítulos realizados por…»), y así no se le pasa.
const SILENCIO: f32 = 0.01;

/// Si en lo grabado hay algo más que silencio: si no, el micrófono no
/// llega (está apagado, es otro, o no se ha hablado).
pub fn hay_voz(audio: &[f32]) -> bool {
    audio.iter().any(|m| m.abs() >= SILENCIO)
}

/// Cada cuánto se avisa del nivel mientras se graba, en segundos.
const CADA: f32 = 0.05;

/// Lo que se hace con cada nivel.
type Avisar = Arc<dyn Fn(f32) + Send + Sync>;

/// El pico de un trozo, de 0 a 1 **en decibelios**: -60 dB o menos es 0, y
/// 0 dB es 1. Como se oye: la frase más baja de H3a (0,045 de pico, -27 dB)
/// pasa de la mitad, el umbral del silencio (-40 dB) se queda en un tercio, y
/// el ruido de antes de hablar (0,0002, -74 dB) no se ve.
pub fn nivel(pico: f32) -> f32 {
    if pico <= 0.0 {
        return 0.0;
    }
    ((20.0 * pico.log10() + 60.0) / 60.0).clamp(0.0, 1.0)
}

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
    /// Mientras se graba, el medidor del nivel.
    medidor: Option<Medidor>,
}

/// El pico de cada `cada` muestras, para avisar del nivel.
struct Medidor {
    cada: usize,
    cuenta: usize,
    pico: f32,
    avisar: Avisar,
}

impl Estado {
    /// Guarda un paquete del micrófono y devuelve el último nivel que se ha
    /// completado en él, si alguno, con quién avisa: se avisa **fuera del
    /// cerrojo**, para no hacer esperar al micrófono.
    fn recibir(&mut self, mono: impl Iterator<Item = f32>) -> Option<(f32, Avisar)> {
        let mut aviso = None;
        match self.medidor.as_mut() {
            None => self.muestras.extend(mono),
            Some(m) => {
                for muestra in mono {
                    self.muestras.push(muestra);
                    m.pico = m.pico.max(muestra.abs());
                    m.cuenta += 1;
                    if m.cuenta >= m.cada {
                        aviso = Some(nivel(m.pico));
                        m.cuenta = 0;
                        m.pico = 0.0;
                    }
                }
            }
        }
        // Sin grabar, solo lo último; se recorta de vez en cuando, no en cada
        // paquete.
        if !self.grabando && self.muestras.len() > self.antes * 2 {
            let sobra = self.muestras.len() - self.antes;
            self.muestras.drain(..sobra);
        }
        aviso.zip(self.medidor.as_ref().map(|m| m.avisar.clone()))
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

    /// Empieza a grabar, con los últimos 0,3 s de antes. Hasta
    /// [`terminar`](Self::terminar), `avisar` recibe el [`nivel`] cada 50 ms.
    pub fn empezar(&self, avisar: impl Fn(f32) + Send + Sync + 'static) {
        let mut e = self.estado.lock().unwrap_or_else(|p| p.into_inner());
        let sobra = e.muestras.len().saturating_sub(e.antes);
        e.muestras.drain(..sobra);
        e.grabando = true;
        e.medidor = Some(Medidor {
            cada: ((self.tasa as f32 * CADA) as usize).max(1),
            cuenta: 0,
            pico: 0.0,
            avisar: Arc::new(avisar),
        });
    }

    /// Para y devuelve lo grabado desde [`empezar`](Self::empezar): mono, a
    /// 16 kHz, de -1 a 1.
    pub fn terminar(&self) -> Vec<f32> {
        let grabado = {
            let mut e = self.estado.lock().unwrap_or_else(|p| p.into_inner());
            e.grabando = false;
            e.medidor = None;
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
            let aviso = estado.lock().ok().and_then(|mut e| {
                e.recibir(
                    datos
                        .chunks(canales)
                        .map(|trama| trama.iter().map(|&m| f32::from_sample(m)).sum::<f32>() / canales as f32),
                )
            });
            if let Some((n, avisar)) = aviso {
                avisar(n);
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

    #[test]
    fn grabando_avisa_del_ultimo_trozo_completo() {
        let oidos = Arc::new(Mutex::new(Vec::new()));
        let apuntar = oidos.clone();
        let avisar: Avisar = Arc::new(move |n| apuntar.lock().unwrap().push(n));
        let mut e = Estado { antes: 10, grabando: true, ..Default::default() };
        e.medidor = Some(Medidor { cada: 4, cuenta: 0, pico: 0.0, avisar });
        // Un trozo a 0 dB y otro a -40 dB, y medio trozo.
        let (n, avisar) = e.recibir([0.0, -1.0, 0.5, 0.0, 0.01, 0.0, 0.0, 0.0, 0.5, 0.0].into_iter()).expect("avisa");
        // Del último completo: el de antes, en un paquete de 10 ms y trozos
        // de 50 ms, no pasa.
        assert!((n - 1.0 / 3.0).abs() < 1e-4, "{n}");
        avisar(n);
        assert_eq!(oidos.lock().unwrap().len(), 1);
        // El medio trozo sigue contando: su pico (0,5) no se ha perdido.
        let (n, _) = e.recibir([0.0, 0.0].into_iter()).expect("completa el trozo");
        assert!((n - nivel(0.5)).abs() < 1e-6, "{n}");
        assert_eq!(e.muestras.len(), 12, "grabando no se tira nada");
        // Sin medidor no hay aviso.
        e.medidor = None;
        assert!(e.recibir([1.0; 8].into_iter()).is_none());
    }

    #[test]
    fn el_nivel_va_en_decibelios() {
        assert_eq!(nivel(0.0), 0.0);
        assert_eq!(nivel(0.0002), 0.0, "el ruido de antes de hablar no se ve");
        assert!((nivel(SILENCIO) - 1.0 / 3.0).abs() < 1e-4);
        assert!(nivel(0.045) > 0.5, "la frase más baja de H3a se ve");
        assert_eq!(nivel(1.0), 1.0);
        assert_eq!(nivel(3.0), 1.0);
    }

    #[test]
    fn el_silencio_no_es_voz() {
        assert!(!hay_voz(&[]));
        assert!(!hay_voz(&[0.0002, -0.003, 0.001]));
        assert!(hay_voz(&[0.0002, -0.045, 0.001]));
    }

    /// Necesita un micrófono: `cargo test -- --ignored microfono`.
    #[test]
    #[ignore]
    fn graba_un_segundo_mas_lo_de_antes() {
        let m = Microfono::abrir().expect("no se abre el micrófono");
        std::thread::sleep(std::time::Duration::from_millis(500));
        let avisos = Arc::new(Mutex::new(0));
        let contar = avisos.clone();
        m.empezar(move |_| *contar.lock().unwrap() += 1);
        std::thread::sleep(std::time::Duration::from_secs(1));
        let audio = m.terminar();
        // 1 s + 0,3 s de antes, con lo que tarde en llegar el último paquete.
        assert!((19_000..22_500).contains(&audio.len()), "{} muestras", audio.len());
        assert!((15..=21).contains(&*avisos.lock().unwrap()), "unos 20 niveles por segundo");
    }
}
