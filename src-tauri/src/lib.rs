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
//! - `registro` (D8) — lo no entendido, lo corregido y los fallos, en
//!   local, y el informe sin nombres que se puede mandar.
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
pub mod registro;
pub mod sesion;
pub mod vigilante;
pub mod voz;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(vigilante::Vigilante::default())
        .manage(sesion::Cerrojo::default())
        .manage(std::sync::Arc::new(commands::voz::Voz::default()))
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
            commands::voz::hay_modelo,
            commands::voz::descargar_modelo,
            commands::voz::abrir_microfono,
            commands::voz::cerrar_microfono,
            commands::voz::empezar_dictado,
            commands::voz::cancelar_dictado,
            commands::voz::terminar_dictado,
            commands::registro::responder_dictado,
            commands::registro::resumen_registro,
            commands::registro::preparar_informe,
            commands::registro::mostrar_informe,
            commands::registro::borrar_registro,
        ])
        .setup(|app| {
            // El registro (D8), lo primero: así se apunta cualquier fallo,
            // también un pánico.
            let registro = std::sync::Arc::new(registro::Registro::new(commands::carpeta_registro(app.handle())?));
            registro.purgar();
            apuntar_panicos(registro.clone());
            app.manage(registro);

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

/// Un pánico, al registro, y luego lo de siempre (la consola).
fn apuntar_panicos(registro: std::sync::Arc<registro::Registro>) {
    let antes = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        registro.apuntar(registro::Que::Fallo { donde: "pánico".into(), error: info.to_string() });
        antes(info);
    }));
}
