//! La sesión (H2): las notas que llevas puestas, clase por clase, y desde
//! H6 también ítem por ítem ([`Juego`]).
//!
//! **La plantilla no se escribe nunca** (D4): lo que se pone en la cuadrícula
//! vive aquí, en `sesion.json`, en la carpeta de la app
//! (`%APPDATA%\es.gestornotas.app\`), y se guarda **con cada nota**: cerrar
//! la app, o que se vaya la luz, no pierde nada. Al exportar se juntan la
//! plantilla y la sesión ([`nota_de`]).
//!
//! Cada nota se guarda con la **clave** del alumno ([`Alumno::clave`]: su ID,
//! o su fila si no lo tiene o está repetido), no con su posición: si
//! Clickedu trae la plantilla otra vez con la lista reordenada, cada nota
//! sigue con su alumno.
//!
//! Una clave con `None` es una nota **quitada a propósito**: la plantilla
//! traía una y se ha borrado en la cuadrícula. No es lo mismo que no haber
//! tocado al alumno.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::ajustes::guardar_json;
use crate::plantillas::{Alumno, Nota, Plantilla};

/// Las notas puestas de una clase, por clave de alumno.
pub type Puestas = BTreeMap<String, Option<f64>>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Sesion {
    /// Sin ítem, por fichero de plantilla: un solo juego de notas por clase,
    /// como la v1.
    #[serde(default)]
    pub clases: BTreeMap<String, Puestas>,
    /// Por ítem de Clickedu (su id, H6). El id de un ítem es único en
    /// Clickedu y es de una sola materia, y las claves son los `Id` de sus
    /// alumnos: no depende del nombre del fichero, así que traer otra vez
    /// las plantillas (que las aparta y las renombra) no las toca.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub items: BTreeMap<String, Puestas>,
}

/// En qué se califica una clase: sin ítem (como la v1) o en un ítem de
/// Clickedu (H6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Juego<'a> {
    /// El fichero de la plantilla.
    SinItem(&'a str),
    /// El id del ítem.
    Item(&'a str),
}

impl<'a> Juego<'a> {
    /// El de `fichero`, en el ítem `item` si lo hay.
    pub fn de(fichero: &'a str, item: Option<&'a str>) -> Self {
        match item {
            Some(i) => Juego::Item(i),
            None => Juego::SinItem(fichero),
        }
    }
}

impl Sesion {
    /// Las notas puestas de `fichero`, sin ítem; ninguna si no se ha tocado.
    pub fn puestas(&self, fichero: &str) -> Puestas {
        self.puestas_de(Juego::SinItem(fichero))
    }

    /// Las notas puestas de `juego`; ninguna si no se ha tocado.
    pub fn puestas_de(&self, juego: Juego) -> Puestas {
        let (mapa, clave) = match juego {
            Juego::SinItem(f) => (&self.clases, f),
            Juego::Item(i) => (&self.items, i),
        };
        mapa.get(clave).cloned().unwrap_or_default()
    }

    fn mapa_mut<'j>(&mut self, juego: Juego<'j>) -> (&mut BTreeMap<String, Puestas>, &'j str) {
        match juego {
            Juego::SinItem(f) => (&mut self.clases, f),
            Juego::Item(i) => (&mut self.items, i),
        }
    }
}

/// Para que dos notas puestas a la vez no se pisen al leer y reescribir el
/// fichero: cada comando que escribe la sesión lo coge antes.
#[derive(Default)]
pub struct Cerrojo(pub Mutex<()>);

