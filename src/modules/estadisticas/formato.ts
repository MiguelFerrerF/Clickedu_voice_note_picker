/**
 * Cómo se escriben las cifras de las Estadísticas (H8) y del Inicio (HUI).
 */

import {t} from '@modules/i18n';

export const NUM = new Intl.NumberFormat('es-ES', {useGrouping: 'always'});

/** Los segundos, en minutos (y horas, si llega). */
export function tiempo(segundos: number): string {
  const min = Math.round(segundos / 60);
  if (min < 60) return t('estadisticas.min', {n: NUM.format(min)});
  return t('estadisticas.horas', {h: NUM.format(Math.floor(min / 60)), n: NUM.format(min % 60)});
}
