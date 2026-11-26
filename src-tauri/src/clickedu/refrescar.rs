//! **Refrescar lo de Clickedu solo** (H11, `docs/01`): con la cuenta
//! guardada, que lo leído de Clickedu no se quede viejo sin darse cuenta.
//!
//! - Los **ítems y sus notas** (`items.json`) se guardan: son una copia de
//!   Clickedu y no tocan nada del profesor, como al traer.
//! - Las **plantillas, solo se cuentan**: se bajan y se comparan con la
//!   carpeta ([`traer::estudiar`], sin aplicar), y lo que haya cambiado
//!   sale como «por traer» ([`por_traer`]). No se toca la carpeta, ni
//!   `sesion.json`, ni `traidas.json`, ni lo de un traer que espera
//!   respuestas (lo bajado va a `refresco`, no a `bajadas`).
//! - **No estorba**: si la sesión está cogida (un traer o una subida), se
//!   salta esta vez; y quien la espera lo para entre materia y materia
//!   (`parar`).
//!
//! Cuándo se lanza y qué se enseña es de `commands::refresco`.

use std::collections::HashSet;
use std::path::PathBuf;

use serde::Serialize;

use super::cliente::Cliente;
use super::items;
use super::traer::planear::{EstadoMateria, Plan, Recordadas, Traida};
use super::traer::{self, Donde, FalloTraer};

/// Lo bajado al refrescar, leído, en `clickedu\refresco`. Se borra al acabar.
pub const CARPETA: &str = "refresco";

/// Una plantilla que ha cambiado en Clickedu y no se trae sola.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PorTraer {
    /// El fichero en la carpeta: el que tendría (nueva) o el que tiene.
    pub fichero: String,
    pub que: QueCambia,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueCambia {
    /// Una materia que no está en la carpeta.
    Nueva,
    /// Alumnos que entran o salen, o cambian de nombre.
    Cambia,
    /// Se trajo y ya no está en Clickedu.
    YaNoEsta,
}

/// Lo que ha cambiado en Clickedu respecto a la carpeta, de un plan sin
/// aplicar. Antes de la primera vez que se trae, nada: no hay con qué
/// comparar, y Conexión ya invita a traer.
pub fn por_traer(plan: &Plan, recordadas: &Recordadas, traidas: &[Traida]) -> Vec<PorTraer> {
    if !recordadas.confirmada {
        return Vec::new();
    }
    let mut res: Vec<PorTraer> = plan
        .materias
        .iter()
        .filter_map(|m| {
            let que = match m.estado {
                EstadoMateria::Nueva { .. } => QueCambia::Nueva,
                EstadoMateria::Cambia { .. } => QueCambia::Cambia,
                _ => return None,
            };
            Some(PorTraer { fichero: m.fichero.clone(), que })
        })
        .collect();
    let en_clickedu: HashSet<&str> = traidas.iter().map(|t| t.materia.id.as_str()).collect();
    res.extend(
        recordadas
            .materias
            .iter()
            .filter(|(id, _)| !en_clickedu.contains(id.as_str()))
            .map(|(_, fichero)| PorTraer { fichero: fichero.clone(), que: QueCambia::YaNoEsta }),
    );
    res
}

/// Dónde está cada cosa.
pub struct Rutas {
    /// `%APPDATA%\es.gestornotas.app\clickedu`.
    pub clickedu: PathBuf,
    pub plantillas: PathBuf,
    pub sesion: PathBuf,
}

/// Cómo ha ido un refresco.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Refrescado {
    pub cuando: u64,
    pub por_traer: Vec<PorTraer>,
    /// El aviso temprano: la forma de las páginas que ha cambiado y los
    /// ítems que no se entienden, sin nombres.
    pub avisos: Vec<String>,
}