/// Lee la sesión. Si el fichero no existe, vacía; si existe y no se entiende,
/// error: no se pisan en silencio las notas que hubiera.
pub fn leer(ruta: &Path) -> Result<Sesion, String> {
    match std::fs::read_to_string(ruta) {
        Ok(texto) => serde_json::from_str(&texto)
            .map_err(|e| format!("las notas guardadas en {} están dañadas: {e}", ruta.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Sesion::default()),
        Err(e) => Err(format!("no se pueden leer las notas guardadas en {}: {e}", ruta.display())),
    }
}

/// Pone (o quita, con `None`) la nota del alumno `clave` en `juego` y
/// guarda la sesión de una vez. La nota ha de pasar [`validar`]. Quien
/// llama tiene el [`Cerrojo`].
pub fn poner(ruta: &Path, juego: Juego, clave: &str, nota: Option<f64>) -> Result<Option<f64>, String> {
    let nota = nota.map(validar).transpose().map_err(|e| e.to_string())?;
    let mut sesion = leer(ruta)?;
    let (mapa, de) = sesion.mapa_mut(juego);
    mapa.entry(de.to_string()).or_default().insert(clave.to_string(), nota);
    guardar_json(ruta, &sesion).map_err(|e| format!("no se puede guardar la nota en {}: {e}", ruta.display()))?;
    Ok(nota)
}

/// Devuelve al alumno `clave` de `juego` a la nota de la plantilla:
/// olvida lo puesto en la sesión. Es deshacer un dictado (H3b) sobre un
/// alumno que no se había tocado. Quien llama tiene el [`Cerrojo`].
pub fn devolver(ruta: &Path, juego: Juego, clave: &str) -> Result<(), String> {
    let mut sesion = leer(ruta)?;
    let (mapa, de) = sesion.mapa_mut(juego);
    let Some(puestas) = mapa.get_mut(de) else { return Ok(()) };
    if puestas.remove(clave).is_none() {
        return Ok(());
    }
    if puestas.is_empty() {
        mapa.remove(de);
    }
    guardar_json(ruta, &sesion).map_err(|e| format!("no se puede guardar en {}: {e}", ruta.display()))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorNota {
    FueraDeRango,
    MasDeDosDecimales,
}

impl fmt::Display for ErrorNota {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FueraDeRango => write!(f, "una nota va de 0 a 10"),
            Self::MasDeDosDecimales => write!(f, "una nota lleva dos decimales como mucho"),
        }
    }
}

/// Una nota de 0 a 10 con dos decimales como mucho, como la acepta Clickedu.
/// Devuelve el valor limpio de la cola del coma flotante (`6.8000000001` →
/// `6.8`). La interfaz valida lo mismo antes de mandarla: esto es la red.
pub fn validar(v: f64) -> Result<f64, ErrorNota> {
    if !v.is_finite() || !(0.0..=10.0).contains(&v) {
        return Err(ErrorNota::FueraDeRango);
    }
    let centesimas = v * 100.0;
    if (centesimas - centesimas.round()).abs() > 1e-6 {
        return Err(ErrorNota::MasDeDosDecimales);
    }
    Ok(centesimas.round() / 100.0)
}

/// La nota que vale de un alumno: la puesta en la sesión si la hay, y si no
/// la de la plantilla.
pub fn nota_de(alumno: &Alumno, puestas: &Puestas) -> Nota {
    match puestas.get(&alumno.clave) {
        Some(Some(v)) => Nota::Valor(*v),
        Some(None) => Nota::Vacia,
        None => alumno.nota.clone(),
    }
}

