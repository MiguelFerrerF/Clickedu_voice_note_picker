//! **Traer las plantillas** desde Clickedu (H5, `docs/01`): una por materia,
//! sueltas en la carpeta de plantillas y con un nombre que sale siempre de
//! la misma regla.
//!
//! - [`nombres`] — el nombre de cada fichero, de los rótulos de Clickedu
//!   (`‹Materia› - ‹Curso› ‹Grupo›.xlsx`; las pendientes,
//!   `Pendiente - ‹Materia› - ‹Curso›.xlsx`). Puro.
//! - [`bajar`] — de cada materia, su primera evaluación y la plantilla de
//!   esa evaluación (**la plantilla no cambia con la evaluación**,
//!   comprobado el 2026-11-22), y los ítems de todas sus evaluaciones
//!   ([`super::items`], H6). Solo red: no escribe nada.
//! - [`preparar`] y [`leer_raiz`] — lo bajado y lo que ya hay en la
//!   carpeta, leídos.
//! - [`planear`] — qué se hace con cada materia y con lo que ya hay en la
//!   carpeta, y a dónde pasan las notas. Puro.
//! - [`aplicar`] — hacerlo: apartar a `antiguas`, escribir las nuevas y
//!   pasar las notas.
//!
//! Clickedu llama a todas igual (`evaluate_‹AAAAMMDD›_‹HHMMSS›.xlsx`): el
//! nombre del fichero no dice qué materia es, así que la regla es la única
//! manera.

use std::path::Path;

use serde::Serialize;

use super::cliente::{self, Cliente, Fallo};
use super::fichas::{self, Estado, Evaluacion, Leido, Materia, Roto};
use super::items::{self, EvaluacionItems};
use crate::plantillas::{self, plegar, ErrorPlantilla};
use planear::{Contenido, EnLaRaiz, FalloMateria, Traida};

pub mod aplicar;
pub mod planear;

// ───────────────────────── Bajar ─────────────────────────

/// La página de Ítems de una materia: sus evaluaciones (`trm`).
pub fn direccion_items(materia: &str) -> String {
    format!("/go/Index.php?ctr=Subject/Items&do=Index&id={materia}")
}

/// La plantilla de una materia y evaluación, la que bajaba la v1.
pub fn direccion_plantilla(materia: &str, evaluacion: &str) -> String {
    format!("/assignatures/importar_notes_items.php?accio=descarregar_plantilla&assig={materia}&id_aval={evaluacion}")
}

/// Lo bajado de todas: las materias y lo de cada una, en su orden.
pub struct Bajado {
    pub materias: Leido<Vec<Materia>>,
    /// La página de las materias, para su huella. Lleva nombres: en local.
    pub pagina_materias: String,
    pub bajadas: Vec<Bajada>,
}

/// Lo bajado de una materia.
pub struct Bajada {
    pub materia: Materia,
    /// La página de Ítems, para su huella. Lleva nombres: en local.
    pub pagina_items: String,
    pub resultado: Result<Plantilla, FalloBajada>,
}

/// La plantilla de una materia, recién bajada.
pub struct Plantilla {
    /// Con la que se ha bajado: la primera de la materia.
    pub evaluacion: Evaluacion,
    /// Cómo se han leído las evaluaciones: degradado es el aviso temprano.
    pub estado_evaluaciones: Estado,
    /// El `.xlsx`, tal cual.
    pub excel: Vec<u8>,
    /// Los ítems de cada evaluación (H6). Que no se entiendan no quita la
    /// plantilla.
    pub items: Result<Vec<EvaluacionItems>, Roto>,
}

/// Por qué no se ha podido bajar la plantilla de una materia. No para las
/// demás.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum FalloBajada {
    /// Su página de Ítems no se entiende: Clickedu ha cambiado.
    Evaluaciones { roto: Roto },
    /// En vez de un Excel ha llegado otra cosa. Sin el cuerpo, que puede
    /// llevar nombres.
    NoEsExcel { estado: u16, contenido: String },
}

