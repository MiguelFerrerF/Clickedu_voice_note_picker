/**
 * Lo que devuelve `leer_estadisticas` (H8). Es `commands::estadisticas::
 * Estadisticas`, con `estadisticas::Notas` y `registro::estadisticas::
 * Dictados`.
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

/** Desde aquí se aprueba (`estadisticas::APROBADO`). */
export const APROBADO = 5;
