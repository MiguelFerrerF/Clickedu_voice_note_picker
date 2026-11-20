/**
 * La conexión con Clickedu: entrar (H4), traer plantillas (H5) y subir notas (H6).
 *
 * H4: arriba, **«Tu cuenta»** (`TarjetaCuenta`), que se rellena una vez.
 * Debajo irán traer las plantillas y subir las notas. Diseño del
 * 2026-11-20, en `docs/01-plan-de-trabajo.md`.
 */

import {TarjetaCuenta} from '@components/clickedu/TarjetaCuenta';
import {Cabecera} from '@components/layout/Cabecera';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Clickedu() {
  useIdioma();
  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera titulo={t('clickedu.titulo')} intro={t('clickedu.intro')} />
      <div style={{display: 'flex', flexDirection: 'column', gap: 16}}>
        <TarjetaCuenta />
        <p style={{margin: 0, fontSize: 12.5, color: 'var(--text-tertiary)'}}>{t('clickedu.llegan')}</p>
      </div>
    </div>
  );
}
