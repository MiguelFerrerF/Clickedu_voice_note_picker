//! Graba las 50 frases de la prueba de voz (H3a), una a una: **ESPACIO para
//! empezar, ESPACIO para terminar**.
//!
//! En la app se mantendrá ESPACIO, pero la consola de Windows no sirve para
//! eso: al mantener una tecla manda «pulsada» y «soltada» con cada
//! repetición, y la grabación se cortaba a los pocos milisegundos. La
//! ventana de la app (el *webview*) sí distingue pulsar de soltar.
//!
//!     cargo run --release --example grabar_frases [-- <carpeta>]
//!
//! Cada frase va a `NN.wav` (16 kHz, mono) en `Documentos\Gestor de
//! Notas\prueba-voz`, o en la carpeta que se pase. Empieza por la primera que
//! falte; con ← se vuelve a una para repetirla. Las grabaciones **no** van
//! al repo: la clase es inventada, pero la voz es tuya.
//!
//! Se graba con [`Microfono`], lo mismo que usará la app (abierto todo el
//! rato, y con los 0,3 s de antes de pulsar), así que lo que se corte al
//! pulsar ESPACIO también se nota en la prueba.

#[path = "comun/corpus.rs"]
mod corpus;

use std::io::{stdout, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::{cursor, execute, terminal};
use gestor_de_notas_lib::voz::microfono::Microfono;
use gestor_de_notas_lib::voz::TASA;

fn main() {
    let argumento = std::env::args().nth(1);
    let carpeta = corpus::carpeta(argumento.as_deref());
    std::fs::create_dir_all(&carpeta).expect("no se puede crear la carpeta de grabaciones");
    let frases = corpus::frases();

    let mut i = frases.iter().position(|f| !corpus::wav(&carpeta, f.n).exists()).unwrap_or(0);
    let microfono = Microfono::abrir(None).unwrap_or_else(|e| panic!("{e}"));
    let mut aviso = format!("Micrófono: «{}»", microfono.nombre);

    terminal::enable_raw_mode().expect("no se puede preparar la consola");
    loop {
        pintar(&carpeta, &frases, i, &aviso);
        let Some(tecla) = tecla() else { continue };
        aviso.clear();
        match (tecla.code, tecla.kind) {
            (KeyCode::Char(' '), KeyEventKind::Press) => {
                aviso = grabar(&microfono, &carpeta, &frases[i]);
                if aviso.starts_with('✓') && i + 1 < frases.len() {
                    i += 1;
                }
            }
            (KeyCode::Right | KeyCode::Enter, KeyEventKind::Press) if i + 1 < frases.len() => i += 1,
            (KeyCode::Left, KeyEventKind::Press) if i > 0 => i -= 1,
            (KeyCode::Char('q') | KeyCode::Esc, KeyEventKind::Press) => break,
            _ => {}
        }
    }
    terminal::disable_raw_mode().ok();
    let hechas = frases.iter().filter(|f| corpus::wav(&carpeta, f.n).exists()).count();
    println!("\n{hechas} de {} frases grabadas en {}", frases.len(), carpeta.display());
}

fn tecla() -> Option<KeyEvent> {
    match event::read().ok()? {
        Event::Key(k) => Some(k),
        _ => None,
    }
}

fn pintar(carpeta: &Path, frases: &[corpus::Frase], i: usize, aviso: &str) {
    let f = &frases[i];
    let hechas = frases.iter().filter(|f| corpus::wav(carpeta, f.n).exists()).count();
    let estado = if corpus::wav(carpeta, f.n).exists() { "ya grabada (si la grabas otra vez, se sustituye)" } else { "sin grabar" };
    let mut s = stdout();
    execute!(s, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0)).ok();
    let lineas = [
        format!("Prueba de voz · frase {} de {} · {hechas} grabadas", f.n, frases.len()),
        String::new(),
        format!("    «{}»", f.frase),
        String::new(),
        format!("    {estado}"),
        String::new(),
        "ESPACIO para empezar, dila, ESPACIO para terminar · ← → para moverte · Q para salir".into(),
        String::new(),
        aviso.into(),
    ];
    for l in lineas {
        write!(s, "{l}\r\n").ok();
    }
    s.flush().ok();
}

/// Graba hasta la siguiente pulsación de ESPACIO (o Intro) y guarda el
/// `.wav`. Devuelve qué ha pasado, para enseñarlo.
fn grabar(microfono: &Microfono, carpeta: &Path, frase: &corpus::Frase) -> String {
    microfono.empezar(|_| {});
    let empezo = Instant::now();
    let mut s = stdout();
    write!(s, "● grabando… pulsa ESPACIO al terminar\r\n").ok();
    s.flush().ok();
    loop {
        let Some(t) = tecla() else { continue };
        // Las «soltadas» y las repeticiones de mantener la tecla no cuentan,
        // ni lo que llegue en el primer medio segundo: es la misma pulsación.
        if t.kind != KeyEventKind::Press || empezo.elapsed() < Duration::from_millis(500) {
            continue;
        }
        if matches!(t.code, KeyCode::Char(' ') | KeyCode::Enter) {
            break;
        }
    }
    let audio = microfono.terminar();
    tragar_teclas();
    let segundos = audio.len() as f32 / TASA as f32;
    // Con los 0,3 s de antes, menos de 0,8 s es que no ha dado tiempo a decir nada.
    if segundos < 0.8 {
        return format!("✗ demasiado corta ({segundos:.1} s): pulsa ESPACIO, dila entera y pulsa ESPACIO");
    }
    let pico = audio.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if let Err(e) = guardar(&corpus::wav(carpeta, frase.n), &audio) {
        return format!("✗ no se puede guardar: {e}");
    }
    let flojo = if pico < 0.05 { " · muy flojo: ¿es el micrófono que toca?" } else { "" };
    format!("✓ frase {} grabada: {segundos:.1} s, pico {:.0} %{flojo}", frase.n, pico * 100.0)
}

/// Tira lo que quede de la pulsación que terminó (sus repeticiones y su
/// «soltada»), para que no empiece otra grabación sin querer.
fn tragar_teclas() {
    while event::poll(Duration::from_millis(300)).unwrap_or(false) {
        let _ = event::read();
    }
}

fn guardar(ruta: &Path, audio: &[f32]) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TASA,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(ruta, spec).map_err(|e| e.to_string())?;
    for &m in audio {
        w.write_sample((m.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).map_err(|e| e.to_string())?;
    }
    w.finalize().map_err(|e| e.to_string())
}
