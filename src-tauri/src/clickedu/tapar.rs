//! Pasar lo de Clickedu a su **forma, sin nombres** (H4): textos,
//! direcciones y trozos de código. Lo usan el explorador
//! (`examples/explorar_clickedu`), el que anonimiza las páginas de los
//! tests (`examples/anonimizar_clickedu`) y la huella de cada página.
//!
//! La regla es **la lista blanca**: de un texto solo quedan los rótulos de
//! la interfaz ([`LETREROS`]), las palabras que no dicen nada de nadie
//! ([`CORTAS`]), las de una o dos letras y los números cortos; lo demás sale
//! como `▢`. Los nombres de las plantillas, siempre `▢`, aunque fueran un
//! rótulo.

use std::collections::HashSet;

use reqwest::Url;

/// Los parámetros que llevan el verbo de lo que hace la página.
#[rustfmt::skip]
pub const PARAMETROS_DE_VERBO: &[&str] = &[
    "do", "accio", "acció", "action", "accion", "acc", "op", "opcio", "opcion", "operacio", "operacion", "task",
    "cmd", "mode", "modo",
];

/// Los parámetros cuyo valor es código (`ctr=Subjects/Subjects`), no un
/// texto: se enseña tal cual (con los números tapados).
#[rustfmt::skip]
pub const PARAMETROS_DE_CODIGO: &[&str] = &[
    "ctr", "tab", "seccio", "seccion", "section", "view", "vista", "tipus", "tipo", "type", "lang", "idioma",
    "menu", "modul", "modulo", "module", "pag", "page",
];

/// Palabras que no dicen nada de nadie: se dejan siempre.
#[rustfmt::skip]
pub const CORTAS: &[&str] = &[
    "de", "del", "la", "las", "los", "el", "y", "i", "o", "u", "a", "al", "en", "per", "por", "para", "amb", "con",
    "sin", "sense", "les", "els", "lo", "un", "una", "uns", "unes", "que", "es", "és", "se", "su", "sus", "seu",
    "seus", "teu", "tu", "mi", "meu", "mis", "meus", "le", "d", "l", "s", "n", "the", "of", "and", "to", "in", "on",
    "for", "by", "at", "or", "with", "from", "your", "lar",
];

