//! Leer las plantillas de Clickedu (H1): una plantilla es una clase.
//!
//! **La plantilla no se escribe nunca** (D4): aquí solo se lee, y el fichero se
//! abre a mano en solo lectura, decidiendo el formato por la firma y no por la
//! extensión (como `importador::lector` de library-app).
//!
//! **El formato.** La v1 leía la primera hoja con la cabecera en la primera
//! fila y las columnas por posición: ID, nombre, nota. Aquí se buscan las
//! columnas **por su cabecera** (`Alumno`, `Nota`…), en catalán o en
//! castellano, y si no se reconocen se vuelve a lo de la v1 **diciéndolo**
//! ([`Aviso::ColumnasPorPosicion`]).
//!
//! **Ninguna normalización en silencio**: lo que se corrige al leer (una nota
//! con coma) o lo que no cuadra (una nota que no es nota, un ID repetido)
//! vuelve como [`Aviso`], para que la interfaz lo diga. Lo que no es nota
//! **no se tira**: se queda como [`Nota::Invalida`], con su texto, para
//! pintarlo en rojo.
//!
//! **El nombre se deja tal cual** viene, con sus espacios: las plantillas
//! reales traen dobles espacios dentro de algún nombre (es dato de Clickedu,
//! el profesor no puede arreglarlo). No se corrige, así que no hay que avisar;
//! la pantalla ya los pinta como uno, y el dictado (H3) compara normalizando
//! por dentro, sin tocar el dato.
//!
//! El formato real, visto el 2026-11-18 en siete plantillas de Clickedu: una
//! hoja `Worksheet`, cabecera `Id | Alumnos | Nota` en la fila 1, el ID como
//! número (o como texto con cifras), el nombre `Apellidos, Nombre` y la nota
//! vacía.

use std::fmt;
use std::io::{BufReader, Read};
use std::path::Path;

use calamine::{Data, Range, Reader, Xls, Xlsx};
use serde::Serialize;

// ─────────────────────────────────────────────────────────────────────────────
// Lo que sale
// ─────────────────────────────────────────────────────────────────────────────

/// Una plantilla leída: sus alumnos, de dónde salió cada cosa y los avisos.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plantilla {
    pub alumnos: Vec<Alumno>,
    pub columnas: Columnas,
    pub avisos: Vec<Aviso>,
}

impl Plantilla {
    /// Cuántos alumnos tienen ya una nota válida en la plantilla.
    pub fn calificados(&self) -> usize {
        self.alumnos.iter().filter(|a| matches!(a.nota, Nota::Valor(_))).count()
    }
}

/// Un alumno, con la fila del Excel donde está (contando desde 1, como Excel).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alumno {
    pub fila: u32,
    pub id: String,
    pub nombre: String,
    pub nota: Nota,
    /// Con qué se guarda su nota en la sesión (H2): el ID, que no cambia
    /// aunque Clickedu reordene la lista; si no tiene o está repetido, la
    /// fila (`fila:7`).
    pub clave: String,
}

/// La nota de la plantilla, tal como venía.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", content = "valor", rename_all = "snake_case")]
pub enum Nota {
    /// Sin poner: lo normal al empezar.
    Vacia,
    /// Un número entre 0 y 10.
    Valor(f64),
    /// Algo que no es nota (`12`, `NP`, `#N/A`), con su texto tal cual.
    Invalida(String),
}

/// De dónde se ha sacado cada cosa: la hoja, la fila de cabecera y las tres
/// columnas, con el texto de su cabecera.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Columnas {
    pub hoja: String,
    /// Contando desde 1, como Excel.
    pub fila_cabecera: u32,
    pub id: Columna,
    pub nombre: Columna,
    pub nota: Columna,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Columna {
    /// Desde 0: la A es la 0.
    pub indice: u32,
    pub cabecera: String,
}

