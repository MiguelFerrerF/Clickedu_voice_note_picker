/**
 * Las plantillas de Clickedu de la carpeta, una por clase.
 *
 * H0: vacía. Ver `docs/01-plan-de-trabajo.md`.
 */

import {PantallaVacia} from '@components/PantallaVacia';
import {ICONOS} from '@components/ui/Icon';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Clases() {
  useIdioma();
  return (
    <PantallaVacia
      titulo={t('clases.titulo')}
      descripcion={t('clases.vacia')}
      icono={ICONOS.clases}
      hito="H1"
    />
  );
}