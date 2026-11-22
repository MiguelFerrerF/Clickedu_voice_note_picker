//! **Aplicar** (H5, la cuarta parte de traer): hacer lo planeado
//! ([`super::planear`]) en la carpeta.
//!
//! 1. Lo apartado, a `plantillas\antiguas\‹AAAA-MM-DD›\` (si ya hay uno con
//!    su nombre, ` (2)`). **Un Excel abierto no se mueve**: se dice, sus
//!    notas no pasan y la nueva de su mismo nombre no se escribe.
//! 2. Las nuevas, de `clickedu\bajadas\`.
//! 3. Las notas, en `sesion.json`, con el [`Cerrojo`]: lo que ya tuviera la
//!    nueva, manda.
//! 4. `traidas.json`, y se vacía `bajadas`.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::planear::{EstadoMateria, Plan, Recordadas};
use crate::ajustes::guardar_json;
use crate::sesion::{self, Cerrojo};

/// La carpeta de lo apartado, dentro de la de plantillas.
pub const ANTIGUAS: &str = "antiguas";

/// Dónde está cada cosa.
pub struct Rutas<'a> {
    /// La carpeta de plantillas.
    pub plantillas: &'a Path,
    /// `sesion.json`.
    pub sesion: &'a Path,
    /// `clickedu\traidas.json`.
    pub recordadas: &'a Path,
    /// `clickedu\bajadas\`: lo bajado, por fichero nuevo.
    pub bajadas: &'a Path,
}

