//! El paso desde la v1.3.4 (H7, D7): traer sus plantillas y sus notas y
//! desinstalarla.
//!
//! La v1 (Python, Inno Setup, por usuario) vive en
//! `%LOCALAPPDATA%\GestorNotasClickedu\` y guarda **dentro** de esa carpeta
//! `plantillas_clickedu_aulas\` y `notas\` (con `notas_viejas\`). Se
//! encuentra por el registro, como la encuentra el «Agregar o quitar
//! programas» de Windows: `…\Uninstall\Gestor de Notas Clickedu_is1`. Su
//! instalador la pone en la carpeta del usuario, pero **pide administrador**
//! y la apunta en `HKLM` (en el ordenador de desarrollo, en `WOW6432Node`):
//! quitarla pide permiso a Windows, así que se hace con un botón, no sola al
//! arrancar.
//!
//! Reglas:
//!
//! - **Se copia, nunca se mueve**, y cada fichero copiado se compara byte a
//!   byte con el suyo. Uno que ya está igual no se vuelve a copiar; uno que
//!   está **distinto** no se pisa: el de la v1 entra como «… (de la v1)».
//! - **Solo con todo copiado y comprobado se desinstala**, y solo si la v1 no
//!   está abierta. Si algo falla, la v1 se queda y se dice por qué; al abrir
//!   la app otra vez se vuelve a intentar (lo ya copiado cuenta como «ya
//!   estaba»).
//! - Desinstalarla no borra lo que el usuario puso en su carpeta (Inno solo
//!   quita lo que instaló), pero da igual: ya está copiado.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::plantillas::es_plantilla;

/// La clave de la v1 en el registro: bajo `HKEY_CURRENT_USER` o
/// `HKEY_LOCAL_MACHINE`, y en este, en la vista de 32 bits o en la de 64.
pub const CLAVE_V1: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Gestor de Notas Clickedu_is1";
#[cfg(windows)]
const CLAVE_V1_32: &str =
    r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Gestor de Notas Clickedu_is1";
/// Sus carpetas de datos, dentro de la de instalación.
pub const PLANTILLAS_V1: &str = "plantillas_clickedu_aulas";
pub const NOTAS_V1: &str = "notas";

/// La v1 instalada.
#[derive(Debug, Clone, PartialEq)]
pub struct V1 {
    pub carpeta: PathBuf,
    pub desinstalador: PathBuf,
    /// Instalada para todo el equipo (`HKLM`): quitarla pide permiso.
    pub admin: bool,
}

/// Lo que se ha traído.
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct Traido {
    /// Plantillas copiadas ahora.
    pub clases: usize,
    /// Ficheros de notas copiados ahora (con los de `notas_viejas`).
    pub notas: usize,
    /// Los que ya estaban, iguales: de un intento anterior.
    pub ya_estaban: usize,
    /// Los que había distintos con el mismo nombre: el de la v1 entra con
    /// otro, que es este.
    pub renombrados: Vec<String>,
}

/// La v1, si está instalada. En otro sistema que Windows, nunca.
pub fn buscar_v1() -> Option<V1> {
    #[cfg(windows)]
    {
        use windows_registry::{CURRENT_USER, LOCAL_MACHINE};
        let (clave, admin) = CURRENT_USER
            .open(CLAVE_V1)
            .map(|c| (c, false))
            .or_else(|_| LOCAL_MACHINE.open(CLAVE_V1_32).map(|c| (c, true)))
            .or_else(|_| LOCAL_MACHINE.open(CLAVE_V1).map(|c| (c, true)))
            .ok()?;
        let carpeta = PathBuf::from(clave.get_string("InstallLocation").ok()?);
        let desinstalador = PathBuf::from(clave.get_string("UninstallString").ok()?.trim().trim_matches('"'));
        Some(V1 { carpeta, desinstalador, admin })
    }
    #[cfg(not(windows))]
    None
}

/// Copia las plantillas de la v1 a `plantillas` y sus notas a `notas`.
pub fn copiar(v1: &Path, plantillas: &Path, notas: &Path) -> Result<Traido, String> {
    let mut t = Traido::default();

    let origen = v1.join(PLANTILLAS_V1);
    if origen.is_dir() {
        fs::create_dir_all(plantillas).map_err(|e| crear(plantillas, e))?;
        for f in ficheros(&origen)? {
            let nombre = f.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if es_plantilla(nombre) {
                traer(&f, plantillas, &mut t, |t| t.clases += 1)?;
            }
        }
    }

    let origen = v1.join(NOTAS_V1);
    if origen.is_dir() {
        copiar_notas(&origen, notas, &mut t)?;
    }
    Ok(t)
}

