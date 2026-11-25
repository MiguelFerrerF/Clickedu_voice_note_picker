/**
 * El recorrido en tres pasos (H9): traer, calificar, subir. Lo enseñan la
 * bienvenida y la Ayuda, con los mismos textos.
 */

import {t, type TranslationKey} from '@modules/i18n';

const PASOS: readonly [TranslationKey, TranslationKey][] = [
  ['bienvenida.traer', 'bienvenida.traer_texto'],
  ['bienvenida.calificar', 'bienvenida.calificar_texto'],
  ['bienvenida.subir', 'bienvenida.subir_texto'],
];

/** `enFila`: los tres uno al lado del otro si caben (la Ayuda); si no, uno debajo de otro. */
export function Recorrido({enFila = false}: {enFila?: boolean}) {
  return (
    <ol
      style={{
        listStyle: 'none',
        margin: 0,
        padding: 0,
        display: 'grid',
        gridTemplateColumns: enFila ? 'repeat(auto-fit, minmax(240px, 1fr))' : undefined,
        gap: enFila ? '12px 28px' : 12,
      }}>
      {PASOS.map(([titulo, texto], i) => (
        <li key={titulo} style={{display: 'flex', gap: 12, alignItems: 'flex-start', minWidth: 0}}>
          <span
            aria-hidden
            style={{
              flexShrink: 0,
              width: 22,
              height: 22,
              borderRadius: '50%',
              display: 'inline-flex',
              alignItems: 'center',
              justifyContent: 'center',
              background: 'var(--bg-sunk)',
              color: 'var(--text-secondary)',
              fontFamily: 'var(--font-mono)',
              fontSize: 11.5,
            }}>
            {i + 1}
          </span>
          <div style={{minWidth: 0}}>
            <div style={{fontSize: 13.5, fontWeight: 600, color: 'var(--text-primary)'}}>{t(titulo)}</div>
            <div style={{marginTop: 2, fontSize: 13, lineHeight: 1.5, color: 'var(--text-secondary)'}}>{t(texto)}</div>
          </div>
        </li>
      ))}
    </ol>
  );
}