/// Los rótulos de la interfaz que se dejan en el informe (en minúscula; se
/// comparan sin mayúsculas). **Ningún nombre de persona** (ni «rosa»,
/// «pilar», «abril», «domingo»…): lo que no está aquí sale como `▢`.
#[rustfmt::skip]
pub const LETREROS: &[&str] = &[
    // Castellano
    "inicio", "asignatura", "asignaturas", "materia", "materias", "área", "áreas", "area", "areas", "evaluación",
    "evaluaciones", "evaluacion", "evaluar", "nota", "notas", "calificación", "calificaciones", "calificacion",
    "curso", "cursos", "grupo", "grupos", "clase", "clases", "alumno", "alumnos", "alumna", "alumnas", "alumnado",
    "nombre", "apellidos", "apellido", "profesor", "profesora", "profesores", "profesorado", "tutor", "tutora",
    "tutoría", "tutoria", "familia", "familias", "padres", "ítem", "ítems", "item", "items", "criterio",
    "criterios", "competencia", "competencias", "estándar", "estándares", "indicador", "indicadores", "media",
    "medias", "peso", "pesos", "porcentaje", "ponderación", "trimestre", "trimestral", "primera", "primer",
    "primero", "segunda", "segundo", "tercera", "tercer", "tercero", "final", "finales", "ordinaria",
    "extraordinaria", "global", "parcial", "recuperación", "observaciones", "observación", "comentario",
    "comentarios", "fecha", "fechas", "hora", "horas", "día", "días", "semana", "mes", "año", "curso", "escolar",
    "académico", "académica", "horario", "horarios", "calendario", "agenda", "mensajes", "mensaje",
    "comunicaciones", "comunicación", "circulares", "noticias", "avisos", "aviso", "faltas", "falta", "asistencia",
    "retrasos", "incidencias", "incidencia", "expediente", "expedientes", "boletín", "boletines", "informe",
    "informes", "actas", "acta", "documentos", "documento", "ficheros", "fichero", "archivo", "archivos",
    "plantilla", "plantillas", "importar", "exportar", "descargar", "subir", "guardar", "editar", "borrar",
    "eliminar", "ver", "listado", "lista", "buscar", "filtrar", "todos", "todas", "ninguno", "sí", "si", "no",
    "configuración", "perfil", "usuario", "contraseña", "ayuda", "cerrar", "salir", "enviar", "aceptar", "cancelar",
    "volver", "anterior", "siguiente", "nuevo", "nueva", "añadir", "total", "totales", "resumen", "estadísticas",
    "actividades", "actividad", "tareas", "tarea", "exámenes", "examen", "recursos", "seguimiento", "programación",
    "programaciones", "unidad", "unidades", "didáctica", "didácticas", "aula", "aulas", "etapa", "nivel", "niveles",
    "secundaria", "primaria", "infantil", "bachillerato", "ciclo", "formativo", "eso", "fp", "matemáticas",
    "lengua", "castellana", "literatura", "inglés", "francés", "alemán", "euskera", "física", "química", "biología",
    "geología", "historia", "geografía", "tecnología", "música", "plástica", "visual", "audiovisual", "educación",
    "religión", "valores", "éticos", "ética", "latín", "griego", "filosofía", "economía", "ciencias", "sociales",
    "naturales", "dibujo", "técnico", "informática", "digitalización", "robótica", "proyecto", "proyectos",
    "optativa", "optativas", "deporte", "cultura", "científica", "clásica", "lunes", "martes", "miércoles",
    "jueves", "viernes", "sábado", "enero", "febrero", "marzo", "mayo", "junio", "julio", "agosto", "septiembre",
    "octubre", "noviembre", "diciembre", "pendiente", "pendientes", "aprobado", "suspenso", "notable",
    "sobresaliente", "suficiente", "insuficiente", "bien", "nueva", "mostrar", "ocultar", "seleccionar",
    "selecciona", "elegir", "opciones", "opción", "acciones", "acción", "datos", "dato", "personales", "centro",
    "colegio", "error", "correcto", "incorrecto", "sesión", "entrar", "acceso", "seguridad", "importación",
    "exportación", "cargar", "procesar", "estado", "tipo", "descripción", "título", "código", "valor", "valores",
    "número", "orden", "posición", "imprimir", "pdf", "excel", "hoja", "cálculo",
    // Català
    "inici", "assignatura", "assignatures", "matèria", "matèries", "àrea", "àrees", "avaluació", "avaluacions",
    "avaluar", "notes", "qualificació", "qualificacions", "curs", "grup", "grups", "classe", "classes", "alumne",
    "alumnes", "nom", "cognoms", "cognom", "professor", "professora", "professors", "professorat", "tutoria",
    "famílies", "família", "pares", "criteri", "criteris", "competència", "competències", "mitjana", "mitjanes",
    "pes", "percentatge", "ponderació", "segon", "segona", "tercera", "ordinària", "extraordinària", "recuperació",
    "observacions", "observació", "comentari", "comentaris", "data", "dates", "dia", "dies", "setmana", "any",
    "acadèmic", "acadèmica", "horari", "calendari", "missatges", "missatge", "comunicacions", "comunicació",
    "notícies", "avisos", "faltes", "assistència", "retards", "incidències", "incidència", "expedient", "butlletí",
    "butlletins", "informes", "documents", "fitxers", "fitxer", "arxiu", "arxius", "descarregar", "pujar", "desar",
    "esborrar", "veure", "llistat", "llista", "cercar", "tots", "totes", "cap", "configuració", "usuari",
    "contrasenya", "ajuda", "tancar", "sortir", "acceptar", "cancel", "tornar", "següent", "nou", "nova", "afegir",
    "resum", "estadístiques", "activitats", "activitat", "tasques", "tasca", "exàmens", "examen", "recursos",
    "seguiment", "programació", "programacions", "unitat", "unitats", "didàctica", "aula", "etapa", "nivell",
    "nivells", "secundària", "primària", "batxillerat", "cicle", "formatiu", "matemàtiques", "llengua",
    "castellana", "literatura", "anglès", "francès", "alemany", "física", "química", "biologia", "geologia",
    "història", "geografia", "tecnologia", "música", "plàstica", "educació", "religió", "ètica", "llatí", "grec",
    "filosofia", "economia", "ciències", "socials", "naturals", "dibuix", "tècnic", "informàtica", "digitalització",
    "robòtica", "projecte", "projectes", "optativa", "dilluns", "dimarts", "dimecres", "dijous", "divendres",
    "dissabte", "gener", "febrer", "març", "maig", "juny", "juliol", "agost", "setembre", "novembre", "desembre",
    "pendent", "pendents", "aprovat", "suspès", "excel·lent", "excel", "lent", "suficient", "insuficient", "bé",
    "mostra", "amagar", "seleccionar", "selecciona", "triar", "opcions", "opció", "accions", "acció", "dades",
    "personals", "centre", "escola", "correcte", "incorrecte", "sessió", "entrar", "accés", "seguretat",
    "importació", "exportació", "carregar", "processar", "estat", "tipus", "descripció", "títol", "codi", "valor",
    "valors", "número", "ordre", "posició", "imprimir", "full", "càlcul", "trimestre",
    // English
    "home", "subject", "subjects", "items", "student", "students", "grade", "grades", "mark", "marks", "evaluation",
    "evaluations", "term", "terms", "settings", "search", "calendar", "messages", "help", "name", "surname",
    "group", "course", "average", "comments", "comment", "delete", "import", "export", "download", "upload", "file",
    "files", "template", "view", "show", "back", "next", "previous", "yes", "cancel", "accept", "submit", "index",
    "list", "detail", "details", "user", "users", "teacher", "teachers", "class", "level", "date", "type", "status",
    "description", "title", "code", "value", "loading", "select", "all", "none", "menu", "logout", "login",
    "password", "error", "ok", "close", "open", "new", "add", "edit", "save", "send", "print",
];