/// `notas\` entera, con sus subcarpetas (`notas_viejas\`).
fn copiar_notas(origen: &Path, destino: &Path, t: &mut Traido) -> Result<(), String> {
    fs::create_dir_all(destino).map_err(|e| crear(destino, e))?;
    for f in ficheros(origen)? {
        let nombre = f.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if !nombre.starts_with("~$") {
            traer(&f, destino, t, |t| t.notas += 1)?;
        }
    }
    for d in fs::read_dir(origen).map_err(|e| leer(origen, e))?.flatten() {
        if d.path().is_dir() {
            copiar_notas(&d.path(), &destino.join(d.file_name()), t)?;
        }
    }
    Ok(())
}

/// Los ficheros (no carpetas) de `carpeta`, ordenados.
fn ficheros(carpeta: &Path) -> Result<Vec<PathBuf>, String> {
    let mut v: Vec<PathBuf> = fs::read_dir(carpeta)
        .map_err(|e| leer(carpeta, e))?
        .flatten()
        .map(|d| d.path())
        .filter(|p| p.is_file())
        .collect();
    v.sort();
    Ok(v)
}

/// Copia `f` a `carpeta` sin pisar nada y comprueba la copia.
fn traer(f: &Path, carpeta: &Path, t: &mut Traido, contar: impl FnOnce(&mut Traido)) -> Result<(), String> {
    let original = fs::read(f).map_err(|e| leer(f, e))?;
    let nombre = f.file_name().ok_or_else(|| format!("{} no tiene nombre", f.display()))?;
    let mut destino = carpeta.join(nombre);
    let mut n = 1;
    loop {
        match fs::read(&destino) {
            Ok(hay) if hay == original => {
                t.ya_estaban += 1;
                return Ok(());
            }
            Ok(_) => {
                destino = carpeta.join(otro_nombre(Path::new(nombre), n));
                n += 1;
            }
            Err(_) => break,
        }
    }
    fs::write(&destino, &original)
        .map_err(|e| format!("no se puede copiar {} a {}: {e}", f.display(), destino.display()))?;
    if fs::read(&destino).map_err(|e| leer(&destino, e))? != original {
        return Err(format!("la copia de {} en {} no ha salido igual", f.display(), destino.display()));
    }
    if n > 1 {
        t.renombrados.push(destino.file_name().unwrap_or_default().to_string_lossy().into_owned());
    }
    contar(t);
    Ok(())
}

/// `notas.xlsx` → `notas (de la v1).xlsx`, `notas (de la v1, 2).xlsx`…
fn otro_nombre(nombre: &Path, n: usize) -> String {
    let base = nombre.file_stem().unwrap_or_default().to_string_lossy();
    let ext = nombre.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    if n == 1 {
        format!("{base} (de la v1){ext}")
    } else {
        format!("{base} (de la v1, {n}){ext}")
    }
}

/// Si `carpeta` (la de plantillas de los ajustes) está dentro de la de la v1:
/// al desinstalarla podría desaparecer, así que pasa a la copia.
pub fn apunta_a_la_v1(carpeta: &Path, v1: &Path) -> bool {
    let plegar = |p: &Path| p.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase();
    let (c, v) = (plegar(carpeta), plegar(v1));
    c == v || c.starts_with(&format!("{v}\\"))
}

fn crear(p: &Path, e: std::io::Error) -> String {
    format!("no se puede crear {}: {e}", p.display())
}

