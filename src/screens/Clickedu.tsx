/**
 * La conexión con Clickedu: entrar (H4), traer plantillas (H5) y subir notas (H6).
 *
 * H4: arriba, **«Tu cuenta»** (`TarjetaCuenta`), que se rellena una vez.
 * H5: debajo, **«Plantillas de Clickedu»** (`TarjetaTraer`). Subir las
 * notas (H6) irá después. Diseño en `docs/01-plan-de-trabajo.md`.
 */

import {useState} from 'react';

import {TarjetaCuenta} from '@components/clickedu/TarjetaCuenta';
import {TarjetaTraer} from '@components/clickedu/TarjetaTraer';
import {Cabecera} from '@components/layout/Cabecera';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Clickedu() {
  useIdioma();
  const [hayCuenta, setHayCuenta] = useState<boolean>();
  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera titulo={t('clickedu.titulo')} intro={t('clickedu.intro')} />
      <div style={{display: 'flex', flexDirection: 'column', gap: 16}}>
        <TarjetaCuenta alCambiar={setHayCuenta} />
        <TarjetaTraer hayCuenta={hayCuenta} />
        <p style={{margin: 0, fontSize: 12.5, color: 'var(--text-tertiary)'}}>{t('clickedu.llegan')}</p>
      </div>
    </div>
  );
}
