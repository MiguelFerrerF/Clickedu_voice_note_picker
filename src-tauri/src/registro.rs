//! El registro (D8): lo que la app no ha entendido, lo que ha preguntado,
//! lo que el profesor ha corregido y los fallos, **guardado en local** para
//! mejorar la app con casos reales.
//!
//! - Se guarda en la carpeta de la app (`%APPDATA%\es.gestornotas.app\
//!   registro\`, D4), un fichero por mes (`casos-2026-11.jsonl`, una línea
//!   por caso), y lo de hace más de [`MESES`] meses se borra solo. Desde
//!   Ajustes se ve cuánto hay y se borra.
//! - **La respuesta buena la da el profesor sin hacer nada aparte**: cada
//!   dictado lleva un `id`, y lo que se hace después con él (elegir la
//!   segunda opción, descartar, deshacer, corregir la nota a mano) se
//!   apunta como [`Respuesta`].
//! - **De aquí no sale nada**: lo que sale es el [`informe`], sin nombres ni
//!   voz, y solo si el profesor lo prepara y lo manda.
//!
//! Apuntar nunca para la app: si no se puede escribir, se dice por la
//! consola y se sigue.

pub mod informe;

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::voz::entender::Entendido;

/// Los meses que se guardan.
pub const MESES: u32 = 6;

/// La versión de la app, que va en cada caso.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Caso {
    /// Segundos desde 1970, en UTC.
    pub cuando: u64,
    pub version: String,
    #[serde(flatten)]
    pub que: Que,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Que {
    /// Una frase dictada y lo que entendió la app.
    Dictado {
        dictado: Dictado,
        /// La plantilla de la clase.
        fichero: String,
        /// Las palabras que se tomaron por el nombre.
        nombre: Vec<String>,
        /// Los alumnos más parecidos, con sus números.
        candidatos: Vec<Candidato>,
    },
    /// Lo que hizo el profesor con el dictado `id`.
    Respuesta { id: String, respuesta: Respuesta },
    /// Un error: dónde y cuál.
    Fallo { donde: String, error: String },
}

/// Lo entendido de una frase, con cada alumno por su **clave** de la
/// sesión. Es lo que recibe la interfaz y lo que se guarda.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dictado {
    pub id: String,
    /// Lo que se oyó (o se escribió), tal cual.
    pub oido: String,
    #[serde(flatten)]
    pub entendido: Entendido<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidato {
    pub clave: String,
    pub parecido: f32,
    pub distancia: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Respuesta {
    /// Eligió una de las opciones de una duda.
    Elegida { clave: String, nota: f64 },
    /// Descartó la duda: no era nadie, o no era eso.
    Descartada,
    /// Deshizo lo que se apuntó.
    Deshecha,
    /// Escribió a mano otra nota en el alumno recién dictado.
    Corregida { nota: Option<f64> },
}

/// Cuánto hay en el registro, para Ajustes.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resumen {
    pub dictados: usize,
    /// Apuntados sin preguntar.
    pub seguros: usize,
    pub preguntados: usize,
    pub no_entendidos: usize,
    /// Deshechos o corregidos: lo que la app apuntó mal.
    pub corregidos: usize,
    pub fallos: usize,
    /// El caso más antiguo.
    pub desde: Option<u64>,
}

pub struct Registro {
    carpeta: PathBuf,
    cerrojo: Mutex<()>,
    contador: AtomicU64,
}

impl Registro {
    pub fn new(carpeta: PathBuf) -> Self {
        Self { carpeta, cerrojo: Mutex::new(()), contador: AtomicU64::new(0) }
    }

    pub fn carpeta(&self) -> &Path {
        &self.carpeta
    }

    /// Un `id` para un dictado, que no se repite.
    pub fn nuevo_id(&self) -> String {
        let n = self.contador.fetch_add(1, Ordering::Relaxed);
        let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        format!("{ms}-{n}")
    }

    /// Apunta un caso. Si no se puede, lo dice por la consola y sigue.
    pub fn apuntar(&self, que: Que) {
        if let Err(e) = self.apuntar_en(ahora(), que) {
            eprintln!("registro: no se puede apuntar: {e}");
        }
    }

    /// Apunta un fallo, si no se ha apuntado ya el mismo hoy: un fallo que
    /// se repite (la carpeta que no está, al mirar cada vez) cuenta una vez
    /// al día.
    pub fn fallo(&self, donde: &str, error: &str) {
        let hoy = ahora() / DIA;
        let repetido = self
            .leer()
            .unwrap_or_default()
            .iter()
            .rev()
            .take_while(|c| c.cuando / DIA == hoy)
            .any(|c| matches!(&c.que, Que::Fallo { donde: d, error: e } if d == donde && e == error));
        if !repetido {
            self.apuntar(Que::Fallo { donde: donde.into(), error: error.into() });
        }
    }

