//! Entender una frase dictada (H3b): de lo que oyó Whisper a **qué alumno y
//! qué nota**, o a la pregunta que hay que hacer.
//!
//! Junta el [intérprete](super::interprete) (la nota y las palabras del
//! nombre) y el [emparejamiento](super::emparejar) (el alumno). La regla es
//! la de la prueba de H3b: **ninguna nota mal puesta en silencio**. Lo que
//! no está claro se pregunta (borde discontinuo, D6) ofreciendo lo más
//! probable; lo que no se entiende, se dice.
//!
//! **Cuándo se pregunta por el alumno**, medido con las 100 grabaciones:
//!
//! - Si lo oído se parece poco al más parecido (menos de [`PARECIDO_SEGURO`]).
//! - Si **con cambiar dos sonidos sería otro alumno** ([`MARGEN`]). Es lo
//!   que pilla el único nombre mal oído que podía pasar en silencio: «Lucía
//!   Martín» oído como «Lucía Martínez», que también está en la clase.
//!   Diciendo el nombre entero («Lucía Martín Ortega») ya no pregunta.
//!
//! La confianza de Whisper en cada palabra **no sirve para esto**: en
//! «Martínez» mal oído era del 57 %, y en los nombres bien oídos va del 10 %
//! al 95 %. Whisper reparte la probabilidad entre maneras de escribir lo
//! mismo («10» o «diez», «9,5» o «9.5»), no entre cosas oídas.

use serde::Serialize;

use super::emparejar::emparejar;
use super::interprete::{interpretar, PorQueDudosa};

/// Por debajo, lo oído no se parece a nadie de la clase.
pub const PARECIDO_MINIMO: f32 = 0.5;
/// Por debajo, se parece, pero no tanto como para apuntarlo sin preguntar.
pub const PARECIDO_SEGURO: f32 = 0.75;
/// Si otro alumno queda a estos sonidos de más, o menos, se pregunta.
pub const MARGEN: usize = 2;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "tipo", rename_all = "camelCase")]
pub enum Entendido {
    /// Se apunta sin preguntar (verde, D6). `alumno`: su posición en la
    /// lista de la clase.
    Seguro { alumno: usize, nota: f64 },
    /// Se pregunta: los alumnos y las notas posibles, lo más probable
    /// primero, y por qué se duda.
    Duda { alumnos: Vec<usize>, notas: Vec<f64>, motivos: Vec<Motivo> },
    /// No se apunta nada, y se dice por qué.
    NoEntendido { motivo: Motivo },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Motivo {
    /// No se oyó ninguna nota (de 0 a 10).
    SinNota,
    /// Se oyó la nota y nada más.
    SinNombre,
    /// Lo oído no se parece a nadie de la clase.
    NombreDesconocido,
    /// Se parece al más parecido, pero no lo bastante.
    NombrePocoClaro,
    /// Otro alumno se llama casi igual.
    NombreParecido,
    /// Escrita de una forma que se puede leer de dos maneras («825»).
    NotaReinterpretada,
    /// Se oyeron dos notas distintas.
    VariasNotas,
    /// Además de la nota, un número que no lo es.
    NumeroDeMas,
}

/// `clase`: los nombres de los alumnos, en el orden de la lista.
pub fn entender<S: AsRef<str>>(texto: &str, clase: &[S]) -> Entendido {
    let frase = interpretar(texto);
    let Some(nota) = frase.nota else {
        return Entendido::NoEntendido { motivo: Motivo::SinNota };
    };
    if frase.nombre.is_empty() {
        return Entendido::NoEntendido { motivo: Motivo::SinNombre };
    }
    let palabras: Vec<&str> = frase.nombre.iter().map(String::as_str).collect();
    let parecidos = emparejar(&palabras, clase);
    let Some(mejor) = parecidos.first().filter(|p| p.parecido >= PARECIDO_MINIMO) else {
        return Entendido::NoEntendido { motivo: Motivo::NombreDesconocido };
    };

    let mut motivos = Vec::new();
    if mejor.parecido < PARECIDO_SEGURO {
        motivos.push(Motivo::NombrePocoClaro);
    }
    // El rival: el más cercano de los demás, si también se parece a lo
    // oído. En lo oído corto («quer», por Iker), dos sonidos son media
    // palabra y no hacen a nadie parecido.
    let rival = parecidos[1..].iter().filter(|p| p.parecido >= PARECIDO_MINIMO).min_by_key(|p| p.distancia);
    if rival.is_some_and(|r| r.distancia <= mejor.distancia + MARGEN) {
        motivos.push(Motivo::NombreParecido);
    }
    match nota.dudosa {
        Some(PorQueDudosa::Reinterpretada) => motivos.push(Motivo::NotaReinterpretada),
        Some(PorQueDudosa::Varias) => motivos.push(Motivo::VariasNotas),
        Some(PorQueDudosa::NumeroDeMas) => motivos.push(Motivo::NumeroDeMas),
        None => {}
    }

    if motivos.is_empty() {
        return Entendido::Seguro { alumno: mejor.alumno, nota: nota.valores[0] };
    }
    // Se ofrece el más parecido y su rival, si lo tiene.
    let alumnos = std::iter::once(mejor.alumno).chain(rival.map(|r| r.alumno)).collect();
    Entendido::Duda { alumnos, notas: nota.valores, motivos }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLASE: &[&str] = &[
        "Lucía Martín Ortega",
        "Lucía Martínez Soler",
        "Pablo García Ruiz",
        "Pablo Gracia Navarro",
        "Núria Vilaseca Roig",
        "Elena Gómez Serrano",
        "Wei Chen",
    ];

    fn de(texto: &str) -> Entendido {
        entender(texto, CLASE)
    }

    #[test]
    fn lo_claro_se_apunta() {
        assert_eq!(de("Elena Gómez siete con cinco"), Entendido::Seguro { alumno: 5, nota: 7.5 });
        assert_eq!(de("Lucía Martínez Soler 8,25"), Entendido::Seguro { alumno: 1, nota: 8.25 });
        assert_eq!(de("Weicheng Nueve"), Entendido::Seguro { alumno: 6, nota: 9.0 });
    }

    #[test]
    fn si_otro_se_llama_casi_igual_se_pregunta() {
        let Entendido::Duda { alumnos, notas, motivos } = de("Lucia Martínez, 9 y medio.") else { panic!() };
        assert_eq!((alumnos, notas, motivos), (vec![1, 0], vec![9.5], vec![Motivo::NombreParecido]));
        let Entendido::Duda { alumnos, .. } = de("Pablo García 10") else { panic!() };
        assert_eq!(alumnos, vec![2, 3]);
        let Entendido::Duda { alumnos, .. } = de("Lucía nueve") else { panic!() };
        assert_eq!(alumnos, vec![0, 1]);
    }

    #[test]
    fn la_nota_reinterpretada_se_pregunta() {
        let Entendido::Duda { alumnos, notas, motivos } = de("Elena Gómez 825") else { panic!() };
        assert_eq!(alumnos[0], 5);
        assert_eq!((notas, motivos), (vec![8.25], vec![Motivo::NotaReinterpretada]));
    }

    #[test]
    fn lo_que_no_se_entiende_se_dice() {
        assert_eq!(de("Way this."), Entendido::NoEntendido { motivo: Motivo::SinNota });
        assert_eq!(de("un siete"), Entendido::NoEntendido { motivo: Motivo::SinNombre });
        assert_eq!(de("Zacarías Fuster 7"), Entendido::NoEntendido { motivo: Motivo::NombreDesconocido });
    }
}
