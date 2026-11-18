/**
 * Los avisos de una plantilla, en palabras. **Ninguna normalización en
 * silencio**: lo que la app corrige al leer, o lo que no cuadra, se dice.
 */

import {t} from '@modules/i18n';

import type {Aviso} from './tipos';

/** «la fila 4», «las filas 2 y 3», «las filas 2, 3, 5 y 9 más». */
export function filasEnPalabras(filas: number[]): string {
  if (filas.length === 1) return t('aviso.la_fila', {fila: filas[0]});
  const MAX = 4;
  if (filas.length > MAX) {
    return t('aviso.las_filas_y_mas', {
      filas: filas.slice(0, MAX - 1).join(', '),
      ultima: filas[MAX - 1],
      mas: filas.length - MAX,
    });
  }
  return t('aviso.las_filas', {filas: filas.slice(0, -1).join(', '), ultima: filas[filas.length - 1]});
}

/**
 * «Castro Molina, Lucía», «A» y «B», «A», «B», «C» y 2 más. Entre comillas
 * porque el nombre ya lleva coma (`Apellidos, Nombre`).
 */
export function alumnosEnPalabras(nombres: string[]): string {
  const q = nombres.map((n) => `«${n.trim().replace(/\s+/g, ' ')}»`);
  if (q.length === 1) return q[0];
  const MAX = 3;
  if (q.length > MAX) {
    return t('lista.y_mas', {primeros: q.slice(0, MAX - 1).join(', '), ultimo: q[MAX - 1], mas: q.length - MAX});
  }
  return t('lista.y', {primeros: q.slice(0, -1).join(', '), ultimo: q[q.length - 1]});
}

export function avisoEnPalabras(a: Aviso): string {
  switch (a.tipo) {
    case 'columnas_por_posicion':
      return t('aviso.columnas_por_posicion');
    case 'notas_con_coma':
      return t('aviso.notas_con_coma', {filas: filasEnPalabras(a.filas)});
    case 'notas_invalidas':
      return t('aviso.notas_invalidas', {filas: filasEnPalabras(a.filas)});
    case 'sin_nombre':
      return t('aviso.sin_nombre', {filas: filasEnPalabras(a.filas)});
    case 'id_repetido':
      return t('aviso.id_repetido', {id: a.id, filas: filasEnPalabras(a.filas)});
  }
}
