/**
 * Lo que devuelve `leer_estadisticas` (H8). Es `commands::estadisticas::
 * Estadisticas`, con `estadisticas::Notas` y `registro::estadisticas::
 * Dictados`. Y lo que devuelve `leer_curso`: `estadisticas::curso::Curso`.
 */

export interface NotasClase {
  alumnos: number;
  conNota: number;
  sinNota: number;
  /** Lo que hay en el campo y no es una nota (`NP`). */
  noSonNota: number;
  media: number | null;
  mediana: number | null;
  aprobados: number;
  /** Cuántas de cada punto: la 0, de 0 a 0,99; la 10, el 10. Once. */
  porPunto: number[];
  /** Quién tiene cada nota, de la más alta a la más baja. Solo para la pantalla (D8). */
  quienes: Quien[];
}

export interface Quien {
  /** La clave del alumno en la plantilla: para ir a lo suyo. */
  clave: string;
  nombre: string;
  nota: number;
  /** Su columna en `porPunto`. */
  punto: number;
}

export interface Dictados {
  porVoz: number;
  aLaPrimera: number;
  preguntados: number;
  noEntendidos: number;
  corregidos: number;
  segundosPorNota: number | null;
  notasLlevadas: number;
  subidas: number;
  exportadas: number;
  /** En segundos; `null` si no se ha llevado nada a Clickedu. */
  ahorro: number | null;
}

export interface Estadisticas {
  notas: NotasClase;
  dictado: {clase: Dictados; todas: Dictados};
}

/** Cada evaluación y cada alumno de una clase traída, con todos sus ítems (H8). */
export interface Curso {
  /** Cuándo se trajeron las notas de Clickedu (segundos Unix). */
  traidos: number;
  evaluaciones: EvaluacionCurso[];
  /** En el orden de la plantilla. */
  alumnos: AlumnoCurso[];
}

export interface EvaluacionCurso {
  id: string;
  nombre: string;
  enCurso: boolean;
  items: ItemCurso[];
  /** Ningún ítem tiene ponderación: la calculada es la media simple. */
  mediaSimple: boolean;
  /** Los ítems sin ponderación cuando los demás la tienen: no cuentan en la calculada. */
  noCuentan: string[];
  /** La media ponderada de sus ítems, con lo de la app: provisional. */
  calculada: NotasClase;
  /** La «Nota final» de Clickedu, al traer. */
  finalClickedu: NotasClase;
}

export interface ItemCurso {
  id: string;
  nombre: string;
  /** En % de la evaluación. */
  ponderacion: number | null;
  /** El ítem del que cuelga. */
  dentroDe: string | null;
  conNota: number;
  media: number | null;
  aprobados: number;
}

export interface AlumnoCurso {
  clave: string;
  nombre: string;
  /** Una por evaluación, en el orden de `Curso.evaluaciones`. */
  evaluaciones: DelAlumno[];
}

export interface DelAlumno {
  /** Su nota en cada ítem, en el orden de `EvaluacionCurso.items`. */
  notas: (number | null)[];
  calculada: number | null;
  /** En cuántos de los ítems que cuentan no tiene nota. */
  faltan: number;
  finalClickedu: number | null;
}

/** Desde aquí se aprueba (`estadisticas::APROBADO`). */
export const APROBADO = 5;
