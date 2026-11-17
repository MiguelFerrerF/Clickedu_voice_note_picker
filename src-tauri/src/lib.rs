//! Gestor de Notas — backend Tauri 2.
//!
//! En H0 no hay comandos: la app abre, tiene el aspecto definitivo y no hace
//! nada. Los módulos llegan con el hito que los necesite
//! (`docs/01-plan-de-trabajo.md`):
//!
//! - `plantillas` (H1) — leer las plantillas Excel de Clickedu.
//! - `exportar` (H2) — escribir el Excel con las notas.
//! - `voz` (H3) — grabar, reconocer y entender la frase.
//! - `clickedu` (H4–H6) — sesión, plantillas y subida de notas.
//!
//! Regla heredada de library-app: **ningún comando síncrono hace trabajo**.
//! Todos `async`, y lo largo con progreso real hacia la interfaz.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error al arrancar la aplicación");
}
