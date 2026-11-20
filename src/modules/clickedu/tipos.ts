/**
 * Lo que devuelven los comandos de Clickedu (`src-tauri/src/commands/clickedu.rs`).
 */

/** La cuenta guardada, **sin la contraseña**. */
export interface CuentaClickedu {
  usuario: string;
  /** El nombre del archivo de seguridad; `null` si falta. */
  archivo: string | null;
  /** Cuándo se eligió el archivo, en segundos desde 1970. */
  elegido: number | null;
}

/** Por qué no se ha podido entrar (`clickedu::cliente::Fallo`). */
export type FalloClickedu =
  | {tipo: 'sin_conexion'; detalle: string}
  | {tipo: 'servidor'; codigo: number}
  | {tipo: 'usuario'}
  | {tipo: 'archivo'}
  | {tipo: 'no_entiendo'; paso: string}
  | {tipo: 'otro'; detalle: string};