/// Por qué no se ha podido traer nada.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum FalloTraer {
    /// No se llega a Clickedu, o la sesión no vuelve: lo que se bajara
    /// después fallaría igual, así que se para. Aún no se ha escrito nada.
    Conexion { fallo: Fallo },
    /// La página de las materias no se entiende: Clickedu ha cambiado.
    Materias {
        roto: Roto,
        #[serde(skip)]
        pagina: String,
    },
    /// No se ha entrado en Clickedu en esta sesión de la app.
    SinEntrar,
    /// Lo demás: el disco, sobre todo.
    Otro { detalle: String },
}

impl std::fmt::Display for FalloTraer {
    /// Para el registro (D8): sin páginas.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FalloTraer::Conexion { fallo } => fallo.fmt(f),
            FalloTraer::Materias { roto, .. } => roto.fmt(f),
            FalloTraer::SinEntrar => f.write_str("no se ha entrado en Clickedu"),
            FalloTraer::Otro { detalle } => f.write_str(detalle),
        }
    }
}

impl From<String> for FalloTraer {
    fn from(detalle: String) -> Self {
        FalloTraer::Otro { detalle }
    }
}

impl From<Fallo> for FalloTraer {
    fn from(fallo: Fallo) -> Self {
        FalloTraer::Conexion { fallo }
    }
}

/// Baja la plantilla de cada materia, una tras otra, con la sesión de
/// `cliente`, y sus ítems si la plantilla ha llegado. `progreso(hechas,
/// total)` tras cada materia.
pub async fn bajar(cliente: &Cliente, mut progreso: impl FnMut(usize, usize)) -> Result<Bajado, FalloTraer> {
    let pagina_materias = cliente.abrir(cliente::ASIGNATURAS).await?.texto;
    let materias = match fichas::materias(&pagina_materias) {
        Ok(m) => m,
        Err(roto) => return Err(FalloTraer::Materias { roto, pagina: pagina_materias }),
    };
    let total = materias.dato.len();
    let mut bajadas = Vec::with_capacity(total);
    for (i, materia) in materias.dato.iter().enumerate() {
        let pagina_items = cliente.abrir(&direccion_items(&materia.id)).await?.texto;
        let resultado = match fichas::evaluaciones(&pagina_items) {
            Err(roto) => Err(FalloBajada::Evaluaciones { roto }),
            Ok(Leido { dato, estado }) => {
                // `evaluaciones` no da nunca la lista vacía: sería roto.
                let evaluacion = dato.first().expect("al menos una evaluación").clone();
                let leida = cliente.bajar(&direccion_plantilla(&materia.id, &evaluacion.id)).await?;
                if leida.estado == 200 && leida.cuerpo.starts_with(b"PK") {
                    let items = items::bajar(cliente, &materia.id, &dato).await?;
                    Ok(Plantilla { evaluacion, estado_evaluaciones: estado, excel: leida.cuerpo, items })
                } else {
                    Err(FalloBajada::NoEsExcel { estado: leida.estado, contenido: leida.tipo })
                }
            }
        };
        bajadas.push(Bajada { materia: materia.clone(), pagina_items, resultado });
        progreso(i + 1, total);
    }
    Ok(Bajado { materias, pagina_materias, bajadas })
}

// ───────────────────────── Preparar ─────────────────────────

/// Lo bajado, a `carpeta\‹fichero›` (`clickedu\bajadas\`) y leído, para
/// [`planear`]. Una plantilla sin alumnos (una pendiente puede venir así:
/// la hoja y nada más) es vacía y no se deja.
pub fn preparar(bajadas: Vec<Bajada>, carpeta: &Path) -> Result<Vec<Traida>, String> {
    std::fs::create_dir_all(carpeta).map_err(|e| format!("no se puede crear {}: {e}", carpeta.display()))?;
    let materias: Vec<Materia> = bajadas.iter().map(|b| b.materia.clone()).collect();
    let mut res = Vec::with_capacity(bajadas.len());
    for (b, nombre) in bajadas.into_iter().zip(nombres(&materias)) {
        let contenido = match b.resultado {
            Err(fallo) => Contenido::Fallo(FalloMateria::Bajar { fallo }),
            Ok(p) => {
                let ruta = carpeta.join(&nombre.fichero);
                std::fs::write(&ruta, &p.excel)
                    .map_err(|e| format!("no se puede guardar lo bajado en {}: {e}", ruta.display()))?;
                match plantillas::leer(&ruta) {
                    Ok(plantilla) => Contenido::Plantilla { plantilla, estado_evaluaciones: p.estado_evaluaciones },
                    Err(e) => {
                        let _ = std::fs::remove_file(&ruta);
                        match e {
                            ErrorPlantilla::SinDatos | ErrorPlantilla::SinAlumnos => Contenido::Vacia,
                            otro => Contenido::Fallo(FalloMateria::NoSeLee { error: otro.to_string() }),
                        }
                    }
                }
            }
        };
        res.push(Traida { materia: b.materia, nombre, contenido });
    }
    Ok(res)
}