/// Lo que se ha corregido o lo que no cuadra. Las filas, contando desde 1.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Aviso {
    /// No se reconocieron las cabeceras: se usan las tres primeras columnas,
    /// como hacía la v1.
    ColumnasPorPosicion,
    /// Notas escritas con coma (`7,5`), leídas como `7.5`.
    NotasConComa { filas: Vec<u32> },
    /// Valores en la columna de la nota que no son una nota de 0 a 10.
    NotasInvalidas { filas: Vec<u32> },
    /// Filas con ID pero sin nombre.
    SinNombre { filas: Vec<u32> },
    /// El mismo ID en más de una fila.
    IdRepetido { id: String, filas: Vec<u32> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorPlantilla {
    /// No se ha podido abrir: no existe, está bloqueado, está roto.
    NoAbre(String),
    /// Ni `.xlsx` ni `.xls` por dentro, aunque lo diga la extensión.
    FormatoDesconocido,
    /// El libro no tiene ninguna hoja con datos.
    SinDatos,
    /// Hay datos, pero ninguna fila de alumno.
    SinAlumnos,
}

impl fmt::Display for ErrorPlantilla {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAbre(e) => write!(f, "no se puede abrir: {e}"),
            Self::FormatoDesconocido => write!(f, "no es un Excel (.xlsx o .xls)"),
            Self::SinDatos => write!(f, "no tiene ninguna hoja con datos"),
            Self::SinAlumnos => write!(f, "no tiene ningún alumno"),
        }
    }
}

impl std::error::Error for ErrorPlantilla {}

// ─────────────────────────────────────────────────────────────────────────────
// Leer
// ─────────────────────────────────────────────────────────────────────────────

/// Lee la plantilla de `ruta`.
pub fn leer(ruta: &Path) -> Result<Plantilla, ErrorPlantilla> {
    let (hoja, rango) = primera_hoja_con_datos(ruta)?;
    leer_rango(hoja, &rango)
}

/// La primera hoja que tenga algo escrito, con su nombre.
fn primera_hoja_con_datos(ruta: &Path) -> Result<(String, Range<Data>), ErrorPlantilla> {
    let no_abre = |e: &dyn fmt::Display| ErrorPlantilla::NoAbre(e.to_string());
    let mut firma = [0u8; 4];
    std::fs::File::open(ruta)
        .and_then(|mut f| f.read_exact(&mut firma))
        .map_err(|e| no_abre(&e))?;

    let fichero = std::fs::File::open(ruta).map_err(|e| no_abre(&e))?;
    let lector = BufReader::new(fichero);
    let hojas: Vec<(String, Range<Data>)> = match firma {
        [0xD0, 0xCF, 0x11, 0xE0] => Xls::new(lector).map_err(|e| no_abre(&e))?.worksheets(),
        [b'P', b'K', 3, 4] => Xlsx::new(lector).map_err(|e| no_abre(&e))?.worksheets(),
        _ => return Err(ErrorPlantilla::FormatoDesconocido),
    };
    hojas
        .into_iter()
        .find(|(_, r)| r.rows().any(|f| f.iter().any(|c| !texto(c).is_empty())))
        .ok_or(ErrorPlantilla::SinDatos)
}