/// Lo que se ha hecho, para contarlo.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Hecho {
    /// Las nuevas escritas en la raíz.
    pub escritas: Vec<String>,
    pub movidas: Vec<Movida>,
    /// Cuántas notas han pasado, en total.
    pub pasan: usize,
    pub fallos: Vec<FalloAplicar>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Movida {
    pub fichero: String,
    /// Dónde ha quedado, desde la carpeta de plantillas:
    /// `antiguas\2026-11-22\biologia_3_a_eso.xlsx`.
    pub a: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum FalloAplicar {
    /// No se ha podido apartar (lo normal: está abierto en Excel).
    Mover { fichero: String, error: String },
    /// La nueva no se ha escrito: la vieja de su nombre sigue ahí, o falla
    /// el disco.
    Escribir { fichero: String, error: Option<String> },
    /// Las notas de una apartada no han pasado: su nueva no está.
    Notas { fichero: String, a: String },
    /// `sesion.json` o `traidas.json` no se han podido guardar.
    Guardar { error: String },
}

/// Hace `plan` en la carpeta. `hoy`, `AAAA-MM-DD`.
pub fn aplicar(plan: &Plan, rutas: &Rutas, hoy: &str, cerrojo: &Cerrojo) -> Hecho {
    let mut hecho = Hecho::default();
    let antiguas = rutas.plantillas.join(ANTIGUAS).join(hoy);

    // 1. Apartar.
    let mut sin_mover: HashSet<String> = HashSet::new();
    for a in &plan.apartadas {
        match apartar(&rutas.plantillas.join(&a.fichero), &antiguas) {
            Ok(destino) => {
                let relativa = destino.strip_prefix(rutas.plantillas).unwrap_or(&destino);
                hecho.movidas.push(Movida { fichero: a.fichero.clone(), a: relativa.display().to_string() });
            }
            Err(error) => {
                sin_mover.insert(a.fichero.to_lowercase());
                hecho.fallos.push(FalloAplicar::Mover { fichero: a.fichero.clone(), error });
            }
        }
    }

    // 2. Escribir las nuevas. La carpeta puede no estar aún.
    if let Err(e) = std::fs::create_dir_all(rutas.plantillas) {
        hecho
            .fallos
            .push(FalloAplicar::Guardar { error: format!("no se puede crear {}: {e}", rutas.plantillas.display()) });
    }
    for m in plan.materias.iter().filter(|m| m.estado.se_escribe()) {
        if sin_mover.contains(&m.fichero.to_lowercase()) {
            hecho.fallos.push(FalloAplicar::Escribir { fichero: m.fichero.clone(), error: None });
            continue;
        }
        match std::fs::copy(rutas.bajadas.join(&m.fichero), rutas.plantillas.join(&m.fichero)) {
            Ok(_) => hecho.escritas.push(m.fichero.clone()),
            Err(e) => {
                hecho.fallos.push(FalloAplicar::Escribir { fichero: m.fichero.clone(), error: Some(e.to_string()) })
            }
        }
    }

    // 3. Las notas. Pueden ir a una nueva escrita o a una igual.
    let estan: HashSet<String> = plan
        .materias
        .iter()
        .filter(|m| matches!(m.estado, EstadoMateria::Igual { .. }) || hecho.escritas.iter().any(|e| e == &m.fichero))
        .map(|m| m.fichero.clone())
        .collect();
    let movidas: HashSet<&str> = hecho.movidas.iter().map(|m| m.fichero.as_str()).collect();
    let mut pasan: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    let mut se_olvidan: Vec<&str> = Vec::new();
    for a in plan.apartadas.iter().filter(|a| movidas.contains(a.fichero.as_str())) {
        match &a.notas {
            Some(paso) if !estan.contains(&paso.a) => {
                // Se quedan en la sesión con su nombre viejo: no se pierden.
                hecho.fallos.push(FalloAplicar::Notas { fichero: a.fichero.clone(), a: paso.a.clone() });
            }
            Some(paso) => {
                let de_la_nueva = pasan.entry(paso.a.clone()).or_default();
                for (clave, v) in &paso.notas {
                    de_la_nueva.entry(clave.clone()).or_insert(*v);
                }
                se_olvidan.push(&a.fichero);
            }
            None => se_olvidan.push(&a.fichero),
        }
    }
    if !pasan.is_empty() || !se_olvidan.is_empty() {
        let hecho_notas = (|| {
            let _cerrojo = cerrojo.0.lock().map_err(|e| e.to_string())?;
            let mut s = sesion::leer(rutas.sesion)?;
            // Primero se olvidan las viejas: una nueva puede llamarse igual
            // que la vieja de la que vienen sus notas.
            for f in &se_olvidan {
                s.clases.remove(*f);
            }
            let mut cuantas = 0;
            for (fichero, notas) in pasan {
                let puestas = s.clases.entry(fichero).or_default();
                for (clave, v) in notas {
                    // Lo que ya tuviera la nueva, manda.
                    if !puestas.contains_key(&clave) {
                        puestas.insert(clave, Some(v));
                        cuantas += 1;
                    }
                }
            }
            s.clases.retain(|_, p| !p.is_empty());
            guardar_json(rutas.sesion, &s)?;
            Ok::<usize, String>(cuantas)
        })();
        match hecho_notas {
            Ok(n) => hecho.pasan = n,
            Err(error) => hecho.fallos.push(FalloAplicar::Guardar { error }),
        }
    }

    // 4. Lo recordado: las traídas que están en la raíz; de las demás, lo
    // de antes.
    let mut recordadas = leer_recordadas(rutas.recordadas).unwrap_or_default();
    recordadas.confirmada = true;
    for m in &plan.materias {
        match &m.estado {
            _ if estan.contains(&m.fichero) => {
                recordadas.materias.insert(m.materia.id.clone(), m.fichero.clone());
            }
            EstadoMateria::Vacia => {
                recordadas.materias.remove(&m.materia.id);
            }
            _ => {}
        }
    }
    if let Err(error) = guardar_json(rutas.recordadas, &recordadas) {
        hecho.fallos.push(FalloAplicar::Guardar { error });
    }
    let _ = std::fs::remove_dir_all(rutas.bajadas);
    hecho
}

/// Lee `traidas.json`; si no está, vacío.
pub fn leer_recordadas(ruta: &Path) -> Result<Recordadas, String> {
    match std::fs::read_to_string(ruta) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{} está dañado: {e}", ruta.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Recordadas::default()),
        Err(e) => Err(format!("no se puede leer {}: {e}", ruta.display())),
    }
}

