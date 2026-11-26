/**
 * La conexión con Clickedu: entrar (H4), traer plantillas (H5) y subir notas (H6).
 *
 * H4: arriba, **«Tu cuenta»** (`TarjetaCuenta`), que se rellena una vez.
 * H5: debajo, **«Plantillas de Clickedu»** (`TarjetaTraer`). Subir las
 * notas (H6) irá después. Diseño en `docs/01-plan-de-trabajo.md`. En
 * «Plantillas de Clickedu», también cómo va el refresco solo (H11).
 *
 * Las dos tarjetas, una al lado de la otra cuando caben (HUI).
 */

import {useState} from 'react';

import {TarjetaCuenta} from '@components/clickedu/TarjetaCuenta';
import {TarjetaTraer} from '@components/clickedu/TarjetaTraer';
import {Cabecera, DOS_COLUMNAS} from '@components/layout/Cabecera';
import {useEstadoRefresco} from '@modules/clickedu/refresco';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export function Clickedu() {
  useIdioma();
  const [hayCuenta, setHayCuenta] = useState<boolean>();
  const [refresco] = useEstadoRefresco();
  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera titulo={t('clickedu.titulo')} intro={t('clickedu.intro')} />
      {/* Lado a lado cuando caben (HUI, 2026-11-25): de una en una, eran
        tiras de todo el ancho con el texto a un lado y el botón al otro. */}
      <div style={DOS_COLUMNAS}>
        <TarjetaCuenta alCambiar={setHayCuenta} />
        <TarjetaTraer hayCuenta={hayCuenta} refresco={refresco} />
      </div>
      <p style={{margin: '16px 0 0', fontSize: 12.5, color: 'var(--text-tertiary)'}}>{t('clickedu.llegan')}</p>
    </div>
  );
}