/// Lee los alumnos de una hoja ya abierta. Público para los tests.
pub fn leer_rango(hoja: String, rango: &Range<Data>) -> Result<Plantilla, ErrorPlantilla> {
    let filas: Vec<&[Data]> = rango.rows().collect();
    // `rows()` empieza en la primera celda usada, no en la A1.
    let (fila0, col0) = rango.start().unwrap_or((0, 0));
    let mut avisos = Vec::new();

    let (i_cabecera, columnas) = match buscar_cabecera(&filas) {
        Some(encontrada) => encontrada,
        None => {
            avisos.push(Aviso::ColumnasPorPosicion);
            let i = filas
                .iter()
                .position(|f| f.iter().any(|c| !texto(c).is_empty()))
                .ok_or(ErrorPlantilla::SinDatos)?;
            (i, [0, 1, 2])
        }
    };
    let cabecera = filas[i_cabecera];
    let columna = |i: usize| Columna {
        indice: (col0 as usize + i) as u32,
        cabecera: cabecera.get(i).map(texto).unwrap_or_default(),
    };
    let columnas_leidas = Columnas {
        hoja,
        fila_cabecera: fila0 + i_cabecera as u32 + 1,
        id: columna(columnas[0]),
        nombre: columna(columnas[1]),
        nota: columna(columnas[2]),
    };

    let mut alumnos = Vec::new();
    let (mut con_coma, mut invalidas, mut sin_nombre) = (vec![], vec![], vec![]);
    for (i, f) in filas.iter().enumerate().skip(i_cabecera + 1) {
        let celda = |c: usize| f.get(c).unwrap_or(&Data::Empty);
        let fila = fila0 + i as u32 + 1;
        let id = texto(celda(columnas[0]));
        // Tal cual, sin recortar: ver el comentario del módulo.
        let nombre = match celda(columnas[1]) {
            Data::String(s) => s.clone(),
            otra => texto(otra),
        };
        if id.is_empty() && nombre.trim().is_empty() {
            // Fila vacía, o con algo solo en la nota: no es un alumno.
            continue;
        }
        if nombre.trim().is_empty() {
            sin_nombre.push(fila);
        }
        let nota = leer_nota(celda(columnas[2]));
        match &nota {
            Nota::Invalida(_) => invalidas.push(fila),
            Nota::Valor(_) if matches!(celda(columnas[2]), Data::String(s) if s.contains(',')) => con_coma.push(fila),
            _ => {}
        }
        alumnos.push(Alumno { fila, id, nombre, nota, clave: String::new() });
    }
    if alumnos.is_empty() {
        return Err(ErrorPlantilla::SinAlumnos);
    }

    if !con_coma.is_empty() {
        avisos.push(Aviso::NotasConComa { filas: con_coma });
    }
    if !invalidas.is_empty() {
        avisos.push(Aviso::NotasInvalidas { filas: invalidas });
    }
    if !sin_nombre.is_empty() {
        avisos.push(Aviso::SinNombre { filas: sin_nombre });
    }
    let repetidos = ids_repetidos(&alumnos);
    for a in &mut alumnos {
        let repetido = repetidos.iter().any(|r| matches!(r, Aviso::IdRepetido { id, .. } if *id == a.id));
        a.clave = if a.id.is_empty() || repetido { format!("fila:{}", a.fila) } else { a.id.clone() };
    }
    avisos.extend(repetidos);

    Ok(Plantilla { alumnos, columnas: columnas_leidas, avisos })
}

/// Busca, en las diez primeras filas, la que tiene una cabecera de nombre y
/// otra de nota. Devuelve la fila y las columnas `[id, nombre, nota]`.
///
/// El ID es la columna con cabecera de ID; si no la hay, la primera que no
/// sea ni el nombre ni la nota.
fn buscar_cabecera(filas: &[&[Data]]) -> Option<(usize, [usize; 3])> {
    for (i, f) in filas.iter().enumerate().take(10) {
        let cabeceras: Vec<String> = f.iter().map(|c| plegar(&texto(c))).collect();
        let buscar = |claves: &[&str]| cabeceras.iter().position(|c| claves.iter().any(|k| es_cabecera(c, k)));
        let (Some(nombre), Some(nota)) = (buscar(CABECERAS_NOMBRE), buscar(CABECERAS_NOTA)) else {
            continue;
        };
        let id = buscar(CABECERAS_ID)
            .filter(|&c| c != nombre && c != nota)
            .or_else(|| (0..cabeceras.len()).find(|&c| c != nombre && c != nota && !cabeceras[c].is_empty()))?;
        return Some((i, [id, nombre, nota]));
    }
    None
}

