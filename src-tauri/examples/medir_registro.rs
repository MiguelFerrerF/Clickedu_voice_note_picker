//! Medir en local (D8): pasa los dictados guardados en el registro por la
//! versión de ahora y cuenta, **sin enseñar ningún nombre**.
//!
//!     cargo run --example medir_registro [-- --escritas] [-- <carpeta del registro>]
//!
//! Lee el registro de `%APPDATA%\es.gestornotas.app\registro` y los alumnos
//! de cada clase de la carpeta de plantillas de los Ajustes. Cada dictado se
//! juzga con lo que hizo el profesor después (`registro::medir`): lo que
//! decidió la app **entonces** y lo que decide **ahora**. Las frases que
//! cambian se enseñan tapadas como en el informe («▢5 ▢6 siete con cinco»).
//!
//! Por defecto, solo lo dictado con ESPACIO; `--escritas` cuenta también lo
//! escrito en el campo de desarrollo. Sale con error si ahora hay más notas
//! **mal puestas en silencio** que entonces.
//!
//! Mide de lo oído en adelante: el audio no se guarda, así que un cambio de
//! Whisper se mide con las grabaciones de H3a (`probar_entender`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gestor_de_notas_lib::ajustes;
use gestor_de_notas_lib::plantillas;
use gestor_de_notas_lib::registro::informe::Tapar;
use gestor_de_notas_lib::registro::medir::{contar, medir, Veredicto};
use gestor_de_notas_lib::registro::Registro;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let escritas = args.iter().any(|a| a == "--escritas");
    let app = dirs::config_dir().expect("no se encuentra %APPDATA%").join("es.gestornotas.app");
    let carpeta_registro =
        args.iter().find(|a| !a.starts_with("--")).map(PathBuf::from).unwrap_or_else(|| app.join("registro"));

    let documentos = dirs::document_dir().expect("no se encuentra Documentos");
    let ajustes = ajustes::leer(&app.join("ajustes.json")).unwrap_or_else(|e| panic!("{e}"));
    let plantillas = PathBuf::from(ajustes::carpeta_plantillas(&ajustes, &documentos).ruta);

    let casos = Registro::new(carpeta_registro.clone()).leer().unwrap_or_else(|e| panic!("{e}"));
    println!("Registro: {}", carpeta_registro.display());
    println!("Plantillas: {}", plantillas.display());

    // Los alumnos de cada clase, leídos una vez: para entender y para tapar.
    let mut alumnos: HashMap<String, Option<Vec<(String, String)>>> = HashMap::new();
    let mut leer =
        |fichero: &str| alumnos.entry(fichero.to_string()).or_insert_with(|| clase(&plantillas, fichero)).clone();
    let medida = medir(&casos, escritas, &mut leer);

    println!(
        "\n{} dictados medidos{}{}.",
        medida.medidos.len(),
        if medida.escritas > 0 {
            format!("; {} escritos en desarrollo no cuentan (--escritas para contarlos)", medida.escritas)
        } else {
            String::new()
        },
        if medida.sin_clase > 0 {
            format!("; {} de clases que ya no se leen", medida.sin_clase)
        } else {
            String::new()
        },
    );
    if medida.medidos.is_empty() {
        return;
    }

    let antes = contar(&medida.medidos, |m| m.antes);
    let ahora = contar(&medida.medidos, |m| m.ahora);
    println!("\n{:<32} {:>8} {:>8}", "", "Entonces", "Ahora");
    for v in Veredicto::TODOS {
        let (a, b) = (antes.get(&v).copied().unwrap_or(0), ahora.get(&v).copied().unwrap_or(0));
        let marca = if a != b { "  ←" } else { "" };
        println!("{:<32} {a:>8} {b:>8}{marca}", v.en_palabras());
    }

    // Lo que cambia, tapado, con la clase como «clase 1».
    let mut numeros: HashMap<&str, usize> = HashMap::new();
    let cambian: Vec<_> = medida.medidos.iter().filter(|m| m.antes != m.ahora).collect();
    if !cambian.is_empty() {
        println!("\nCambian {}:", cambian.len());
    }
    for m in cambian {
        let n = numeros.len() + 1;
        let n = *numeros.entry(m.fichero.as_str()).or_insert(n);
        let nombres: Vec<String> = leer(&m.fichero).unwrap_or_default().into_iter().map(|(_, nombre)| nombre).collect();
        let frase = Tapar::new(&nombres, Vec::new()).frase(&m.oido);
        println!("  [clase {n}] «{frase}»: {} → {}", m.antes.en_palabras(), m.ahora.en_palabras());
    }

    let (mal_antes, mal_ahora) = (
        antes.get(&Veredicto::MalEnSilencio).copied().unwrap_or(0),
        ahora.get(&Veredicto::MalEnSilencio).copied().unwrap_or(0),
    );
    if mal_ahora > mal_antes {
        println!("\nAhora hay MÁS notas mal puestas en silencio ({mal_ahora}) que entonces ({mal_antes}).");
        std::process::exit(1);
    }
}

/// Los alumnos de la plantilla `fichero`, como `(clave, nombre)`.
fn clase(carpeta: &Path, fichero: &str) -> Option<Vec<(String, String)>> {
    let p = plantillas::leer(&carpeta.join(fichero)).ok()?;
    Some(p.alumnos.into_iter().map(|a| (a.clave, a.nombre)).collect())
}
