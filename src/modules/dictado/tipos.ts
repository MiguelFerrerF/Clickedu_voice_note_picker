/**
 * Lo que devuelve `entender_frase` (H3b) y lo que se le contesta
 * (`responder_dictado`, D8): el espejo de `registro::Dictado`,
 * `voz::entender::Entendido` y `Motivo`, y `registro::Respuesta`. Si cambia
 * allí, cambia aquí.
 */

/** Por qué no se apunta nada. */
export type MotivoNoEntendido = 'sin_nota' | 'nota_rara' | 'sin_nombre' | 'nombre_desconocido';

/** Por qué se pregunta. */
export type MotivoDuda =
  'nombre_poco_claro' | 'nombre_parecido' | 'nota_reinterpretada' | 'varias_notas' | 'numero_de_mas';

/**
 * Una frase dictada, entendida. Cada alumno va por su **clave** de la
 * sesión, no por su posición. `id`: con el que se contesta, para el
 * registro. `oido`: lo que se entendió, tal cual.
 */
export type Dictado =
  | {id: string; oido: string; tipo: 'seguro'; alumno: string; nota: number}
  | {id: string; oido: string; tipo: 'duda'; alumnos: string[]; notas: number[]; motivos: MotivoDuda[]}
  | {id: string; oido: string; tipo: 'no_entendido'; motivo: MotivoNoEntendido};

/** Lo que hizo el profesor con un dictado: la respuesta buena, para el registro. */
export type Respuesta =
  | {tipo: 'elegida'; clave: string; nota: number}
  | {tipo: 'descartada'}
  | {tipo: 'deshecha'}
  | {tipo: 'corregida'; nota: number | null};

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
  return d.alumnos.flatMap((clave) => d.notas.map((nota) => ({clave, nota}))).slice(0, 4);
}
