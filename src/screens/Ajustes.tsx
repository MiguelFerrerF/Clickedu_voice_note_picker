/**
 * Carpetas, micrófono y cuenta. Cada ajuste llega con el hito que lo necesita; la pantalla, con el primero (H2: las carpetas).
 *
 * H0: vacía. Ver `docs/01-plan-de-trabajo.md`.
 */

import {PantallaVacia} from '@components/PantallaVacia';
import {ICONOS} from '@components/ui/Icon';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Ajustes() {
  useIdioma();
  return (
    <PantallaVacia
      titulo={t('ajustes.titulo')}
      descripcion={t('ajustes.vacia')}
      icono={ICONOS.ajustes}
      hito="H2"
    />
  );
}