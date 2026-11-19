/**
 * Lo que devuelve `entender_frase` (H3b): el espejo de
 * `src-tauri/src/commands/voz.rs` y de `voz::entender::Motivo`. Si cambia
 * allí, cambia aquí.
 */

/** Por qué no se apunta nada. */
export type MotivoNoEntendido = 'sin_nota' | 'sin_nombre' | 'nombre_desconocido';

/** Por qué se pregunta. */
export type MotivoDuda =
  'nombre_poco_claro' | 'nombre_parecido' | 'nota_reinterpretada' | 'varias_notas' | 'numero_de_mas';

/**
 * Una frase dictada, entendida. Cada alumno va por su **clave** (la de la
 * sesión), no por su posición. `oido`: lo que se entendió, tal cual.
 */
export type Dictado =
  | {tipo: 'seguro'; oido: string; clave: string; nota: number}
  | {tipo: 'duda'; oido: string; claves: string[]; notas: number[]; motivos: MotivoDuda[]}
  | {tipo: 'no_entendido'; oido: string; motivo: MotivoNoEntendido};

/**
 * Lo último que ha pasado con el dictado en una clase, para enseñarlo en el
 * panel. `apuntada` es la nota que se puso (sin preguntar, o al elegir), con
 * la que había antes para poder deshacerla: `undefined` si el alumno no se
 * había tocado en la sesión, `null` si se había quitado.
 */
export interface UltimoDictado {
  fichero: string;
  dictado: Dictado;
  apuntada?: {clave: string; nota: number; antes: number | null | undefined};
}

/** Las opciones de una duda: cada alumno con cada nota, lo más probable primero. */
export function opciones(d: Extract<Dictado, {tipo: 'duda'}>): {clave: string; nota: number}[] {
  return d.claves.flatMap((clave) => d.notas.map((nota) => ({clave, nota}))).slice(0, 4);
}
