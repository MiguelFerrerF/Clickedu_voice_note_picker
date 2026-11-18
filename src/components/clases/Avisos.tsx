/**
 * Lo que la app ha corregido o no le cuadra de una plantilla, en una caja
 * encima de la cuadrícula. El punto ámbar (`--warn`) es «aquí hay algo que
 * mirar», el mismo de la barra lateral; el texto va en el color normal.
 */

import {avisoEnPalabras} from '@modules/clases/avisos';
import type {Aviso} from '@modules/clases/tipos';
import {t} from '@modules/i18n';

export function Avisos({avisos}: {avisos: Aviso[]}) {
  if (avisos.length === 0) return null;
  return (
    <div
      role="status"
      style={{
        marginBottom: 16,
        padding: '12px 16px',
        background: 'var(--bg-card)',
        border: '1px solid var(--border)',
        borderRadius: 8,
      }}>
      <div style={{display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, fontWeight: 600, color: 'var(--text-primary)'}}>
        <Punto />
        {t('avisos.titulo')}
      </div>
      <ul style={{margin: '8px 0 0', paddingLeft: 24, fontSize: 12.5, lineHeight: 1.6, color: 'var(--text-secondary)'}}>
        {avisos.map((a, i) => (
          <li key={i}>{avisoEnPalabras(a)}</li>
        ))}
      </ul>
    </div>
  );
}

/** El punto ámbar de «aquí hay algo que mirar». */
export function Punto() {
  return (
    <span
      aria-hidden
      style={{width: 7, height: 7, borderRadius: '50%', background: 'var(--warn)', flexShrink: 0, display: 'inline-block'}}
    />
  );
}