/// Pasa textos y direcciones a su forma, sin nombres.
pub struct Tapar {
    nombres: HashSet<String>,
    letreros: HashSet<&'static str>,
}

impl Tapar {
    /// Con los nombres que se tapan siempre (los de las plantillas).
    pub fn new<S: AsRef<str>>(nombres: impl IntoIterator<Item = S>) -> Self {
        let mut t = Tapar { nombres: HashSet::new(), letreros: LETREROS.iter().copied().collect() };
        for n in nombres {
            t.anadir(n.as_ref());
        }
        t
    }

    /// Cuántas palabras de nombres se tapan siempre.
    pub fn cuantos(&self) -> usize {
        self.nombres.len()
    }

    /// Si `palabra` es de un nombre de los que se tapan siempre.
    pub fn es_nombre(&self, palabra: &str) -> bool {
        self.nombres.contains(&clave(palabra))
    }

    /// Añade las palabras de `texto` a los nombres que se tapan siempre.
    pub fn anadir(&mut self, texto: &str) {
        for (palabra, letras) in trozos(texto) {
            if letras && !CORTAS.contains(&palabra.to_lowercase().as_str()) {
                self.nombres.insert(clave(palabra));
            }
        }
    }

    /// Un texto de la página: se quedan las palabras de [`LETREROS`] y
    /// [`CORTAS`], las de una o dos letras y los números de hasta cuatro
    /// cifras (años, notas). Lo demás, `▢`; un nombre de las plantillas,
    /// siempre `▢`.
    pub fn texto(&self, texto: &str) -> String {
        recortar(&self.texto_entero(texto), 80)
    }