fn leer(p: &Path, e: std::io::Error) -> String {
    format!("no se puede leer {}: {e}", p.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Una v1 inventada: dos plantillas, un README y notas con una vieja.
    fn v1(raiz: &Path) -> PathBuf {
        let v1 = raiz.join("GestorNotasClickedu");
        let p = v1.join(PLANTILLAS_V1);
        fs::create_dir_all(&p).unwrap();
        fs::write(p.join("biologia_1_ab_bach.xlsx"), b"biologia").unwrap();
        fs::write(p.join("quimica_2_bach.xls"), b"quimica").unwrap();
        fs::write(p.join("README_PLANTILLAS.txt"), b"lee esto").unwrap();
        fs::write(p.join("~$biologia_1_ab_bach.xlsx"), b"bloqueo de Excel").unwrap();
        let n = v1.join(NOTAS_V1);
        fs::create_dir_all(n.join("notas_viejas")).unwrap();
        fs::write(n.join("notas_biologia.xlsx"), b"notas").unwrap();
        fs::write(n.join("notas_viejas").join("notas_biologia_1.xlsx"), b"viejas").unwrap();
        v1
    }

    fn carpeta_temporal(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gestornotas-migracion-{nombre}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn copia_plantillas_y_notas_sin_lo_demas() {
        let raiz = carpeta_temporal("copia");
        let (pl, no) = (raiz.join("docs").join("plantillas"), raiz.join("docs").join("notas"));
        let t = copiar(&v1(&raiz), &pl, &no).unwrap();
        assert_eq!(t, Traido { clases: 2, notas: 2, ya_estaban: 0, renombrados: vec![] });
        assert_eq!(fs::read(pl.join("biologia_1_ab_bach.xlsx")).unwrap(), b"biologia");
        assert!(!pl.join("README_PLANTILLAS.txt").exists());
        assert!(!pl.join("~$biologia_1_ab_bach.xlsx").exists());
        assert_eq!(fs::read(no.join("notas_viejas").join("notas_biologia_1.xlsx")).unwrap(), b"viejas");
        // Lo de la v1, intacto.
        assert!(raiz.join("GestorNotasClickedu").join(PLANTILLAS_V1).join("quimica_2_bach.xls").exists());
        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn otra_vez_cuenta_lo_que_ya_estaba() {
        let raiz = carpeta_temporal("otra-vez");
        let (pl, no) = (raiz.join("plantillas"), raiz.join("notas"));
        let v1 = v1(&raiz);
        copiar(&v1, &pl, &no).unwrap();
        let t = copiar(&v1, &pl, &no).unwrap();
        assert_eq!(t, Traido { clases: 0, notas: 0, ya_estaban: 4, renombrados: vec![] });
        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn uno_distinto_no_se_pisa() {
        let raiz = carpeta_temporal("distinto");
        let (pl, no) = (raiz.join("plantillas"), raiz.join("notas"));
        fs::create_dir_all(&pl).unwrap();
        fs::write(pl.join("biologia_1_ab_bach.xlsx"), b"la de la v2").unwrap();
        let v1 = v1(&raiz);
        let t = copiar(&v1, &pl, &no).unwrap();
        assert_eq!(t.clases, 2);
        assert_eq!(t.renombrados, vec!["biologia_1_ab_bach (de la v1).xlsx".to_string()]);
        assert_eq!(fs::read(pl.join("biologia_1_ab_bach.xlsx")).unwrap(), b"la de la v2");
        assert_eq!(fs::read(pl.join("biologia_1_ab_bach (de la v1).xlsx")).unwrap(), b"biologia");
        // Y al repetir, la renombrada cuenta como ya traída.
        let t = copiar(&v1, &pl, &no).unwrap();
        assert_eq!((t.clases, t.ya_estaban), (0, 4));
        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn sin_carpetas_de_datos_no_trae_nada() {
        let raiz = carpeta_temporal("vacia");
        let t = copiar(&raiz.join("no-esta"), &raiz.join("p"), &raiz.join("n")).unwrap();
        assert_eq!(t, Traido::default());
        fs::remove_dir_all(raiz).unwrap();
    }

    /// Solo lee el registro: `cargo test --lib -- --ignored v1_de_este`.
    #[test]
    #[ignore]
    fn la_v1_de_este_ordenador() {
        println!("{:?}", buscar_v1());
    }

    #[test]
    fn la_carpeta_de_los_ajustes_dentro_de_la_v1() {
        let v1 = Path::new(r"C:\Users\profe\AppData\Local\GestorNotasClickedu\");
        assert!(apunta_a_la_v1(
            Path::new(r"C:\Users\profe\AppData\Local\GestorNotasClickedu\plantillas_clickedu_aulas"),
            v1
        ));
        assert!(apunta_a_la_v1(Path::new(r"c:\users\PROFE\appdata\local\gestornotasclickedu"), v1));
        assert!(!apunta_a_la_v1(Path::new(r"C:\Users\profe\AppData\Local\GestorNotasClickeduOtra"), v1));
        assert!(!apunta_a_la_v1(Path::new(r"C:\Users\profe\Documents\Gestor de Notas\plantillas"), v1));
    }
}
