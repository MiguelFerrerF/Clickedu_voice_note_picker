/**
 * Lo que devuelven los comandos de las Clases (H1) y de las notas (H2). Es el
 * espejo de `src-tauri/src/plantillas.rs`, `ajustes.rs`, `sesion.rs`,
 * `exportar.rs`, `clickedu/items.rs` y `commands/`: si cambia allí, cambia aquí.
 */

export interface CarpetaPlantillas {
  ruta: string;
  porDefecto: boolean;
  existe: boolean;
}

export type Lectura =
  {tipo: 'bien'; alumnos: number; calificados: number; avisos: number} | {tipo: 'mal'; error: string};

export interface Clase {
  /** El nombre del fichero, que es también su identificador. */
  fichero: string;
  /** El nombre legible, sacado del fichero. */
  nombre: string;
  /** De otro curso (su fichero empieza por `Pendiente - `): va aparte. */
  pendiente: boolean;
  lectura: Lectura;
  /**
   * Sus ítems de Clickedu, guardados al traer (H6). `null` si no se han
   * traído nunca (quien no se conecta, o una plantilla puesta a mano): se
   * califica sin ítem, como en la v1. Es `clickedu::items::ItemsClase`.
   */
  items: ItemsClase | null;
}

/** Los ítems de una clase, por evaluación, tal como estaban al traer (H6). */
export interface ItemsClase {
  /** Cuándo se trajeron (segundos Unix): sin conexión, se enseña. */
  traidos: number;
  evaluaciones: EvaluacionItems[];
}

export interface EvaluacionItems {
  id: string;
  nombre: string;
  /** La de hoy, por sus fechas: se enseña abierta. */
  enCurso: boolean;
  items: ItemClickedu[];
}

/** Un ítem de Clickedu: a él se suben las notas (`docs/03`). */
export interface ItemClickedu {
  id: string;
  nombre: string;
  /** En %, de la tabla de Ítems; `null` si no se ha podido leer. */
  ponderacion: number | null;
  /** Cuántos alumnos de la clase tenían ya nota en Clickedu en este ítem, al traer. */
  enClickedu: number;
  /** Cuántos tienen nota en la sesión de este ítem. */
  calificados: number;
  /** Cuántos están publicados: en Clickedu y, en la app, ninguna o la misma. La barra los pinta aparte. */
  publicadas: number;
  /** Cuántos tienen en la app una nota que no está igual en Clickedu: lo que queda por subir. */
  porSubir: number;
}

/** El ítem en el que se califica una clase. */
export interface ItemElegido {
  evaluacion: string;
  item: string;
}

/** El ítem y su evaluación, si `elegido` sigue entre los de la clase. */
export function buscarItem(
  items: ItemsClase | null | undefined,
  elegido: ItemElegido | null | undefined,
): {evaluacion: EvaluacionItems; item: ItemClickedu} | undefined {
  if (!items || !elegido) return undefined;
  const evaluacion = items.evaluaciones.find((e) => e.id === elegido.evaluacion);
  const item = evaluacion?.items.find((i) => i.id === elegido.item);
  return evaluacion && item ? {evaluacion, item} : undefined;
}

export interface ListaClases {
  carpeta: CarpetaPlantillas;
  clases: Clase[];
}

export type Nota = {tipo: 'vacia'} | {tipo: 'valor'; valor: number} | {tipo: 'invalida'; valor: string};

export interface Alumno {
  /** La fila del Excel, desde 1. */
  fila: number;
  id: string;
  nombre: string;
  /** La nota de la plantilla, tal cual. La que vale es `notaDe`. */
  nota: Nota;
  /** Con qué se guarda su nota en la sesión: el ID, o `fila:N`. */
  clave: string;
}

export interface Columna {
  indice: number;
  cabecera: string;
}

export interface Columnas {
  hoja: string;
  filaCabecera: number;
  id: Columna;
  nombre: Columna;
  nota: Columna;
}

export type Aviso =
  | {tipo: 'columnas_por_posicion'}
  | {tipo: 'notas_con_coma'; filas: number[]}
  | {tipo: 'notas_invalidas'; filas: number[]}
  | {tipo: 'sin_nombre'; filas: number[]}
  | {tipo: 'id_repetido'; id: string; filas: number[]};

export interface Plantilla {
  alumnos: Alumno[];
  columnas: Columnas;
  avisos: Aviso[];
}

/**
 * Las notas puestas en la sesión, por clave de alumno. `null` es una nota
 * quitada a propósito: la plantilla traía una y se ha borrado.
 */
export type Puestas = Record<string, number | null>;

/** Lo que devuelve `leer_clase`: la plantilla y las notas puestas. */
export interface ClaseAbierta {
  plantilla: Plantilla;
  puestas: Puestas;
  /**
   * En un ítem (H6), las notas que tenían en Clickedu al traer, por `id` de
   * alumno y tal cual llegan (`7.5`, o un texto). `null` sin ítem.
   */
  enClickedu: Record<string, string> | null;
}

/** Lo que devuelve `exportar_clase`. */
export interface Exportado {
  ruta: string;
  conNota: number;
  alumnos: number;
  /** Dónde ha ido el exportado de antes, si lo había. */
  viejo: string | null;
}

/** Lo que manda el backend cuando cambia algo en la carpeta de plantillas. */
export const EVENTO_PLANTILLAS = 'plantillas-cambiadas';

/** Lo que manda el backend cuando se guarda una nota. */
export const EVENTO_NOTAS = 'notas-cambiadas';