/// Las plantillas de la raíz de `carpeta`, leídas: sin las subcarpetas
/// (`antiguas` no cuenta). Si la carpeta no está, ninguna.
pub fn leer_raiz(carpeta: &Path) -> Result<Vec<EnLaRaiz>, String> {
    let entradas = match std::fs::read_dir(carpeta) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("no se puede leer {}: {e}", carpeta.display())),
    };
    let mut res = Vec::new();
    for entrada in entradas.flatten() {
        let fichero = entrada.file_name().to_string_lossy().into_owned();
        if !entrada.file_type().is_ok_and(|t| t.is_file()) || !plantillas::es_plantilla(&fichero) {
            continue;
        }
        res.push(EnLaRaiz { lectura: plantillas::leer(&entrada.path()).map_err(|e| e.to_string()), fichero });
    }
    res.sort_by(|a, b| a.fichero.cmp(&b.fichero));
    Ok(res)
}

// ───────────────────────── Nombres ─────────────────────────

/// Un retoque de forma en lo que pone Clickedu, para que sirva de nombre de
/// fichero: **se enseña** (ninguna normalización en silencio).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Retoque {
    pub de: String,
    pub a: String,
}

/// Lo que no ha salido como siempre al montar un nombre.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum AvisoNombre {
    /// El encabezado no es de ESO ni de Bachillerato: va tal cual.
    CursoTalCual { encabezado: String },
    /// La materia no trae encabezado (las leídas solo por sus enlaces).
    SinCurso,
}

/// El nombre del fichero de una materia.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Nombre {
    /// El `id` de la materia.
    pub id: String,
    /// `Física y Química - 3º ESO A.xlsx`.
    pub fichero: String,
    /// De otro curso: va aparte, en «Pendientes».
    pub pendiente: bool,
    pub retoques: Vec<Retoque>,
    pub avisos: Vec<AvisoNombre>,
}

/// Los nombres de los ficheros de `materias`, en su orden.
///
/// - *Materia*: el título. *Curso*: el encabezado, abreviado («Tercero de
///   ESO» → «3º ESO», «Primero de Bachillerato» → «1º Bach»). *Grupo*: la
///   letra, si el subtítulo es un grupo («3º E.S.O.-A» → «A»).
/// - **Las pendientes** (el subtítulo empieza por «pendiente»): el curso es
///   el del final del título y, si solo trae el número, la etapa del
///   encabezado.
/// - Si dos se llamarían igual, el subtítulo entre paréntesis; si aun así,
///   también el `id`.
pub fn nombres(materias: &[Materia]) -> Vec<Nombre> {
    let mut res: Vec<Nombre> = materias.iter().map(nombre).collect();
    let tallos: Vec<String> = res.iter().map(|n| n.fichero.clone()).collect();

    // Los repetidos, con el subtítulo; los que aún se repiten, con el id.
    for paso in 0..2 {
        let repetidos = repetidos(&res.iter().map(|n| n.fichero.as_str()).collect::<Vec<_>>());
        for (i, n) in res.iter_mut().enumerate() {
            if !repetidos[i] {
                continue;
            }
            let sub = retocar(&materias[i].grupo, &mut n.retoques);
            let entre = match (paso, sub.is_empty()) {
                (0, false) => sub,
                (_, true) => materias[i].id.clone(),
                _ => format!("{sub}, {}", materias[i].id),
            };
            n.fichero = format!("{} ({entre})", tallos[i]);
        }
    }

    for n in &mut res {
        // Windows no acepta un nombre que acabe en punto o en espacio.
        let limpio = n.fichero.trim_end_matches(['.', ' ']).to_string();
        if limpio != n.fichero {
            n.retoques.push(Retoque { de: n.fichero.clone(), a: limpio.clone() });
        }
        n.fichero = format!("{limpio}.xlsx");
        n.retoques.dedup();
    }
    res
}