    fn apuntar_en(&self, cuando: u64, que: Que) -> Result<(), String> {
        let _cogido = self.cerrojo.lock().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&self.carpeta).map_err(|e| e.to_string())?;
        let caso = Caso { cuando, version: VERSION.into(), que };
        let linea = serde_json::to_string(&caso).map_err(|e| e.to_string())?;
        let ruta = self.carpeta.join(nombre_del_mes(cuando));
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&ruta).map_err(|e| e.to_string())?;
        writeln!(f, "{linea}").map_err(|e| e.to_string())
    }

    /// Todos los casos, del más antiguo al más nuevo. Una línea que no se
    /// entiende (cortada por un apagón) se salta.
    pub fn leer(&self) -> Result<Vec<Caso>, String> {
        let mut casos = Vec::new();
        for ruta in self.ficheros()? {
            let texto = std::fs::read_to_string(&ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
            casos.extend(texto.lines().filter_map(|l| serde_json::from_str::<Caso>(l).ok()));
        }
        casos.sort_by_key(|c| c.cuando);
        Ok(casos)
    }

    pub fn resumen(&self) -> Result<Resumen, String> {
        Ok(resumir(&self.leer()?))
    }

    /// Borra todo el registro.
    pub fn borrar(&self) -> Result<(), String> {
        let _cogido = self.cerrojo.lock().map_err(|e| e.to_string())?;
        for ruta in self.ficheros()? {
            std::fs::remove_file(&ruta).map_err(|e| format!("no se puede borrar {}: {e}", ruta.display()))?;
        }
        Ok(())
    }

    /// Borra los meses de hace más de [`MESES`].
    pub fn purgar(&self) {
        let (a, m, _) = fecha(ahora());
        let limite = (a * 12 + m - 1).saturating_sub(MESES);
        let _cogido = self.cerrojo.lock();
        for ruta in self.ficheros().unwrap_or_default() {
            let viejo = mes_del_nombre(&ruta).is_some_and(|(a, m)| a * 12 + m - 1 < limite);
            if viejo {
                let _ = std::fs::remove_file(&ruta);
            }
        }
    }

    fn ficheros(&self) -> Result<Vec<PathBuf>, String> {
        let entradas = match std::fs::read_dir(&self.carpeta) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("no se puede leer {}: {e}", self.carpeta.display())),
        };
        let mut rutas: Vec<PathBuf> =
            entradas.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| mes_del_nombre(p).is_some()).collect();
        rutas.sort();
        Ok(rutas)
    }
}

const DIA: u64 = 86_400;

pub fn ahora() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn nombre_del_mes(cuando: u64) -> String {
    let (a, m, _) = fecha(cuando);
    format!("casos-{a:04}-{m:02}.jsonl")
}

fn mes_del_nombre(ruta: &Path) -> Option<(u32, u32)> {
    let nombre = ruta.file_name()?.to_str()?;
    let am = nombre.strip_prefix("casos-")?.strip_suffix(".jsonl")?;
    let (a, m) = am.split_once('-')?;
    Some((a.parse().ok()?, m.parse().ok()?))
}

/// Año, mes y día (UTC) de unos segundos desde 1970. El algoritmo
/// `civil_from_days` de Howard Hinnant: no hace falta una biblioteca de
/// fechas para esto.
pub fn fecha(cuando: u64) -> (u32, u32, u32) {
    let z = (cuando / DIA) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let a = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (a as u32, m as u32, d as u32)
}

/// `2026-11-19`.
pub fn dia(cuando: u64) -> String {
    let (a, m, d) = fecha(cuando);
    format!("{a:04}-{m:02}-{d:02}")
}

/// Lo que se hizo con cada dictado, por su `id`: la última respuesta.
pub fn respuestas(casos: &[Caso]) -> std::collections::HashMap<&str, &Respuesta> {
    casos
        .iter()
        .filter_map(|c| match &c.que {
            Que::Respuesta { id, respuesta } => Some((id.as_str(), respuesta)),
            _ => None,
        })
        .collect()
}