/// Las cabeceras reconocidas, ya plegadas (minúsculas, sin acentos). Se
/// comparan **por palabra**, no por subcadena: «Nombre del alumno» es un
/// nombre, pero «Anotación» no es una nota.
const CABECERAS_NOMBRE: &[&str] = &["alumno", "alumna", "alumne", "alumnos", "alumnes", "nombre", "nom", "estudiante"];
const CABECERAS_NOTA: &[&str] = &["nota", "notas", "notes", "calificacion", "qualificacio"];
const CABECERAS_ID: &[&str] = &["id", "codigo", "codi", "cod", "numero", "num"];

fn es_cabecera(cabecera: &str, clave: &str) -> bool {
    cabecera.split(|c: char| !c.is_alphanumeric()).any(|palabra| palabra == clave)
}

fn leer_nota(celda: &Data) -> Nota {
    let en_rango = |v: f64| (0.0..=10.0).contains(&v);
    match celda {
        Data::Empty => Nota::Vacia,
        Data::Int(i) if en_rango(*i as f64) => Nota::Valor(*i as f64),
        Data::Float(v) if en_rango(*v) => Nota::Valor(*v),
        Data::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                return Nota::Vacia;
            }
            match s.replace(',', ".").parse::<f64>() {
                Ok(v) if en_rango(v) && v.is_finite() => Nota::Valor(v),
                _ => Nota::Invalida(s.to_string()),
            }
        }
        otra => Nota::Invalida(texto(otra)),
    }
}

fn ids_repetidos(alumnos: &[Alumno]) -> Vec<Aviso> {
    let mut vistos: Vec<(&str, Vec<u32>)> = Vec::new();
    for a in alumnos.iter().filter(|a| !a.id.is_empty()) {
        match vistos.iter_mut().find(|(id, _)| *id == a.id) {
            Some((_, filas)) => filas.push(a.fila),
            None => vistos.push((&a.id, vec![a.fila])),
        }
    }
    vistos
        .into_iter()
        .filter(|(_, filas)| filas.len() > 1)
        .map(|(id, filas)| Aviso::IdRepetido { id: id.to_string(), filas })
        .collect()
}

