/**
 * La pantalla principal: la clase elegida, sus alumnos y sus notas. H1 la llena en lectura, H2 deja escribir y exportar, H3 trae el dictado.
 *
 * H0: vacía. Ver `docs/01-plan-de-trabajo.md`.
 */

import {PantallaVacia} from '@components/PantallaVacia';
import {ICONOS} from '@components/ui/Icon';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Calificar() {
  useIdioma();
  return (
    <PantallaVacia
      titulo={t('calificar.titulo')}
      descripcion={t('calificar.vacia')}
      icono={ICONOS.calificar}
      hito="H1–H3"
    />
  );
}