/**
 * Lo que la app ha corregido o no le cuadra de una plantilla, encima de la
 * cuadrícula. El punto ámbar (`--warn`) es «aquí hay algo que mirar», el
 * mismo de la barra lateral; el texto va en el color normal.
 *
 * **Plegados en una línea** (HUI): «● 3 avisos al leer la plantilla ›», que
 * se abre al pulsarla. Se dicen siempre (ninguna normalización en silencio),
 * pero abiertos empujaban la clase hacia abajo cada vez que se abría. Quien los
 * monta les pone `key` por clase: otra clase empieza plegada.
 */

import {useId, useState} from 'react';

import {Icon, ICONOS} from '@components/ui/Icon';
import {avisoEnPalabras} from '@modules/clases/avisos';
import type {Aviso} from '@modules/clases/tipos';
import {t} from '@modules/i18n';

export function Avisos({avisos}: {avisos: Aviso[]}) {
  if (avisos.length === 0) return null;
  return (
    <Plegados
      rotulo={avisos.length === 1 ? t('avisos.plegados_uno') : t('avisos.plegados', {n: avisos.length})}
      textos={avisos.map(avisoEnPalabras)}
      style={{marginBottom: 12}}
    />
  );
}

/**
 * La línea plegada con el punto ámbar y, al abrirla, `textos` debajo. La
 * usan también los avisos de Clickedu (H5).
 */
export function Plegados({rotulo, textos, style}: {rotulo: string; textos: string[]; style?: React.CSSProperties}) {
  const [abiertos, setAbiertos] = useState(false);
  const id = useId();
  return (
    <div role="status" style={style}>
      <button
        type="button"
        className="linea-plegable"
        onClick={() => setAbiertos((a) => !a)}
        aria-expanded={abiertos}
        aria-controls={id}
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: 8,
          margin: '0 -8px',
          padding: '4px 8px',
          fontSize: 13,
          color: 'var(--text-secondary)',
          border: 'none',
          borderRadius: 6,
          cursor: 'pointer',
        }}>
        <Punto />
        {rotulo}
        <span
          style={{
            display: 'flex',
            transform: abiertos ? 'rotate(90deg)' : 'none',
            transition: 'transform .15s',
          }}>
          <Icon d={ICONOS.galonDerecha} size={14} />
        </span>
      </button>
      {abiertos && (
        <ul
          id={id}
          style={{
            margin: '4px 0 0',
            paddingLeft: 24,
            fontSize: 12.5,
            lineHeight: 1.6,
            color: 'var(--text-secondary)',
            maxWidth: 760,
          }}>
          {textos.map((texto, i) => (
            <li key={i}>{texto}</li>
          ))}
        </ul>
      )}
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
