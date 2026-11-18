/**
 * Lo que devuelven los comandos de las Clases (H1) y de las notas (H2). Es el
 * espejo de `src-tauri/src/plantillas.rs`, `ajustes.rs`, `sesion.rs`,
 * `exportar.rs` y `commands/`: si cambia allí, cambia aquí.
 */

export interface CarpetaPlantillas {
  ruta: string;
  porDefecto: boolean;
  existe: boolean;
}

export type Lectura =
  | {tipo: 'bien'; alumnos: number; calificados: number; avisos: number}
  | {tipo: 'mal'; error: string};

export interface Clase {
  /** El nombre del fichero, que es también su identificador. */
  fichero: string;
  /** El nombre legible, sacado del fichero. */
  nombre: string;
  lectura: Lectura;
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