/// Cuáles se llaman como otra: Windows no distingue mayúsculas.
fn repetidos(nombres: &[&str]) -> Vec<bool> {
    let claves: Vec<String> = nombres.iter().map(|n| n.to_lowercase()).collect();
    claves.iter().map(|c| claves.iter().filter(|o| *o == c).count() > 1).collect()
}

/// El nombre de una materia, sin mirar las demás ni la extensión.
fn nombre(m: &Materia) -> Nombre {
    let mut retoques = Vec::new();
    let mut avisos = Vec::new();
    let pendiente = plegar(&m.grupo).starts_with("pendiente");

    // El curso del encabezado, abreviado; o tal cual, con aviso.
    let mut del_encabezado = |retoques: &mut Vec<Retoque>| match &m.curso {
        Some(e) => match curso_corto(e) {
            Some(c) => Some(c),
            None => {
                avisos.push(AvisoNombre::CursoTalCual { encabezado: e.clone() });
                Some(retocar(e, retoques))
            }
        },
        None => {
            avisos.push(AvisoNombre::SinCurso);
            None
        }
    };

    let fichero = if pendiente {
        let titulo = retocar(&m.nombre, &mut retoques);
        let palabras: Vec<&str> = titulo.split(' ').collect();
        let (materia, curso) = match palabras.as_slice() {
            [materia @ .., numero, etapa] if !materia.is_empty() && es_ordinal(numero) && etapa_de(etapa).is_some() => {
                (materia.join(" "), Some(format!("{numero} {}", etapa_de(etapa).unwrap_or_default())))
            }
            [materia @ .., numero] if !materia.is_empty() && es_ordinal(numero) => {
                let etapa =
                    m.curso.as_deref().and_then(curso_corto).and_then(|c| c.split(' ').nth(1).map(String::from));
                let curso = match etapa {
                    Some(etapa) => Some(format!("{numero} {etapa}")),
                    None => del_encabezado(&mut retoques).map(|c| format!("{numero} ({c})")),
                };
                (materia.join(" "), curso)
            }
            _ => (titulo.clone(), del_encabezado(&mut retoques)),
        };
        match curso {
            Some(c) => format!("{}{materia} - {c}", plantillas::PENDIENTE),
            None => format!("{}{materia}", plantillas::PENDIENTE),
        }
    } else {
        let materia = retocar(&m.nombre, &mut retoques);
        let curso_y_grupo: Vec<String> =
            [del_encabezado(&mut retoques), letra_de_grupo(&m.grupo)].into_iter().flatten().collect();
        if curso_y_grupo.is_empty() {
            materia
        } else {
            format!("{materia} - {}", curso_y_grupo.join(" "))
        }
    };
    Nombre { id: m.id.clone(), fichero, pendiente, retoques, avisos }
}

/// «Tercero de ESO» → `3º ESO`; «Primero de Bachillerato» → `1º Bach`. Lo
/// que no sea así, `None`.
fn curso_corto(encabezado: &str) -> Option<String> {
    let palabras: Vec<String> = encabezado.split_whitespace().map(plegar).collect();
    let [numero, de, etapa] = palabras.as_slice() else { return None };
    let numero = match numero.as_str() {
        "primero" => 1,
        "segundo" => 2,
        "tercero" => 3,
        "cuarto" => 4,
        _ => return None,
    };
    let etapa = etapa_de(etapa)?;
    (de == "de" && (etapa == "ESO" || numero <= 2)).then(|| format!("{numero}º {etapa}"))
}

