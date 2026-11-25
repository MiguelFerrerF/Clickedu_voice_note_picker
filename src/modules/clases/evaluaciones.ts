/**
 * Las evaluaciones de todas las clases, juntadas por su nombre («2ª
 * evaluación»): su número en Clickedu cambia de una etapa a otra. Las usan
 * Clases y el Inicio (HUI).
 */

import type {Clase} from './tipos';

/** Una evaluación del selector: la de varias clases con el mismo nombre. */
export interface Evaluacion {
  clave: string;
  nombre: string;
  /** La de hoy en el mayor número de clases. */
  hoy: boolean;
}

/** Con qué se juntan las evaluaciones de clases distintas: su nombre. */
export const claveDe = (nombre: string) => nombre.trim().toLowerCase().replace(/\s+/g, ' ');

/** Las evaluaciones de todas las clases, en el orden en que salen. */
export function evaluacionesDe(clases: Clase[]): Evaluacion[] {
  const vistas = new Map<string, {nombre: string; enCurso: number}>();
  for (const c of clases)
    for (const e of c.items?.evaluaciones ?? []) {
      const v = vistas.get(claveDe(e.nombre)) ?? {nombre: e.nombre, enCurso: 0};
      v.enCurso += e.enCurso ? 1 : 0;
      vistas.set(claveDe(e.nombre), v);
    }
  const mas = Math.max(0, ...[...vistas.values()].map((v) => v.enCurso));
  return [...vistas].map(([clave, v]) => ({clave, nombre: v.nombre, hoy: mas > 0 && v.enCurso === mas}));
}