/// Refresca: con la sesión de `sesion` (si está cogida, `Ok(None)`: se
/// salta) o entrando con `entrar()`, baja todo, guarda los ítems y cuenta
/// lo que hay por traer. `parar()` lo para entre materia y materia.
pub async fn refrescar<F, Fut>(
    sesion: &tauri::async_runtime::Mutex<Option<Cliente>>,
    entrar: F,
    rutas: Rutas,
    parar: impl Fn() -> bool,
    ahora: u64,
) -> Result<Option<Refrescado>, FalloTraer>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Cliente, FalloTraer>>,
{
    let Ok(mut cogida) = sesion.try_lock() else { return Ok(None) };
    let ruta_items = rutas.clickedu.join(items::FICHERO);
    let (bajado, rotos) =
        traer::bajar_y_guardar_items(&mut cogida, entrar, &ruta_items, |_, _| {}, parar, ahora).await?;
    drop(cogida);

    let hecho = tauri::async_runtime::spawn_blocking(move || {
        let bajadas = rutas.clickedu.join(CARPETA);
        let donde =
            Donde { clickedu: &rutas.clickedu, bajadas: &bajadas, plantillas: &rutas.plantillas, sesion: &rutas.sesion };
        let estudiado = traer::estudiar(bajado, rotos, &donde);
        let _ = std::fs::remove_dir_all(&bajadas);
        let e = estudiado?;
        Ok::<_, String>(Refrescado {
            cuando: ahora,
            por_traer: por_traer(&e.plan, &e.recordadas, &e.traidas),
            avisos: e.cambios,
        })
    })
    .await
    .map_err(|e| format!("el trabajo en segundo plano se ha caído: {e}"))??;
    Ok(Some(hecho))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickedu::cliente::Fallo;
    use crate::clickedu::falso::{self, Modo, ARCHIVO_BUENO, CONTRASENA, USUARIO};
    use crate::clickedu::traer::aplicar;
    use tauri::async_runtime::{block_on, Mutex};

    /// Una carpeta de la app con su `clickedu`, sus plantillas y su sesión.
    struct Montado {
        _dir: tempfile::TempDir,
        clickedu: PathBuf,
        plantillas: PathBuf,
        sesion: PathBuf,
    }

    impl Montado {
        fn nuevo() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let (clickedu, plantillas) = (dir.path().join("clickedu"), dir.path().join("plantillas"));
            std::fs::create_dir_all(&clickedu).unwrap();
            std::fs::create_dir_all(&plantillas).unwrap();
            let sesion = dir.path().join("sesion.json");
            Montado { clickedu, plantillas, sesion, _dir: dir }
        }

        fn rutas(&self) -> Rutas {
            Rutas { clickedu: self.clickedu.clone(), plantillas: self.plantillas.clone(), sesion: self.sesion.clone() }
        }

        /// Lo que hay en la carpeta de plantillas y en la de la app, con
        /// su contenido: para ver que no se toca.
        fn foto(&self) -> Vec<(String, Vec<u8>)> {
            let mut res = Vec::new();
            for d in [&self.plantillas, &self.clickedu] {
                for e in std::fs::read_dir(d).unwrap().flatten() {
                    if e.file_type().unwrap().is_file() && e.file_name() != items::FICHERO {
                        res.push((e.file_name().to_string_lossy().into_owned(), std::fs::read(e.path()).unwrap()));
                    }
                }
            }
            if let Ok(s) = std::fs::read(&self.sesion) {
                res.push(("sesion.json".into(), s));
            }
            res.sort();
            res
        }
    }

    async fn entrar_en(base: String) -> Result<Cliente, FalloTraer> {
        let cliente = Cliente::new(&base)?;
        cliente.entrar(USUARIO, CONTRASENA, ("a.txt", ARCHIVO_BUENO)).await?;
        Ok(cliente)
    }

    /// Como si ya se hubiera traído: `traidas.json` confirmado, con
    /// `materias` (id, fichero).
    fn ya_traido(m: &Montado, materias: &[(&str, &str)]) {
        let recordadas = Recordadas {
            confirmada: true,
            materias: materias.iter().map(|(id, f)| (id.to_string(), f.to_string())).collect(),
        };
        crate::ajustes::guardar_json(&m.clickedu.join("traidas.json"), &recordadas).unwrap();
        assert_eq!(aplicar::leer_recordadas(&m.clickedu.join("traidas.json")).unwrap(), recordadas);
    }

    #[test]
    fn refresca_los_items_sin_tocar_plantillas_sesion_ni_traidas() {
        let f = falso::arrancar(Modo::Normal);
        let m = Montado::nuevo();
        // La plantilla de Matemáticas, la de Clickedu tal cual: igual.
        let mates = "Matemáticas - 3º ESO A.xlsx";
        std::fs::write(m.plantillas.join(mates), falso::plantilla_101()).unwrap();
        std::fs::write(&m.sesion, r#"{"notas":{}}"#).unwrap();
        ya_traido(&m, &[("101", mates)]);
        let antes = m.foto();

        let sesion = Mutex::new(None);
        let r = block_on(refrescar(&sesion, || entrar_en(f.base.clone()), m.rutas(), || false, 1_800_000_000))
            .unwrap()
            .unwrap();
        assert_eq!(r.cuando, 1_800_000_000);
        // Los ítems, guardados, con las notas de Clickedu.
        let g = items::leer(&m.clickedu.join(items::FICHERO)).unwrap();
        let i = &g.materias["101"].evaluaciones[0].items[0];
        assert_eq!(i.notas.get("5001").map(String::as_str), Some("7.5"));
        // Lo demás, como estaba; y lo bajado, borrado.
        assert_eq!(m.foto(), antes);
        assert!(!m.clickedu.join(CARPETA).exists());
        assert!(!m.clickedu.join("bajadas").exists());
        // Matemáticas es igual: nada por traer.
        assert!(r.por_traer.iter().all(|p| p.fichero != mates), "{:?}", r.por_traer);
        // La sesión se queda para la próxima vez.
        assert!(block_on(sesion.lock()).is_some());
    }

    #[test]
    fn ve_una_plantilla_cambiada_y_la_cuenta() {
        let f = falso::arrancar(Modo::Normal);
        let m = Montado::nuevo();
        let mates = "Matemáticas - 3º ESO A.xlsx";
        // Otra plantilla con el nombre de Matemáticas: ha cambiado.
        let mut libro = rust_xlsxwriter::Workbook::new();
        let hoja = libro.add_worksheet();
        for (c, t) in ["Id", "Alumnos", "Nota"].iter().enumerate() {
            hoja.write_string(0, c as u16, *t).unwrap();
        }
        hoja.write_number(1, 0, 9999.0).unwrap();
        hoja.write_string(1, 1, "Inventado, Alumno").unwrap();
        libro.save(m.plantillas.join(mates)).unwrap();
        ya_traido(&m, &[("101", mates), ("777", "Latín - 4º ESO A.xlsx")]);
        let antes = m.foto();

        let sesion = Mutex::new(None);
        let r = block_on(refrescar(&sesion, || entrar_en(f.base.clone()), m.rutas(), || false, 1))
            .unwrap()
            .unwrap();
        assert!(r.por_traer.contains(&PorTraer { fichero: mates.into(), que: QueCambia::Cambia }), "{:?}", r.por_traer);
        // La 777 se trajo y ya no está en Clickedu.
        assert!(r.por_traer.contains(&PorTraer { fichero: "Latín - 4º ESO A.xlsx".into(), que: QueCambia::YaNoEsta }));
        // Y no se ha cambiado nada.
        assert_eq!(m.foto(), antes);
    }

    #[test]
    fn antes_de_traer_la_primera_vez_no_cuenta_nada() {
        let f = falso::arrancar(Modo::Normal);
        let m = Montado::nuevo();
        let sesion = Mutex::new(None);
        let r = block_on(refrescar(&sesion, || entrar_en(f.base.clone()), m.rutas(), || false, 1))
            .unwrap()
            .unwrap();
        assert!(r.por_traer.is_empty());
        // Los ítems, sí.
        assert!(!items::leer(&m.clickedu.join(items::FICHERO)).unwrap().materias.is_empty());
    }

    #[test]
    fn se_salta_con_la_sesion_cogida() {
        let m = Montado::nuevo();
        let sesion = Mutex::new(None);
        let _cogida = block_on(sesion.lock());
        let r = block_on(refrescar(
            &sesion,
            || async { panic!("no debería entrar") },
            m.rutas(),
            || false,
            1,
        ));
        assert_eq!(r, Ok(None));
        assert!(!m.clickedu.join(items::FICHERO).exists());
    }

    #[test]
    fn se_para_si_alguien_espera_la_sesion() {
        let f = falso::arrancar(Modo::Normal);
        let m = Montado::nuevo();
        let sesion = Mutex::new(None);
        let r = block_on(refrescar(&sesion, || entrar_en(f.base.clone()), m.rutas(), || true, 1));
        assert_eq!(r, Err(FalloTraer::Parado));
        // Sin escribir nada.
        assert!(!m.clickedu.join(items::FICHERO).exists());
    }

    #[test]
    fn la_cuenta_rechazada_llega_como_tal() {
        let f = falso::arrancar(Modo::Normal);
        let m = Montado::nuevo();
        let sesion = Mutex::new(None);
        let entrar = || async {
            let cliente = Cliente::new(&f.base)?;
            cliente.entrar(USUARIO, "mala", ("a.txt", ARCHIVO_BUENO)).await?;
            Ok(cliente)
        };
        let r = block_on(refrescar(&sesion, entrar, m.rutas(), || false, 1));
        assert_eq!(r, Err(FalloTraer::Conexion { fallo: Fallo::Usuario }));
        assert!(block_on(sesion.lock()).is_none());
    }
}
