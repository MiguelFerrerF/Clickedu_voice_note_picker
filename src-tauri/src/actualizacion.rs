//! Lo del actualizador que se prueba sin red (H7): leer las notas de una
//! versión nueva.
//!
//! El cuerpo del release lo compone `scripts/notas-de-version.mjs` con
//! `docs/releases/vX.Y.Z.md`: el Markdown para la página de GitHub y, detrás,
//! un bloque comentado que lee la app sin una segunda petición (como en
//! library-app, `docs/08-actualizaciones.md` §4):
//!
//! ```text
//! <!--RELEASE_DATA
//! {"highlights":"…","entries":[{"type":"feat","text":"…"}]}
//! -->
//! ```
//!
//! `tauri-action` lo copia en las `notes` de `latest.json`, que es lo que
//! devuelve el actualizador. Sin el bloque, o roto, las notas salen vacías y
//! no se rompe nada.

use serde::{Deserialize, Serialize};

/// Una línea de «Cambios»: `feat`, `fix`, `perf` o `chore`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entrada {
    #[serde(rename(deserialize = "type"))]
    pub tipo: String,
    #[serde(rename(deserialize = "text"))]
    pub texto: String,
}

/// El titular y los cambios de una versión.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Notas {
    pub titular: String,
    pub entradas: Vec<Entrada>,
}

#[derive(Deserialize)]
struct Bloque {
    #[serde(default)]
    highlights: String,
    #[serde(default)]
    entries: Vec<Entrada>,
}

const MARCA: &str = "<!--RELEASE_DATA";

/// Las notas de `cuerpo`. GitHub guarda los cuerpos con `\r\n`: se quitan
/// antes de buscar la marca.
pub fn leer_notas(cuerpo: &str) -> Notas {
    let cuerpo = cuerpo.replace("\r\n", "\n");
    let Some(empieza) = cuerpo.find(MARCA) else { return Notas::default() };
    let resto = &cuerpo[empieza + MARCA.len()..];
    let Some(acaba) = resto.find("-->") else { return Notas::default() };
    match serde_json::from_str::<Bloque>(resto[..acaba].trim()) {
        Ok(b) => Notas { titular: b.highlights.trim().to_string(), entradas: b.entries },
        Err(_) => Notas::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_el_bloque_detras_del_markdown() {
        let cuerpo = "## Qué cambia\r\n\r\nTexto.\r\n\r\n<!--RELEASE_DATA\r\n\
            {\"highlights\":\"La v2\",\"entries\":[{\"type\":\"feat\",\"text\":\"Dictar\"},\
            {\"type\":\"fix\",\"text\":\"La mediana\"}]}\r\n-->\r\n";
        let n = leer_notas(cuerpo);
        assert_eq!(n.titular, "La v2");
        assert_eq!(
            n.entradas,
            vec![
                Entrada { tipo: "feat".into(), texto: "Dictar".into() },
                Entrada { tipo: "fix".into(), texto: "La mediana".into() },
            ]
        );
    }

    #[test]
    fn sin_bloque_o_roto_salen_vacias() {
        assert_eq!(leer_notas("Solo Markdown"), Notas::default());
        assert_eq!(leer_notas("<!--RELEASE_DATA\n{roto\n-->"), Notas::default());
        assert_eq!(leer_notas("<!--RELEASE_DATA\n{\"highlights\":\"x\"}"), Notas::default());
    }

    #[test]
    fn sin_entradas_queda_el_titular() {
        let n = leer_notas("<!--RELEASE_DATA\n{\"highlights\":\" Solo esto \"}\n-->");
        assert_eq!(n.titular, "Solo esto");
        assert!(n.entradas.is_empty());
    }
}
