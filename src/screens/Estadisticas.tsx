/**
 * Las estadísticas de la clase y del dictado.
 *
 * H0: vacía. Ver `docs/01-plan-de-trabajo.md`.
 */

import {PantallaVacia} from '@components/PantallaVacia';
import {ICONOS} from '@components/ui/Icon';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Estadisticas() {
  useIdioma();
  return (
    <PantallaVacia
      titulo={t('estadisticas.titulo')}
      descripcion={t('estadisticas.vacia')}
      icono={ICONOS.estadisticas}
      hito="H8"
    />
  );
}