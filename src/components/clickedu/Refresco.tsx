/**
 * Lo del refresco solo (H11) que enseñan Inicio, Conexión y Ajustes.
 *
 * - `LineaRefresco`: cómo va, en una línea gris. Un fallo de red **no** va
 *   en rojo: se vuelve a probar solo (sin ruido, `docs/01`). Sí la cuenta
 *   que Clickedu no acepta: eso es un error de verdad y pide hacer algo
 *   (D6).
 * - `AvisoPorTraer`: las plantillas que han cambiado en Clickedu, plegadas
 *   con el punto ámbar, como los avisos de traer. No se traen solas.
 */

import {Plegados} from '@components/clases/Avisos';
import {cuando, type EstadoRefresco, type PorTraer} from '@modules/clickedu/refresco';
import {t} from '@modules/i18n';

const LINEA: React.CSSProperties = {margin: 0, fontSize: 12.5, lineHeight: 1.5, color: 'var(--text-tertiary)'};

export function LineaRefresco({estado}: {estado?: EstadoRefresco}) {
  if (!estado) return null;
  if (!estado.encendido) return <p style={LINEA}>{t('refresco.apagado')}</p>;
  if (estado.parado)
    return (
      <p role="alert" style={{...LINEA, color: 'var(--danger)'}}>
        {t('refresco.parado')}
      </p>
    );
  if (estado.refrescando) return <p style={LINEA}>{t('refresco.refrescando')}</p>;
  if (estado.fallo)
    return (
      <p style={LINEA} title={estado.fallo}>
        {t('refresco.fallo', {minutos: estado.minutos})}
      </p>
    );
  if (estado.ultimo !== null) return <p style={LINEA}>{t('refresco.al_dia', {cuando: cuando(estado.ultimo)})}</p>;
  return <p style={LINEA}>{t('refresco.todavia')}</p>;
}

function queCambia(p: PorTraer): string {
  switch (p.que) {
    case 'nueva':
      return t('refresco.nueva', {fichero: p.fichero});
    case 'cambia':
      return t('refresco.cambia', {fichero: p.fichero});
    case 'ya_no_esta':
      return t('refresco.ya_no_esta', {fichero: p.fichero});
  }
}

export function AvisoPorTraer({porTraer}: {porTraer: PorTraer[]}) {
  if (porTraer.length === 0) return null;
  return (
    <Plegados
      rotulo={porTraer.length === 1 ? t('refresco.por_traer_uno') : t('refresco.por_traer', {n: porTraer.length})}
      textos={porTraer.map(queCambia)}
    />
  );
}
