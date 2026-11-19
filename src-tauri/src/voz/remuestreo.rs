//! Cambiar la tasa de muestreo de una grabación entera.
//!
//! Los micrófonos graban a 44,1 o 48 kHz y Whisper quiere 16 kHz. Se hace
//! aquí, con un filtro *sinc* con ventana de Blackman, en vez de traer una
//! biblioteca: es una grabación corta, mono y de una vez (al soltar ESPACIO),
//! y así se ve qué se le hace al audio.
//!
//! Al bajar de tasa, el filtro corta un poco por debajo de la nueva Nyquist
//! (el 95 %) para que lo que no cabe no vuelva como ruido (*aliasing*).

use std::f64::consts::PI;

/// Pasos de cruce por cero del *sinc* a cada lado: cuanto más, más limpio y
/// más lento. 16 sobra para voz.
const LOBULOS: f64 = 16.0;

/// `muestras` (mono, a `de` Hz) a `a` Hz.
pub fn remuestrear(muestras: &[f32], de: u32, a: u32) -> Vec<f32> {
    if de == a || muestras.is_empty() {
        return muestras.to_vec();
    }
    let razon = a as f64 / de as f64;
    // Frecuencia de corte, como fracción de la Nyquist de la entrada.
    let corte = razon.min(1.0) * 0.95;
    let ancho = (LOBULOS / corte).ceil();
    let n_salida = (muestras.len() as f64 * razon).floor() as usize;
    let ultima = muestras.len() as i64 - 1;

    (0..n_salida)
        .map(|j| {
            let t = j as f64 / razon;
            let desde = ((t - ancho).ceil() as i64).max(0);
            let hasta = ((t + ancho).floor() as i64).min(ultima);
            let mut suma = 0.0;
            for i in desde..=hasta {
                let d = t - i as f64;
                suma += muestras[i as usize] as f64 * corte * sinc(corte * d) * blackman(d / ancho);
            }
            suma as f32
        })
        .collect()
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

/// La ventana de Blackman, centrada: `u` va de -1 a 1.
fn blackman(u: f64) -> f64 {
    0.42 + 0.5 * (PI * u).cos() + 0.08 * (2.0 * PI * u).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seno(frecuencia: f64, tasa: u32, segundos: f64) -> Vec<f32> {
        let n = (tasa as f64 * segundos) as usize;
        (0..n).map(|i| (2.0 * PI * frecuencia * i as f64 / tasa as f64).sin() as f32 * 0.5).collect()
    }

    fn rms(x: &[f32]) -> f64 {
        (x.iter().map(|&v| (v as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
    }

    #[test]
    fn la_voz_pasa_igual() {
        // Un tono de 1 kHz, de 48 kHz a 16 kHz: sale el mismo tono. Se miran
        // las muestras del centro (en los bordes el filtro no tiene vecinos).
        let salida = remuestrear(&seno(1000.0, 48_000, 1.0), 48_000, 16_000);
        assert_eq!(salida.len(), 16_000);
        let ideal = seno(1000.0, 16_000, 1.0);
        let error = (200..15_800).map(|i| (salida[i] - ideal[i]).abs()).fold(0.0, f32::max);
        assert!(error < 0.005, "error máximo {error}");
    }

    #[test]
    fn lo_que_no_cabe_no_vuelve_como_ruido() {
        // 12 kHz no cabe en 16 kHz (la Nyquist es 8 kHz): sin filtro
        // volvería como un tono de 4 kHz con toda su fuerza.
        let salida = remuestrear(&seno(12_000.0, 48_000, 1.0), 48_000, 16_000);
        assert!(rms(&salida[200..15_800]) < 0.002, "rms {}", rms(&salida[200..15_800]));
    }

    #[test]
    fn tasas_que_no_son_multiplo() {
        let salida = remuestrear(&seno(440.0, 44_100, 2.0), 44_100, 16_000);
        assert_eq!(salida.len(), 32_000);
        let ideal = seno(440.0, 16_000, 2.0);
        let error = (200..31_800).map(|i| (salida[i] - ideal[i]).abs()).fold(0.0, f32::max);
        assert!(error < 0.005, "error máximo {error}");
    }

    #[test]
    fn misma_tasa_no_toca_nada() {
        let x = seno(300.0, 16_000, 0.1);
        assert_eq!(remuestrear(&x, 16_000, 16_000), x);
    }
}