/// `ESO`, `Bach` o `Bachillerato` (como sea de mayúsculas), a `ESO` o `Bach`.
fn etapa_de(palabra: &str) -> Option<&'static str> {
    match plegar(palabra).as_str() {
        "eso" => Some("ESO"),
        "bach" | "bachillerato" => Some("Bach"),
        _ => None,
    }
}

/// `3º`, `12º`.
fn es_ordinal(palabra: &str) -> bool {
    palabra.strip_suffix('º').is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// La letra del grupo, si el subtítulo es un grupo: `3º E.S.O.-A` → `A`. Lo
/// de tras el último `-`, una o dos mayúsculas, y delante el curso
/// (`3º ESO`, `1º Bach`).
fn letra_de_grupo(subtitulo: &str) -> Option<String> {
    let llano = retocar(subtitulo, &mut Vec::new());
    let (curso, letra) = llano.rsplit_once('-')?;
    let letra = letra.trim();
    let es_letra = (1..=2).contains(&letra.chars().count()) && letra.chars().all(|c| c.is_ascii_uppercase());
    let palabras: Vec<&str> = curso.split_whitespace().collect();
    let es_curso = matches!(palabras.as_slice(), [n, e] if es_ordinal(n) && etapa_de(e).is_some());
    (es_letra && es_curso).then(|| letra.to_string())
}

/// Lo que pone Clickedu, para un nombre de fichero: `E.S.O.` → `ESO`,
/// `2ºESO` → `2º ESO` y lo que Windows no acepta (`<>:"/\|?*`) → `-`. Cada
/// retoque, a `retoques`.
fn retocar(texto: &str, retoques: &mut Vec<Retoque>) -> String {
    let mut anotar = |de: &str, a: &str| {
        let r = Retoque { de: de.into(), a: a.into() };
        if !retoques.contains(&r) {
            retoques.push(r);
        }
    };
    let mut t = texto.to_string();
    if t.contains("E.S.O.") {
        anotar("E.S.O.", "ESO");
        t = t.replace("E.S.O.", "ESO");
    }
    // `2ºESO` → `2º ESO`: un `º` tras un número y pegado a una letra.
    let letras: Vec<char> = t.chars().collect();
    let mut separado = String::new();
    for (i, &c) in letras.iter().enumerate() {
        separado.push(c);
        let tras_numero = i > 0 && letras[i - 1].is_ascii_digit();
        if c == 'º' && tras_numero && letras.get(i + 1).is_some_and(|s| s.is_alphabetic()) {
            let inicio = letras[..i].iter().rposition(|c| !c.is_ascii_digit()).map_or(0, |p| p + 1);
            let fin = letras[i + 1..].iter().position(|c| !c.is_alphanumeric()).map_or(letras.len(), |p| i + 1 + p);
            let pegado: String = letras[inicio..fin].iter().collect();
            let numero: String = letras[inicio..=i].iter().collect();
            let resto: String = letras[i + 1..fin].iter().collect();
            anotar(&pegado, &format!("{numero} {resto}"));
            separado.push(' ');
        }
    }
    t = separado;
    for prohibido in ['<', '>', ':', '"', '/', '\\', '|', '?', '*'] {
        if t.contains(prohibido) {
            anotar(&prohibido.to_string(), "-");
            t = t.replace(prohibido, "-");
        }
    }
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickedu::falso::{self, Modo, ARCHIVO_BUENO, BAJAR_101, CONTRASENA, USUARIO};

    fn bajar_de(modo: Modo) -> (Result<Bajado, FalloTraer>, Vec<(usize, usize)>, Vec<String>) {
        let f = falso::arrancar(modo);
        let cliente = Cliente::new(&f.base).unwrap();
        tauri::async_runtime::block_on(cliente.entrar(USUARIO, CONTRASENA, ("a.txt", ARCHIVO_BUENO))).unwrap();
        let mut avance = Vec::new();
        let bajado = tauri::async_runtime::block_on(bajar(&cliente, |h, t| avance.push((h, t))));
        (bajado, avance, f.pedidas())
    }

    #[test]
    fn baja_cada_materia_y_una_que_falla_no_para_las_demas() {
        let (bajado, avance, _) = bajar_de(Modo::Normal);
        let bajado = bajado.unwrap();
        assert_eq!(bajado.materias.estado, Estado::Bien);
        assert_eq!(avance, [(1, 3), (2, 3), (3, 3)]);
        let [buena, sin_evaluaciones, no_excel] = bajado.bajadas.as_slice() else { panic!() };
        let p = buena.resultado.as_ref().unwrap();
        assert_eq!(p.evaluacion.id, "7");
        assert_eq!(p.excel, falso::plantilla_101());
        assert_eq!(buena.materia.nombre, "Matemáticas");
        assert!(buena.pagina_items.contains("1ª evaluación"));
        // Sus ítems, con cuántos tienen ya nota en Clickedu.
        let evaluaciones = p.items.as_ref().unwrap();
        assert_eq!(evaluaciones.len(), 1);
        let items: Vec<_> =
            evaluaciones[0].items.iter().map(|i| (i.id.as_str(), i.nombre.as_str(), i.en_clickedu)).collect();
        assert_eq!(items, [("9001", "Examen 1", 1), ("9002", "Trabajo", 0)]);
        assert!(
            matches!(&sin_evaluaciones.resultado, Err(FalloBajada::Evaluaciones { roto }) if roto.ficha == "evaluaciones")
        );
        assert!(
            matches!(&no_excel.resultado, Err(FalloBajada::NoEsExcel { estado: 200, contenido }) if contenido.starts_with("text/html"))
        );
    }

    #[test]
    fn si_caduca_al_bajar_vuelve_a_entrar() {
        let (bajado, _, pedidas) = bajar_de(Modo::Caduca);
        let bajado = bajado.unwrap();
        assert_eq!(bajado.bajadas[0].resultado.as_ref().unwrap().excel, falso::plantilla_101());
        // La descarga de la 101, sin sesión la primera vez: entra y la repite.
        let bajar = format!("GET {BAJAR_101}");
        let i = pedidas.iter().position(|p| *p == bajar).unwrap();
        assert_eq!(pedidas[i + 1], "POST /user.php?action=doLogin");
        assert_eq!(pedidas.iter().skip(i + 1).find(|p| p.contains("descarregar")), Some(&bajar));
    }

    #[test]
    fn preparar_deja_las_buenas_y_no_las_vacias() {
        let materia = |id: &str, nombre: &str| Materia {
            id: id.into(),
            nombre: nombre.into(),
            grupo: "3º E.S.O.-A".into(),
            curso: Some("Tercero de ESO".into()),
        };
        let bajada = |m: Materia, excel: Vec<u8>| Bajada {
            materia: m,
            pagina_items: String::new(),
            resultado: Ok(Plantilla {
                evaluacion: Evaluacion {
                    id: "7".into(), nombre: "1ª".into(), desde: None, hasta: None, elegida: true
                },
                estado_evaluaciones: Estado::Bien,
                excel,
                items: Ok(vec![]),
            }),
        };
        // Como la pendiente de verdad: la hoja «Worksheet» y nada más.
        let mut libro = rust_xlsxwriter::Workbook::new();
        libro.add_worksheet().set_name("Worksheet").unwrap();
        let vacia = libro.save_to_buffer().unwrap();
        let bajadas = vec![
            bajada(materia("1", "Matemáticas"), falso::plantilla_101()),
            bajada(materia("2", "Música"), vacia),
            bajada(materia("3", "Tecnología"), b"PK no es un zip".to_vec()),
            Bajada {
                materia: materia("4", "Inglés"),
                pagina_items: String::new(),
                resultado: Err(FalloBajada::NoEsExcel { estado: 500, contenido: String::new() }),
            },
        ];
        let dir = tempfile::tempdir().unwrap();
        let traidas = preparar(bajadas, &dir.path().join("bajadas")).unwrap();
        assert!(
            matches!(&traidas[0].contenido, Contenido::Plantilla { plantilla, .. } if plantilla.alumnos.len() == 2)
        );
        assert!(matches!(traidas[1].contenido, Contenido::Vacia));
        assert!(matches!(traidas[2].contenido, Contenido::Fallo(FalloMateria::NoSeLee { .. })));
        assert!(matches!(traidas[3].contenido, Contenido::Fallo(FalloMateria::Bajar { .. })));
        // Solo queda la buena, con su nombre.
        let quedan: Vec<String> = std::fs::read_dir(dir.path().join("bajadas"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(quedan, ["Matemáticas - 3º ESO A.xlsx"]);
        // Y leída desde la raíz, igual.
        let raiz = leer_raiz(&dir.path().join("bajadas")).unwrap();
        assert_eq!(raiz[0].lectura.as_ref().unwrap().alumnos.len(), 2);
        assert!(leer_raiz(&dir.path().join("no está")).unwrap().is_empty());
    }

    #[test]
    fn sin_conexion_para() {
        // Un puerto que nadie escucha.
        let puerto = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let cliente = Cliente::new(&format!("http://127.0.0.1:{puerto}")).unwrap();
        let bajado = tauri::async_runtime::block_on(bajar(&cliente, |_, _| {}));
        assert!(matches!(bajado, Err(FalloTraer::Conexion { .. })), "{:?}", bajado.err());
    }

    fn materia(id: &str, nombre: &str, grupo: &str, curso: Option<&str>) -> Materia {
        Materia { id: id.into(), nombre: nombre.into(), grupo: grupo.into(), curso: curso.map(String::from) }
    }

    fn ficheros(materias: &[Materia]) -> Vec<String> {
        nombres(materias).into_iter().map(|n| n.fichero).collect()
    }

    #[test]
    fn el_curso_abreviado() {
        assert_eq!(curso_corto("Tercero de ESO").as_deref(), Some("3º ESO"));
        assert_eq!(curso_corto("cuarto de E.S.O").as_deref(), None);
        assert_eq!(curso_corto("Primero de Bachillerato").as_deref(), Some("1º Bach"));
        assert_eq!(curso_corto("Segundo de Bachillerato").as_deref(), Some("2º Bach"));
        assert_eq!(curso_corto("Tercero de Bachillerato"), None);
        assert_eq!(curso_corto("Ciclo de Grado Medio"), None);
    }

    #[test]
    fn la_letra_del_grupo() {
        assert_eq!(letra_de_grupo("3º E.S.O.-A").as_deref(), Some("A"));
        assert_eq!(letra_de_grupo("1º Bach-BC").as_deref(), Some("BC"));
        assert_eq!(letra_de_grupo("QUÍMICA 2"), None);
        assert_eq!(letra_de_grupo("F/Q 1"), None);
        assert_eq!(letra_de_grupo("PENDIENTE FÍSICA Y QUÍMICA 2ºESO"), None);
        assert_eq!(letra_de_grupo("3º E.S.O.-Ab"), None);
    }

    #[test]
    fn los_retoques_se_anotan() {
        let mut r = Vec::new();
        assert_eq!(retocar("Física y Química 2ºESO", &mut r), "Física y Química 2º ESO");
        assert_eq!(retocar("F/Q: 1", &mut r), "F-Q- 1");
        assert_eq!(
            r,
            [
                Retoque { de: "2ºESO".into(), a: "2º ESO".into() },
                Retoque { de: ":".into(), a: "-".into() },
                Retoque { de: "/".into(), a: "-".into() },
            ]
        );
        assert_eq!(retocar("3º E.S.O.-A", &mut Vec::new()), "3º ESO-A");
    }

    /// Las de la página de verdad, anonimizada: las 10 de la cuenta.
    #[test]
    fn las_de_verdad() {
        let pagina = include_str!("../../tests/datos/clickedu/materias.html");
        let materias = fichas::materias(pagina).unwrap().dato;
        let n = nombres(&materias);
        let f: Vec<&str> = n.iter().map(|n| n.fichero.as_str()).collect();
        assert_eq!(
            f,
            [
                "Biología y Geología - 3º ESO A.xlsx",
                "Física y Química - 3º ESO A.xlsx",
                "Biología y Geología - 3º ESO B.xlsx",
                "Pendiente - Física y Química - 2º ESO.xlsx",
                "Pendiente - Biología y Geología - 3º ESO.xlsx",
                // La página anonimizada tapa el subtítulo de las dos: van con el id.
                &format!("Física y Química - 1º Bach (F-Q ▢, {}).xlsx", materias[5].id),
                &format!("Física y Química - 1º Bach (F-Q ▢, {}).xlsx", materias[6].id),
                "Química - 2º Bach (QUÍMICA 2).xlsx",
                "Química - 2º Bach (QUÍMICA 1).xlsx",
                "Pendiente - Física y Química - 1º Bach.xlsx",
            ]
        );
        let pendientes: Vec<bool> = n.iter().map(|n| n.pendiente).collect();
        assert_eq!(pendientes, [false, false, false, true, true, false, false, false, false, true]);
        // El encabezado tapado («▢ de ESO») no se usa: la pendiente trae su curso.
        assert!(n.iter().all(|n| n.avisos.is_empty()), "{n:?}");
        assert_eq!(n[3].retoques, [Retoque { de: "2ºESO".into(), a: "2º ESO".into() }]);
        assert_eq!(n[5].retoques, [Retoque { de: "/".into(), a: "-".into() }]);
        assert!(n[0].retoques.is_empty() && n[7].retoques.is_empty());
    }

    #[test]
    fn repetidas_por_el_subtitulo() {
        let f = ficheros(&[
            materia("1", "Física y Química", "F/Q 1", Some("Primero de Bachillerato")),
            materia("2", "Física y Química", "F/Q 2", Some("Primero de Bachillerato")),
            materia("3", "Física y Química", "1º Bach-A", Some("Primero de Bachillerato")),
        ]);
        assert_eq!(
            f,
            [
                "Física y Química - 1º Bach (F-Q 1).xlsx",
                "Física y Química - 1º Bach (F-Q 2).xlsx",
                "Física y Química - 1º Bach A.xlsx",
            ]
        );
        // Sin subtítulo, con el id; y sin distinguir mayúsculas, como Windows.
        let f = ficheros(&[
            materia("7", "Química", "", Some("Segundo de Bachillerato")),
            materia("8", "QUÍMICA", "", Some("Segundo de Bachillerato")),
        ]);
        assert_eq!(f, ["Química - 2º Bach (7).xlsx", "QUÍMICA - 2º Bach (8).xlsx"]);
    }

    #[test]
    fn el_curso_que_no_se_conoce_va_tal_cual() {
        let n = nombres(&[
            materia("1", "Tecnología", "CFGM-A", Some("Ciclo Formativo: Grado Medio")),
            materia("2", "Música", "", None),
        ]);
        assert_eq!(n[0].fichero, "Tecnología - Ciclo Formativo- Grado Medio.xlsx");
        assert_eq!(n[0].avisos, [AvisoNombre::CursoTalCual { encabezado: "Ciclo Formativo: Grado Medio".into() }]);
        assert_eq!(n[0].retoques, [Retoque { de: ":".into(), a: "-".into() }]);
        assert_eq!(n[1].fichero, "Música.xlsx");
        assert_eq!(n[1].avisos, [AvisoNombre::SinCurso]);
    }

    #[test]
    fn las_pendientes() {
        let f = ficheros(&[
            // El curso, del final del título.
            materia("1", "Lengua 1º Bachillerato", "PENDIENTE LENGUA", Some("Segundo de Bachillerato")),
            // Solo el número: la etapa, del encabezado.
            materia("2", "Matemáticas 2º", "Pendiente MATES", Some("Tercero de ESO")),
            // Sin curso en el título: el del encabezado.
            materia("3", "Inglés", "PENDIENTE INGLÉS", Some("Cuarto de ESO")),
        ]);
        assert_eq!(
            f,
            [
                "Pendiente - Lengua - 1º Bach.xlsx",
                "Pendiente - Matemáticas - 2º ESO.xlsx",
                "Pendiente - Inglés - 4º ESO.xlsx",
            ]
        );
    }

    #[test]
    fn sin_punto_al_final() {
        let n = nombres(&[materia("1", "Ciencias Aplic.", "", None)]);
        assert_eq!(n[0].fichero, "Ciencias Aplic.xlsx");
        assert_eq!(n[0].retoques, [Retoque { de: "Ciencias Aplic.".into(), a: "Ciencias Aplic".into() }]);
    }
}
