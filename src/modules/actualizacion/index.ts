/**
 * Las actualizaciones (H7): buscar, descargar e instalar.
 *
 * Lo hace todo Rust (`commands/actualizacion.rs`) con el actualizador oficial
 * de Tauri, que comprueba la firma: aquí solo se llama y se tipa. La interfaz
 * no tiene ningún permiso del actualizador.
 */

import {Channel, invoke} from '@tauri-apps/api/core';

/** `actualizacion::Entrada`: una línea de «Cambios». */
export interface Entrada {
  tipo: 'feat' | 'fix' | 'perf' | 'chore' | string;
  texto: string;
}

/** `commands::actualizacion::VersionNueva`. */
export interface VersionNueva {
  version: string;
  actual: string;
  /** `AAAA-MM-DD`, o vacía. */
  publicada: string;
  titular: string;
  entradas: Entrada[];
}

/** En bytes; `total` 0 = no se sabe. */
export interface AvanceDescarga {
  hecho: number;
  total: number;
}

/** La versión nueva, o `null` si está al día. Sin conexión, un error. */
export function buscarActualizacion(): Promise<VersionNueva | null> {
  return invoke<VersionNueva | null>('buscar_actualizacion');
}

/** Descarga la versión encontrada y comprueba su firma. */
export function descargarActualizacion(alAvanzar: (a: AvanceDescarga) => void): Promise<void> {
  const avance = new Channel<AvanceDescarga>();
  avance.onmessage = alAvanzar;
  return invoke<void>('descargar_actualizacion', {avance});
}

/** Instala lo descargado. Si sale bien **la app se cierra** y vuelve a abrirse. */
export function instalarActualizacion(): Promise<void> {
  return invoke<void>('instalar_actualizacion');
}

const CLAVE_SALTADA = 'gestornotas-version-saltada';

/** La versión que se pidió saltar al arrancar. Solo es una comodidad. */
export function versionSaltada(): string | null {
  try {
    return localStorage.getItem(CLAVE_SALTADA);
  } catch {
    return null;
  }
}

export function saltarVersion(version: string): void {
  try {
    localStorage.setItem(CLAVE_SALTADA, version);
  } catch {
    /* sin almacenamiento, volverá a ofrecerse: no pasa nada */
  }
}
