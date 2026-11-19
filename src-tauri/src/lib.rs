//! Gestor de Notas — backend Tauri 2.
//!
//! Los módulos llegan con el hito que los necesita
//! (`docs/01-plan-de-trabajo.md`):
//!
//! - `plantillas` (H1) — leer las plantillas Excel de Clickedu y nombrarlas.
//! - `ajustes` (H1) — `ajustes.json`: dónde están las plantillas.
//! - `vigilante` (H1) — si cambia la carpeta de plantillas, avisar.
//! - `sesion` (H2) — las notas puestas, guardadas al momento.
//! - `exportar` (H2) — el Excel que se sube a Clickedu.
//! - `voz` (H3) — del micrófono al texto, con Whisper local.
//! - `commands` — lo que la interfaz puede llamar.
//!
//! Llegará `clickedu` (H4–H6).
//!
//! Regla heredada de library-app: **ningún comando síncrono hace trabajo**.
//! Todos `async`, y lo largo con progreso real hacia la interfaz.

pub mod ajustes;
pub mod commands;
pub mod exportar;
pub mod plantillas;
pub mod sesion;
pub mod vigilante;
pub mod voz;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(vigilante::Vigilante::default())
        .manage(sesion::Cerrojo::default())
        .invoke_handler(tauri::generate_handler![
            commands::clases::listar_clases,
            commands::clases::leer_clase,
            commands::clases::elegir_carpeta_plantillas,
            commands::clases::restablecer_carpeta_plantillas,
            commands::clases::abrir_carpeta_plantillas,
            commands::notas::poner_nota,
            commands::notas::devolver_nota,
            commands::notas::exportar_clase,
            commands::notas::abrir_carpeta_notas,
            commands::voz::entender_frase,
        ])
        .setup(|app| {
            // Vigilar la carpeta desde el arranque. Si los ajustes no se
            // pueden leer, no se vigila: la pantalla de Clases dirá por qué.
            if let Ok(c) = commands::carpeta(app.handle()) {
                commands::clases::empezar_a_vigilar(app.handle(), &c);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error al arrancar la aplicación");
}
