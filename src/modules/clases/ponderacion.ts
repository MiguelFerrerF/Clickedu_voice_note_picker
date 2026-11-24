/**
 * La ponderación de un ítem, como se enseña en Clases y Calificar (H6): en %
 * de la evaluación, a la española («3,33 %»). La de un ítem que cuelga de
 * otro ya llega multiplicada (`clickedu/items.rs`); la ayuda dice de dónde sale.
 */

import type {ItemClickedu} from '@modules/clases/tipos';
import {t} from '@modules/i18n';

const PORCENTAJE = new Intl.NumberFormat('es-ES', {maximumFractionDigits: 2});

/** «3,33 %», o nada si no hay. */
export function ponderacionTexto(item: ItemClickedu): string | null {
  return item.ponderacion === null ? null : t('clases.ponderacion', {n: PORCENTAJE.format(item.ponderacion)});
}

/** Si cuelga de otro ítem: «El 30 % de «Prácticas»». */
export function ponderacionAyuda(item: ItemClickedu): string | undefined {
  const d = item.dentroDe;
  if (!d) return undefined;
  return d.ponderacion === null
    ? t('clases.ponderacion_dentro_sin', {nombre: d.nombre})
    : t('clases.ponderacion_dentro', {n: PORCENTAJE.format(d.ponderacion), nombre: d.nombre});
}