/// Cuántos alumnos tienen nota, contando la sesión.
pub fn calificados(plantilla: &Plantilla, puestas: &Puestas) -> usize {
    plantilla.alumnos.iter().filter(|a| matches!(nota_de(a, puestas), Nota::Valor(_))).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alumno(clave: &str, nota: Nota) -> Alumno {
        Alumno { fila: 2, id: clave.into(), nombre: "Castro Molina, Lucía".into(), nota, clave: clave.into() }
    }

    #[test]
    fn validar_notas() {
        assert_eq!(validar(7.5), Ok(7.5));
        assert_eq!(validar(0.0), Ok(0.0));
        assert_eq!(validar(10.0), Ok(10.0));
        assert_eq!(validar(6.8), Ok(6.8));
        assert_eq!(validar(6.8 + 1e-12), Ok(6.8));
        assert_eq!(validar(7.25), Ok(7.25));
        assert_eq!(validar(7.255), Err(ErrorNota::MasDeDosDecimales));
        assert_eq!(validar(10.01), Err(ErrorNota::FueraDeRango));
        assert_eq!(validar(-0.5), Err(ErrorNota::FueraDeRango));
        assert_eq!(validar(f64::NAN), Err(ErrorNota::FueraDeRango));
    }

    #[test]
    fn la_sesion_manda_sobre_la_plantilla() {
        let mut puestas = Puestas::new();
        let a = alumno("1001", Nota::Valor(4.0));
        assert_eq!(nota_de(&a, &puestas), Nota::Valor(4.0));
        puestas.insert("1001".into(), Some(6.5));
        assert_eq!(nota_de(&a, &puestas), Nota::Valor(6.5));
        // Quitada a propósito.
        puestas.insert("1001".into(), None);
        assert_eq!(nota_de(&a, &puestas), Nota::Vacia);
        // Lo que no era nota, sustituido.
        let b = alumno("1002", Nota::Invalida("NP".into()));
        puestas.insert("1002".into(), Some(5.0));
        assert_eq!(nota_de(&b, &puestas), Nota::Valor(5.0));
    }

    #[test]
    fn poner_guarda_al_momento_y_no_pisa_lo_danado() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("app").join("sesion.json");
        assert_eq!(leer(&ruta).unwrap(), Sesion::default());

        let (mates, bio) = (Juego::SinItem("mates.xlsx"), Juego::SinItem("bio.xlsx"));
        assert_eq!(poner(&ruta, mates, "1001", Some(7.5)), Ok(Some(7.5)));
        poner(&ruta, mates, "fila:9", None).unwrap();
        poner(&ruta, bio, "2001", Some(10.0)).unwrap();
        assert!(poner(&ruta, bio, "2001", Some(11.0)).is_err());

        let s = leer(&ruta).unwrap();
        assert_eq!(s.puestas("mates.xlsx").get("1001"), Some(&Some(7.5)));
        assert_eq!(s.puestas("mates.xlsx").get("fila:9"), Some(&None));
        assert_eq!(s.puestas("bio.xlsx").get("2001"), Some(&Some(10.0)));
        assert!(s.puestas("otra.xlsx").is_empty());
        assert!(!ruta.with_extension("json.tmp").exists());

        // Devolver a la de la plantilla no es quitarla.
        devolver(&ruta, mates, "1001").unwrap();
        devolver(&ruta, bio, "2001").unwrap();
        devolver(&ruta, Juego::SinItem("otra.xlsx"), "1").unwrap();
        let s = leer(&ruta).unwrap();
        assert_eq!(s.puestas("mates.xlsx").get("1001"), None);
        assert_eq!(s.puestas("mates.xlsx").get("fila:9"), Some(&None));
        assert!(!s.clases.contains_key("bio.xlsx"));

        std::fs::write(&ruta, "{ medio json").unwrap();
        assert!(poner(&ruta, mates, "1001", Some(3.0)).is_err());
        assert_eq!(std::fs::read_to_string(&ruta).unwrap(), "{ medio json");
    }

    #[test]
    fn cada_item_es_otro_juego_de_notas() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("sesion.json");
        // Una sesión de antes de H6, sin `items`, se lee igual.
        std::fs::write(&ruta, r#"{"clases":{"bio.xlsx":{"2001":7.0}}}"#).unwrap();
        assert_eq!(leer(&ruta).unwrap().puestas("bio.xlsx").get("2001"), Some(&Some(7.0)));

        poner(&ruta, Juego::de("bio.xlsx", Some("42700")), "2001", Some(4.5)).unwrap();
        poner(&ruta, Juego::de("bio.xlsx", Some("42701")), "2001", Some(9.0)).unwrap();
        let s = leer(&ruta).unwrap();
        assert_eq!(s.puestas("bio.xlsx").get("2001"), Some(&Some(7.0)));
        assert_eq!(s.puestas_de(Juego::Item("42700")).get("2001"), Some(&Some(4.5)));
        assert_eq!(s.puestas_de(Juego::Item("42701")).get("2001"), Some(&Some(9.0)));

        devolver(&ruta, Juego::Item("42700"), "2001").unwrap();
        let s = leer(&ruta).unwrap();
        assert!(!s.items.contains_key("42700"));
        assert_eq!(s.puestas_de(Juego::Item("42701")).len(), 1);
        assert_eq!(s.puestas("bio.xlsx").len(), 1);
    }
}