/// Una celda como texto, sin espacios a los lados. Los enteros guardados como
/// número (`1234.0`) salen sin decimales: un ID no es una cifra.
fn texto(celda: &Data) -> String {
    match celda {
        Data::Empty => String::new(),
        Data::String(s) | Data::DateTimeIso(s) | Data::DurationIso(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", *f as i64),
        Data::Float(f) => f.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.to_string(),
        Data::Error(e) => format!("#{e:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// El nombre legible
// ─────────────────────────────────────────────────────────────────────────────

/// El nombre de la clase que se enseña, sacado del nombre del fichero (sin
/// extensión).
///
/// - `biologia_1_ab_bach` → «Biología 1.º AB Bachillerato»: el formato que
///   recomendaba la v1, `materia_curso_grupo_nivel`.
/// - `Matemáticas - 3r ESO A` → igual: es como las guarda la sincronización
///   con Clickedu, y ya es legible.
/// - Cualquier otro → los `_` como espacios y la primera en mayúscula.
///
/// El fichero se enseña siempre al lado, así que esto solo ha de leerse bien,
/// no ser exacto.
pub fn nombre_legible(sin_extension: &str) -> String {
    let partes: Vec<&str> = sin_extension.split('_').filter(|p| !p.is_empty()).collect();
    let es_curso = |p: &str| p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty();
    if !sin_extension.contains(' ') && partes.len() >= 2 && es_curso(partes[1]) {
        let mut res = vec![materia(partes[0]), format!("{}.º", partes[1])];
        if let Some(grupo) = partes.get(2) {
            res.push(grupo.to_uppercase());
        }
        if let Some(nivel) = partes.get(3) {
            let n = nivel.to_lowercase();
            res.push(if n.starts_with("bach") {
                "Bachillerato".to_string()
            } else {
                nivel.to_uppercase()
            });
        }
        for resto in partes.iter().skip(4) {
            res.push(resto.to_string());
        }
        return res.join(" ");
    }
    let espacios = sin_extension.replace('_', " ");
    let limpio = espacios.split_whitespace().collect::<Vec<_>>().join(" ");
    primera_mayuscula(&limpio)
}

/// La materia con sus tildes, si es una de las de siempre: el nombre de
/// fichero de la v1 no las llevaba. En camelCase (`fisicaQuimica`, como las
/// reales) se parte en palabras, y cada una con su tilde.
fn materia(p: &str) -> String {
    if let Some(entera) = con_tilde(p) {
        return entera;
    }
    let mut palabras: Vec<String> = vec![String::new()];
    for c in p.chars() {
        if c.is_uppercase() && !palabras.last().unwrap().is_empty() {
            palabras.push(String::new());
        }
        palabras.last_mut().unwrap().push(c);
    }
    palabras
        .iter()
        .map(|w| con_tilde(w).unwrap_or_else(|| primera_mayuscula(&w.to_lowercase())))
        .collect::<Vec<_>>()
        .join(" ")
}

fn con_tilde(p: &str) -> Option<String> {
    const CON_TILDE: &[(&str, &str)] = &[
        ("biologia", "Biología"),
        ("matematicas", "Matemáticas"),
        ("fisicaquimica", "Física y Química"),
        ("fisica", "Física"),
        ("quimica", "Química"),
        ("ingles", "Inglés"),
        ("frances", "Francés"),
        ("aleman", "Alemán"),
        ("tecnologia", "Tecnología"),
        ("musica", "Música"),
        ("geografia", "Geografía"),
        ("filosofia", "Filosofía"),
        ("economia", "Economía"),
        ("informatica", "Informática"),
        ("educacionfisica", "Educación Física"),
        ("religion", "Religión"),
        ("latin", "Latín"),
        ("griego", "Griego"),
        ("lengua", "Lengua"),
        ("historia", "Historia"),
        ("catalan", "Catalán"),
        ("valenciano", "Valenciano"),
        ("plastica", "Plástica"),
        ("dibujo", "Dibujo"),
    ];
    let llano = plegar(p);
    CON_TILDE.iter().find(|(k, _)| *k == llano).map(|(_, v)| v.to_string())
}

fn primera_mayuscula(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(p) => p.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Minúsculas y sin tildes, para comparar cabeceras y ordenar.
pub fn plegar(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            otra => otra,
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// La carpeta
// ─────────────────────────────────────────────────────────────────────────────

/// Una clase de la lista: el fichero, su nombre legible y cómo ha ido leerla.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clase {
    pub fichero: String,
    pub nombre: String,
    pub lectura: Lectura,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Lectura {
    Bien { alumnos: usize, calificados: usize, avisos: usize },
    Mal { error: String },
}

/// ¿Es un fichero de plantilla? `.xlsx` o `.xls`, y no el `~$…` que deja Excel
/// mientras tiene un libro abierto.
pub fn es_plantilla(nombre: &str) -> bool {
    let n = nombre.to_lowercase();
    !n.starts_with("~$") && (n.ends_with(".xlsx") || n.ends_with(".xls"))
}

/// Las clases de `carpeta`, ordenadas por nombre. Solo la carpeta, sin
/// subcarpetas: `notas/` puede vivir al lado y no son plantillas.
///
/// `calificados` cuenta los alumnos con nota de cada plantilla (fichero y
/// plantilla leída): la sesión (H2) tiene notas que el Excel no.
pub fn listar(carpeta: &Path, calificados: impl Fn(&str, &Plantilla) -> usize) -> std::io::Result<Vec<Clase>> {
    let mut clases = Vec::new();
    for entrada in std::fs::read_dir(carpeta)? {
        let entrada = entrada?;
        if !entrada.file_type()?.is_file() {
            continue;
        }
        let fichero = entrada.file_name().to_string_lossy().into_owned();
        if !es_plantilla(&fichero) {
            continue;
        }
        let sin_extension = Path::new(&fichero).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let lectura = match leer(&entrada.path()) {
            Ok(p) => Lectura::Bien { alumnos: p.alumnos.len(), calificados: calificados(&fichero, &p), avisos: p.avisos.len() },
            Err(e) => Lectura::Mal { error: e.to_string() },
        };
        clases.push(Clase { nombre: nombre_legible(&sin_extension), fichero, lectura });
    }
    clases.sort_by(|a, b| plegar(&a.nombre).cmp(&plegar(&b.nombre)).then_with(|| a.fichero.cmp(&b.fichero)));
    Ok(clases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombres_legibles() {
        assert_eq!(nombre_legible("biologia_1_ab_bach"), "Biología 1.º AB Bachillerato");
        assert_eq!(nombre_legible("matematicas_3_a_eso"), "Matemáticas 3.º A ESO");
        assert_eq!(nombre_legible("ingles_2_b_eso"), "Inglés 2.º B ESO");
        assert_eq!(nombre_legible("Fisica_2"), "Física 2.º");
        // Como las reales: camelCase.
        assert_eq!(nombre_legible("fisicaQuimica_1_bc_bach"), "Física y Química 1.º BC Bachillerato");
        assert_eq!(nombre_legible("cienciasSociales_2_a_eso"), "Ciencias Sociales 2.º A ESO");
        assert_eq!(nombre_legible("educacionFisica_4_ab_eso"), "Educación Física 4.º AB ESO");
        assert_eq!(nombre_legible("astronomia_4_c_eso"), "Astronomia 4.º C ESO");
        // La sincronización con Clickedu ya los deja legibles.
        assert_eq!(nombre_legible("Matemàtiques - 3r ESO A"), "Matemàtiques - 3r ESO A");
        assert_eq!(nombre_legible("notas_trimestre_1"), "Notas trimestre 1");
        assert_eq!(nombre_legible("Tutoria"), "Tutoria");
    }

    #[test]
    fn notas() {
        assert_eq!(leer_nota(&Data::Empty), Nota::Vacia);
        assert_eq!(leer_nota(&Data::Float(7.5)), Nota::Valor(7.5));
        assert_eq!(leer_nota(&Data::Int(10)), Nota::Valor(10.0));
        assert_eq!(leer_nota(&Data::String(" 6,25 ".into())), Nota::Valor(6.25));
        assert_eq!(leer_nota(&Data::String("  ".into())), Nota::Vacia);
        assert_eq!(leer_nota(&Data::Float(12.0)), Nota::Invalida("12".into()));
        assert_eq!(leer_nota(&Data::String("NP".into())), Nota::Invalida("NP".into()));
        assert_eq!(leer_nota(&Data::Float(-1.0)), Nota::Invalida("-1".into()));
        assert_eq!(leer_nota(&Data::String("NaN".into())), Nota::Invalida("NaN".into()));
    }

    #[test]
    fn cabeceras_por_palabra() {
        assert!(es_cabecera("nombre del alumno", "alumno"));
        assert!(es_cabecera("nota (0-10)", "nota"));
        assert!(!es_cabecera("anotacion", "nota"));
        assert!(!es_cabecera("nombres", "nombre"));
    }

    #[test]
    fn que_es_plantilla() {
        assert!(es_plantilla("biologia_1_ab_bach.xlsx"));
        assert!(es_plantilla("VIEJA.XLS"));
        assert!(!es_plantilla("~$biologia_1_ab_bach.xlsx"));
        assert!(!es_plantilla("notas.csv"));
        assert!(!es_plantilla("desktop.ini"));
    }
}
