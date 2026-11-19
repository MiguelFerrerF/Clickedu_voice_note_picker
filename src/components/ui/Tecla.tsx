/**
 * Una tecla, dibujada como tal: se reconoce antes que leída.
 *
 * Nació en el panel del dictado (ESPACIO, 1–4, Esc) y se sacó aquí cuando la
 * ayuda de la cuadrícula la necesitó también (HUI).
 */

import {Fragment} from 'react';

export function Tecla({children}: {children: React.ReactNode}) {
  return (
    <kbd
      style={{
        display: 'inline-block',
        minWidth: 20,
        padding: '0 6px',
        border: '1px solid var(--border-strong)',
        borderBottomWidth: 2,
        borderRadius: 4,
        background: 'var(--bg-card)',
        color: 'var(--text-primary)',
        fontFamily: 'var(--font-mono)',
        fontSize: 11,
        fontWeight: 600,
        lineHeight: '17px',
        textAlign: 'center',
        whiteSpace: 'nowrap',
      }}>
      {children}
    </kbd>
  );
}

/**
 * Un texto con teclas: cada `{nombre}` de `teclas` se dibuja como la tecla
 * con ese rótulo. Lo que no está en `teclas` se deja como está.
 */
export function conTeclas(texto: string, teclas: Readonly<Record<string, string>>) {
  return texto.split(/(\{\w+\})/).map((trozo, i) => {
    const rotulo = trozo.startsWith('{') ? teclas[trozo.slice(1, -1)] : undefined;
    return rotulo === undefined ? <Fragment key={i}>{trozo}</Fragment> : <Tecla key={i}>{rotulo}</Tecla>;
  });
}