fn resumir(casos: &[Caso]) -> Resumen {
    let respuestas = respuestas(casos);
    let mut r = Resumen { desde: casos.first().map(|c| c.cuando), ..Resumen::default() };
    for c in casos {
        match &c.que {
            Que::Dictado { dictado, .. } => {
                r.dictados += 1;
                match dictado.entendido {
                    Entendido::Seguro { .. } => {
                        r.seguros += 1;
                        if matches!(
                            respuestas.get(dictado.id.as_str()),
                            Some(Respuesta::Deshecha | Respuesta::Corregida { .. })
                        ) {
                            r.corregidos += 1;
                        }
                    }
                    Entendido::Duda { .. } => r.preguntados += 1,
                    Entendido::NoEntendido { .. } => r.no_entendidos += 1,
                }
            }
            Que::Fallo { .. } => r.fallos += 1,
            Que::Respuesta { .. } => {}
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voz::entender::Motivo;

    pub(crate) fn dictado(id: &str, entendido: Entendido<String>) -> Que {
        Que::Dictado {
            dictado: Dictado { id: id.into(), oido: "Lucía Martín nueve y medio".into(), entendido },
            fichero: "mates.xlsx".into(),
            nombre: vec!["lucia".into(), "martin".into()],
            candidatos: vec![
                Candidato { clave: "1001".into(), parecido: 1.0, distancia: 0 },
                Candidato { clave: "1002".into(), parecido: 0.83, distancia: 2 },
            ],
        }
    }

    #[test]
    fn fechas() {
        assert_eq!(fecha(0), (1970, 1, 1));
        assert_eq!(fecha(1_795_046_400), (2026, 11, 19));
        assert_eq!(fecha(951_782_400), (2000, 2, 29));
        assert_eq!(dia(1_795_046_400 + DIA - 1), "2026-11-19");
    }

    #[test]
    fn apuntar_leer_resumir_y_borrar() {
        let dir = tempfile::tempdir().unwrap();
        let r = Registro::new(dir.path().join("registro"));
        assert_eq!(r.resumen().unwrap(), Resumen::default());

        let seguro = Entendido::Seguro { alumno: "1001".to_string(), nota: 9.5 };
        let duda = Entendido::Duda {
            alumnos: vec!["1001".to_string(), "1002".to_string()],
            notas: vec![9.5],
            motivos: vec![Motivo::NombreParecido],
        };
        r.apuntar(dictado("a", seguro.clone()));
        r.apuntar(Que::Respuesta { id: "a".into(), respuesta: Respuesta::Deshecha });
        r.apuntar(dictado("b", duda));
        r.apuntar(Que::Respuesta { id: "b".into(), respuesta: Respuesta::Elegida { clave: "1002".into(), nota: 9.5 } });
        r.apuntar(dictado("c", seguro));
        r.apuntar(dictado("d", Entendido::NoEntendido { motivo: Motivo::SinNota }));
        r.fallo("leer_clase", "no se puede leer");
        r.fallo("leer_clase", "no se puede leer");
        // Una línea cortada no estropea las demás.
        let mes = r.ficheros().unwrap()[0].clone();
        std::fs::OpenOptions::new().append(true).open(&mes).unwrap().write_all(b"{\"cuando\": 1").unwrap();

        let casos = r.leer().unwrap();
        assert_eq!(casos.len(), 7);
        assert_eq!(casos[0].que, dictado("a", Entendido::Seguro { alumno: "1001".to_string(), nota: 9.5 }));
        let resumen = r.resumen().unwrap();
        assert_eq!((resumen.dictados, resumen.seguros, resumen.preguntados, resumen.no_entendidos), (4, 2, 1, 1));
        assert_eq!((resumen.corregidos, resumen.fallos), (1, 1));

        r.borrar().unwrap();
        assert!(r.leer().unwrap().is_empty());
    }

    #[test]
    fn lo_viejo_se_borra() {
        let dir = tempfile::tempdir().unwrap();
        let r = Registro::new(dir.path().to_path_buf());
        let (a, m, _) = fecha(ahora());
        let hace = |meses: u32| {
            let n = a * 12 + m - 1 - meses;
            dir.path().join(format!("casos-{:04}-{:02}.jsonl", n / 12, n % 12 + 1))
        };
        for meses in [0, MESES, MESES + 1, 30] {
            std::fs::write(hace(meses), "").unwrap();
        }
        std::fs::write(dir.path().join("otra.txt"), "").unwrap();
        r.purgar();
        assert!(hace(0).exists() && hace(MESES).exists());
        assert!(!hace(MESES + 1).exists() && !hace(30).exists());
        assert!(dir.path().join("otra.txt").exists());
    }

    #[test]
    fn el_dictado_va_plano_para_la_interfaz() {
        let d = Dictado {
            id: "x".into(),
            oido: "Elena 7".into(),
            entendido: Entendido::Seguro { alumno: "1001".into(), nota: 7.0 },
        };
        assert_eq!(
            serde_json::to_value(&d).unwrap(),
            serde_json::json!({"id": "x", "oido": "Elena 7", "tipo": "seguro", "alumno": "1001", "nota": 7.0})
        );
    }
}