    /// Como [`Tapar::texto`], sin recortar.
    pub fn texto_entero(&self, texto: &str) -> String {
        let junto = texto.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut forma = String::new();
        for (trozo, letras) in trozos(&junto) {
            let minus = trozo.to_lowercase();
            let numero = trozo.chars().all(|c| c.is_ascii_digit());
            let queda = if letras {
                CORTAS.contains(&minus.as_str())
                    || (!self.nombres.contains(&clave(trozo))
                        && (trozo.chars().count() <= 2 || self.letreros.contains(minus.as_str())))
            } else if numero {
                trozo.len() <= 4
            } else {
                // Lo que no es letra ni número: signos y espacios.
                !trozo.chars().any(char::is_alphanumeric)
            };
            forma.push_str(if queda { trozo } else { "▢" });
        }
        forma
    }

    /// Un trozo de código (una ruta, un nombre de campo, una clase CSS): se
    /// queda todo menos los números (`▢`) y un nombre de las plantillas.
    pub fn codigo(&self, codigo: &str) -> String {
        trozos(codigo)
            .into_iter()
            .map(|(t, letras)| {
                if t.chars().all(|c| c.is_ascii_digit()) || (letras && self.nombres.contains(&clave(t))) {
                    "▢".to_string()
                } else if !letras && t.chars().any(|c| c.is_alphanumeric()) {
                    // Letras y cifras mezcladas que no son una palabra.
                    "▢".to_string()
                } else {
                    t.to_string()
                }
            })
            .collect()
    }

    /// Una dirección de Clickedu: la ruta como código; los parámetros, con
    /// su nombre; el valor, como código si es de verbo o de código y como
    /// texto si no (`id=▢`, `nom=▢`). `exacta` deja los números con sus
    /// cifras (`id=▢5`); si no, `▢`: es la forma con que se agrupan.
    pub fn url(&self, url: &Url, exacta: bool) -> String {
        let ruta = url.path();
        // Un fichero subido (`/private/…/upload/…/Nombre_Apellido.pdf`) se
        // llama como quiera quien lo subió: de él, solo la extensión.
        let mut forma = match ruta.rsplit_once('/') {
            Some((carpeta, fichero)) if ruta.contains("/upload/") => {
                let ext = fichero.rsplit_once('.').map(|(_, e)| format!(".{e}")).unwrap_or_default();
                format!("{}/▢{}", self.codigo(carpeta), self.codigo(&ext))
            }
            _ => self.codigo(ruta),
        };
        let pares: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
        for (i, (k, v)) in pares.iter().enumerate() {
            forma.push(if i == 0 { '?' } else { '&' });
            forma.push_str(&self.codigo(k));
            forma.push('=');
            let kl = k.to_lowercase();
            let valor = if v.is_empty() {
                String::new()
            } else if v.chars().all(|c| c.is_ascii_digit()) {
                if exacta {
                    format!("▢{}", v.len())
                } else {
                    "▢".into()
                }
            } else if v.chars().any(|c| c.is_ascii_digit())
                && v.chars().all(|c| c.is_ascii_digit() || "/-_:. ".contains(c))
            {
                // Una fecha (`2027/03/15_10:00`): cada una sería una forma nueva.
                "▢fecha".into()
            } else if PARAMETROS_DE_VERBO.contains(&kl.as_str()) || PARAMETROS_DE_CODIGO.contains(&kl.as_str()) {
                self.codigo(v)
            } else {
                self.texto(v)
            };
            forma.push_str(&valor);
        }
        forma
    }
}

/// Cómo se compara una palabra con los nombres: en minúscula y sin tildes
/// (`lucia` en una dirección es `Lucía` en la plantilla).
pub fn clave(palabra: &str) -> String {
    palabra
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            otra => otra,
        })
        .collect()
}