/// Mueve `ruta` a `antiguas`, con ` (2)`, ` (3)`… si ya hay uno con su
/// nombre. Devuelve dónde ha quedado.
fn apartar(ruta: &Path, antiguas: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(antiguas).map_err(|e| e.to_string())?;
    let tallo = ruta.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let extension = ruta.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default();
    let mut destino = antiguas.join(format!("{tallo}{extension}"));
    let mut n = 2;
    while destino.exists() {
        destino = antiguas.join(format!("{tallo} ({n}){extension}"));
        n += 1;
    }
    std::fs::rename(ruta, &destino).map_err(|e| e.to_string())?;
    Ok(destino)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickedu::fichas::Materia;
    use crate::clickedu::traer::planear::{Apartada, Paso, PlanMateria, Por};
    use crate::sesion::Sesion;

    fn materia(id: &str, fichero: &str, estado: EstadoMateria) -> PlanMateria {
        PlanMateria {
            materia: Materia { id: id.into(), nombre: "Materia".into(), grupo: String::new(), curso: None },
            fichero: fichero.into(),
            pendiente: false,
            retoques: vec![],
            avisos: vec![],
            estado,
            estado_evaluaciones: None,
        }
    }

    fn paso(a: &str, notas: &[(&str, f64)]) -> Option<Paso> {
        Some(Paso {
            a: a.into(),
            por: Por::Alumnos,
            notas: notas.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            pasan: notas.len(),
            sin_alumno: 0,
            sin_id: 0,
        })
    }

    struct Carpeta {
        _dir: tempfile::TempDir,
        plantillas: PathBuf,
        sesion: PathBuf,
        recordadas: PathBuf,
        bajadas: PathBuf,
    }

    impl Carpeta {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let c = Carpeta {
                plantillas: dir.path().join("plantillas"),
                sesion: dir.path().join("app").join("sesion.json"),
                recordadas: dir.path().join("app").join("clickedu").join("traidas.json"),
                bajadas: dir.path().join("app").join("clickedu").join("bajadas"),
                _dir: dir,
            };
            std::fs::create_dir_all(&c.plantillas).unwrap();
            std::fs::create_dir_all(&c.bajadas).unwrap();
            c
        }
        fn rutas(&self) -> Rutas<'_> {
            Rutas {
                plantillas: &self.plantillas,
                sesion: &self.sesion,
                recordadas: &self.recordadas,
                bajadas: &self.bajadas,
            }
        }
        fn en_raiz(&self, f: &str, contenido: &str) {
            std::fs::write(self.plantillas.join(f), contenido).unwrap();
        }
        fn bajada(&self, f: &str, contenido: &str) {
            std::fs::write(self.bajadas.join(f), contenido).unwrap();
        }
        fn leer(&self, relativa: &str) -> String {
            std::fs::read_to_string(self.plantillas.join(relativa)).unwrap()
        }
    }

    const HOY: &str = "2026-11-22";

    #[test]
    fn aparta_escribe_y_pasa_las_notas() {
        let c = Carpeta::new();
        c.en_raiz("bio_v1.xlsx", "vieja bio");
        c.en_raiz("Bio - 3º ESO A.xlsx", "la de antes, distinta");
        c.en_raiz("Igual.xlsx", "igual");
        // Ya había una apartada hoy con ese nombre.
        std::fs::create_dir_all(c.plantillas.join(ANTIGUAS).join(HOY)).unwrap();
        std::fs::write(c.plantillas.join(ANTIGUAS).join(HOY).join("bio_v1.xlsx"), "de esta mañana").unwrap();
        c.bajada("Bio - 3º ESO A.xlsx", "nueva bio");
        c.bajada("Nueva.xlsx", "nueva");

        let mut s = Sesion::default();
        s.clases.entry("bio_v1.xlsx".into()).or_default().insert("11".into(), Some(7.0));
        s.clases.entry("Bio - 3º ESO A.xlsx".into()).or_default().insert("12".into(), Some(3.0));
        // La nueva ya tenía una para el 13: manda.
        s.clases.entry("Nueva.xlsx".into()).or_default().insert("13".into(), Some(10.0));
        guardar_json(&c.sesion, &s).unwrap();

        let plan = Plan {
            materias: vec![
                materia("1", "Bio - 3º ESO A.xlsx", EstadoMateria::Cambia { alumnos: 3 }),
                materia("2", "Igual.xlsx", EstadoMateria::Igual { alumnos: 3 }),
                materia("3", "Nueva.xlsx", EstadoMateria::Nueva { alumnos: 3 }),
                materia("4", "Vacia.xlsx", EstadoMateria::Vacia),
            ],
            apartadas: vec![
                Apartada { fichero: "bio_v1.xlsx".into(), notas: paso("Bio - 3º ESO A.xlsx", &[("11", 7.0)]) },
                Apartada {
                    fichero: "Bio - 3º ESO A.xlsx".into(),
                    notas: paso("Bio - 3º ESO A.xlsx", &[("11", 1.0), ("12", 3.0)]),
                },
            ],
            dudas: vec![],
        };
        // Otra apartada, con notas para la nueva.
        let mut plan = plan;
        c.en_raiz("otra.xlsx", "otra");
        plan.apartadas
            .push(Apartada { fichero: "otra.xlsx".into(), notas: paso("Nueva.xlsx", &[("13", 2.0), ("14", 6.0)]) });

        let hecho = aplicar(&plan, &c.rutas(), HOY, &Cerrojo::default());
        assert_eq!(hecho.fallos, []);
        assert_eq!(hecho.escritas, ["Bio - 3º ESO A.xlsx", "Nueva.xlsx"]);
        let a = |f: &str| Path::new(ANTIGUAS).join(HOY).join(f).display().to_string();
        assert_eq!(
            hecho.movidas,
            [
                Movida { fichero: "bio_v1.xlsx".into(), a: a("bio_v1 (2).xlsx") },
                Movida { fichero: "Bio - 3º ESO A.xlsx".into(), a: a("Bio - 3º ESO A.xlsx") },
                Movida { fichero: "otra.xlsx".into(), a: a("otra.xlsx") },
            ]
        );
        assert_eq!(c.leer("Bio - 3º ESO A.xlsx"), "nueva bio");
        assert_eq!(c.leer(&a("bio_v1 (2).xlsx")), "vieja bio");
        assert_eq!(c.leer(&a("bio_v1.xlsx")), "de esta mañana");
        assert_eq!(c.leer("Igual.xlsx"), "igual");
        assert!(!c.plantillas.join("otra.xlsx").exists());

        let s = sesion::leer(&c.sesion).unwrap();
        // El 11 viene de la primera apartada (recordada antes); el 12, de la
        // misma de antes.
        assert_eq!(
            s.puestas("Bio - 3º ESO A.xlsx"),
            BTreeMap::from([("11".into(), Some(7.0)), ("12".into(), Some(3.0))])
        );
        assert_eq!(s.puestas("Nueva.xlsx"), BTreeMap::from([("13".into(), Some(10.0)), ("14".into(), Some(6.0))]));
        assert!(!s.clases.contains_key("bio_v1.xlsx") && !s.clases.contains_key("otra.xlsx"));
        assert_eq!(hecho.pasan, 3);

        let r = leer_recordadas(&c.recordadas).unwrap();
        assert!(r.confirmada);
        assert_eq!(
            r.materias,
            BTreeMap::from([
                ("1".into(), "Bio - 3º ESO A.xlsx".into()),
                ("2".into(), "Igual.xlsx".into()),
                ("3".into(), "Nueva.xlsx".into()),
            ])
        );
        assert!(!c.bajadas.exists());
    }

    /// Un Excel abierto (sin compartir el borrado, como lo abre Excel) no se
    /// mueve: se dice, sus notas siguen donde estaban y la nueva de su
    /// nombre no se escribe.
    #[cfg(windows)]
    #[test]
    fn un_excel_abierto_no_se_mueve() {
        use std::os::windows::fs::OpenOptionsExt;
        let c = Carpeta::new();
        c.en_raiz("Bio.xlsx", "abierta");
        c.bajada("Bio.xlsx", "nueva");
        let mut s = Sesion::default();
        s.clases.entry("Bio.xlsx".into()).or_default().insert("11".into(), Some(7.0));
        guardar_json(&c.sesion, &s).unwrap();
        let plan = Plan {
            materias: vec![materia("1", "Bio.xlsx", EstadoMateria::Cambia { alumnos: 1 })],
            apartadas: vec![Apartada { fichero: "Bio.xlsx".into(), notas: paso("Bio.xlsx", &[("11", 7.0)]) }],
            dudas: vec![],
        };
        let abierta = std::fs::OpenOptions::new().read(true).share_mode(1).open(c.plantillas.join("Bio.xlsx")).unwrap();
        let hecho = aplicar(&plan, &c.rutas(), HOY, &Cerrojo::default());
        drop(abierta);
        assert!(
            matches!(&hecho.fallos[0], FalloAplicar::Mover { fichero, .. } if fichero == "Bio.xlsx"),
            "{:?}",
            hecho.fallos
        );
        assert_eq!(hecho.fallos[1], FalloAplicar::Escribir { fichero: "Bio.xlsx".into(), error: None });
        assert_eq!(c.leer("Bio.xlsx"), "abierta");
        assert_eq!(sesion::leer(&c.sesion).unwrap(), s);
        assert!(leer_recordadas(&c.recordadas).unwrap().materias.is_empty());
    }
}
