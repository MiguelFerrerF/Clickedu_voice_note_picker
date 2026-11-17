// Evita que se abra una consola extra en Windows al compilar en release.
// NO QUITAR.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    gestor_de_notas_lib::run();
}