/// Parte un texto en trozos de letras, de cifras y de lo demás, y dice cuáles
/// son de letras. `CamelCase` se parte por las mayúsculas: `doLogin` →
/// `do`, `Login`.
pub fn trozos(texto: &str) -> Vec<(&str, bool)> {
    #[derive(PartialEq, Clone, Copy)]
    enum Clase {
        Letra,
        Cifra,
        Otro,
    }
    let clase = |c: char| {
        if c.is_alphabetic() {
            Clase::Letra
        } else if c.is_ascii_digit() {
            Clase::Cifra
        } else {
            Clase::Otro
        }
    };
    let mut res = Vec::new();
    let mut inicio = 0;
    let mut anterior: Option<(Clase, char)> = None;
    for (i, c) in texto.char_indices() {
        let k = clase(c);
        if let Some((ka, ca)) = anterior {
            let corta = ka != k || (k == Clase::Letra && ca.is_lowercase() && c.is_uppercase());
            if corta {
                res.push((&texto[inicio..i], ka == Clase::Letra));
                inicio = i;
            }
        }
        anterior = Some((k, c));
    }
    if let Some((k, _)) = anterior {
        res.push((&texto[inicio..], k == Clase::Letra));
    }
    res
}

pub fn recortar(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://colegio.clickedu.eu").unwrap()
    }

    #[test]
    fn tapa_los_nombres() {
        let t = Tapar::new(["GARCÍA LÓPEZ, Lucía", "Rosa de la Fuente"]);
        assert_eq!(t.texto("Notas de Lucía García"), "Notas de ▢ ▢");
        assert_eq!(t.texto("Matemáticas 3ºA - 1a Avaluació 2025-2026"), "Matemáticas 3ºA - 1a Avaluació 2025-2026");
        // Un nombre que no está en las plantillas también se tapa.
        assert_eq!(t.texto("Hola, Pablo Sanz"), "▢, ▢ ▢");
        // «Rosa» es un nombre de la plantilla: tapado aunque fuera letrero.
        assert_eq!(t.texto("rosa"), "▢");
        // Una letra sola no dice de quién es.
        assert_eq!(t.texto("DNI 12345678Z, tel 948123456"), "▢ ▢Z, ▢ ▢");
        assert_eq!(t.texto("psanz@colegio.es"), "▢@colegio.es");
        assert_eq!(t.texto("LUCIA GARCIA"), "▢ ▢");
        assert!(t.es_nombre("lucia") && !t.es_nombre("notas"));
    }

    #[test]
    fn la_forma_de_una_direccion() {
        let t = Tapar::new(["Lucía García"]);
        let u = base().join("/go/Index.php?ctr=Subject/Items&do=Index&id=1234&nom=Lucia&trm=").unwrap();
        assert_eq!(t.url(&u, true), "/go/Index.php?ctr=Subject/Items&do=Index&id=▢4&nom=▢&trm=");
        assert_eq!(t.url(&u, false), "/go/Index.php?ctr=Subject/Items&do=Index&id=▢&nom=▢&trm=");
        let u = base().join("/alumnes/fitxa_lucia_3.php?accio=veure").unwrap();
        assert_eq!(t.url(&u, false), "/alumnes/fitxa_▢_▢.php?accio=veure");
        let u = base().join("/private/colegio/upload/noticies/arxius/12_Juan_Perez_trabajo.pdf").unwrap();
        assert_eq!(t.url(&u, false), "/private/colegio/upload/noticies/arxius/▢.pdf");
        let u = base().join("/assignatures/sessio.php?assig=12&data_hora_sessio=2027/03/15_10:00").unwrap();
        assert_eq!(t.url(&u, false), "/assignatures/sessio.php?assig=▢&data_hora_sessio=▢fecha");
    }

    #[test]
    fn trozos_parte_camel_case() {
        assert_eq!(trozos("doLogin"), [("do", true), ("Login", true)]);
        assert_eq!(trozos("ab12_c"), [("ab", true), ("12", false), ("_", false), ("c", true)]);
    }
}
