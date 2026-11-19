/**
 * Lo que devuelve `resumen_registro` (D8): el espejo de `registro::Resumen`.
 * Si cambia allí, cambia aquí.
 */

export interface ResumenRegistro {
  dictados: number;
  /** Apuntados sin preguntar. */
  seguros: number;
  preguntados: number;
  noEntendidos: number;
  /** Deshechos o corregidos a mano después: lo que la app apuntó mal. */
  corregidos: number;
  fallos: number;
  /** El caso más antiguo, en segundos desde 1970; `null` si no hay ninguno. */
  desde: number | null;
}
