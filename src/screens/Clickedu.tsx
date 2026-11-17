/**
 * La conexión con Clickedu: entrar (H4), traer plantillas (H5) y subir notas (H6).
 *
 * H0: vacía. Ver `docs/01-plan-de-trabajo.md`.
 */

import {PantallaVacia} from '@components/PantallaVacia';
import {ICONOS} from '@components/ui/Icon';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Clickedu() {
  useIdioma();
  return (
    <PantallaVacia
      titulo={t('clickedu.titulo')}
      descripcion={t('clickedu.vacia')}
      icono={ICONOS.clickedu}
      hito="H4–H6"
    />
  );
}