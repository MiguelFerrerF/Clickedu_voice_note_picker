/**
 * Lo que devuelven los comandos de las Clases (H1). Es el espejo de
 * `src-tauri/src/plantillas.rs` y `ajustes.rs`: si cambia allí, cambia aquí.
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
  nota: Nota;
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

/** Lo que manda el backend cuando cambia algo en la carpeta de plantillas. */
export const EVENTO_PLANTILLAS = 'plantillas-cambiadas';
