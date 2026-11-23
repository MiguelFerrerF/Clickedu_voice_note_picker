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
 * registro. `oido`: lo que se entendió, tal cual. `repetida`: si Whisper
 * repitió la frase en bucle, la primera vez, que es lo que se enseña.
 */
export type Dictado = {id: string; oido: string; repetida?: string} & (
  | {tipo: 'seguro'; alumno: string; nota: number}
  | {tipo: 'duda'; alumnos: string[]; notas: number[]; motivos: MotivoDuda[]}
  | {tipo: 'no_entendido'; motivo: MotivoNoEntendido}
);

/** Lo que hizo el profesor con un dictado: la respuesta buena, para el registro. */
export type Respuesta =
  | {tipo: 'elegida'; clave: string; nota: number}
  | {tipo: 'descartada'}
  | {tipo: 'deshecha'}
  | {tipo: 'corregida'; nota: number | null};

/**
 * Un dictado que espera en el panel: una duda (hasta que eliges) o lo que no
 * se ha entendido (hasta que lo cierras o se dicta otra cosa).
 */
export interface Pendiente {
  /** La clase y su ítem: el `juego` de Calificar (H6). */
  juego: string;
  dictado: Dictado;
}

/**
 * Una nota apuntada por voz (sin preguntar, o al elegir), para enseñarla en
 * el panel entre las últimas. `antes` es la que había, para poder
 * deshacerla: `undefined` si el alumno no se había tocado en la sesión,
 * `null` si se había quitado.
 */
export interface Apuntado {
  /** La clase y su ítem: el `juego` de Calificar (H6). */
  juego: string;
  dictado: Dictado;
  clave: string;
  nota: number;
  antes: number | null | undefined;
}

/**
 * Se puede deshacer el apuntado `i` de `lista` (lo más nuevo primero) si es
 * lo último dictado a ese alumno: si no, su `antes` es una nota que ya no
 * está. Al deshacer el nuevo vuelve la del viejo, y el viejo se puede
 * deshacer otra vez.
 */
export function deshacible(lista: Apuntado[], i: number): boolean {
  return !lista.slice(0, i).some((a) => a.clave === lista[i].clave);
}

/** Las opciones de una duda: cada alumno con cada nota, lo más probable primero. */
export function opciones(d: Extract<Dictado, {tipo: 'duda'}>): {clave: string; nota: number}[] {
  return d.alumnos.flatMap((clave) => d.notas.map((nota) => ({clave, nota}))).slice(0, 4);
}
