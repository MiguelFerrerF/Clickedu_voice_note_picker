/**
 * Desplegable propio, con el tema de la app.
 *
 * COPIADO de library-app (y esta, de la Quality Test App). La lista se pinta por **portal** a
 * `document.body` para que escape de cualquier antepasado con
 * `overflow: hidden` —la cuadrícula de notas lo tiene— y se estiliza con los
 * tokens en vez de con el `<option>` nativo del sistema. Al hacer scroll o
 * redimensionar, el menú sigue al disparador, lo que además permite hacer
 * scroll *dentro* de una lista larga sin que se cierre.
 *
 * Es el que hay que usar para elegir clase, asignatura o ítem: el desplegable
 * del sistema no admite ni recuento por valor ni marca de selección, y en
 * Windows sale en tema del sistema, no en el de la app.
 */

import {Fragment, useEffect, useRef, useState, type CSSProperties} from 'react';
import {createPortal} from 'react-dom';
import {Icon, ICONOS} from './Icon';

export type DropdownOption =
  | string
  | number
  | {value: string | number; label: string; disabled?: boolean; grupo?: string};

export interface DropdownProps {
  value: string | number;
  options: readonly DropdownOption[];
  onChange: (value: string) => void;
  /**
   * 'md' (por defecto) casa con los campos; 'sm' es para filas densas;
   * 'titulo' es el título de la pantalla (HUI): sin caja, con la letra de la
   * `Cabecera`, para que lo que se elige no salga dos veces.
   */
  size?: 'sm' | 'md' | 'titulo';
  mono?: boolean;
  disabled?: boolean;
  title?: string;
  placeholder?: string;
}

/** El alto máximo de la lista, lo que mide una opción y el aire con el borde. */
const ALTO_MAX = 260;
const ALTO_OPCION = 27;
const MARGEN = 8;

function norm(o: DropdownOption): {
  value: string;
  label: string;
  disabled?: boolean;
  grupo?: string;
} {
  if (typeof o === 'object')
    return {value: String(o.value), label: o.label, disabled: o.disabled, grupo: o.grupo};
  return {value: String(o), label: String(o)};
}

export function Dropdown({
  value,
  options,
  onChange,
  size = 'md',
  mono,
  disabled,
  title,
  placeholder,
}: DropdownProps) {
  const [abierto, setAbierto] = useState(false);
  const [pos, setPos] = useState<{
    left: number;
    top?: number;
    bottom?: number;
    width: number;
    alto: number;
  } | null>(null);
  const btnRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const opts = options.map(norm);
  const actual = opts.find((o) => o.value === String(value));
  const sm = size === 'sm';
  const titulo = size === 'titulo';
  const fontFamily = mono ? 'var(--font-mono)' : 'var(--font-body)';

  /**
   * Debajo del disparador si cabe; si no, hacia donde haya más sitio (en una
   * ventana baja, se salía por abajo). Y nunca más alto
   * que ese sitio: lo que no quepa se desplaza dentro.
   */
  const colocar = () => {
    const r = btnRef.current?.getBoundingClientRect();
    if (!r) return;
    const left = Math.min(r.left, window.innerWidth - Math.max(r.width, 220) - 8);
    const grupos = new Set(opts.flatMap((o) => (o.grupo ? [o.grupo] : []))).size;
    const necesita = Math.min(ALTO_MAX, (opts.length + grupos) * ALTO_OPCION + 10);
    const abajo = window.innerHeight - r.bottom - 4 - MARGEN;
    const arriba = r.top - 4 - MARGEN;
    if (abajo >= necesita || abajo >= arriba)
      setPos({left, top: r.bottom + 4, width: r.width, alto: Math.min(ALTO_MAX, abajo)});
    else
      setPos({left, bottom: window.innerHeight - r.top + 4, width: r.width, alto: Math.min(ALTO_MAX, arriba)});
  };

  // Al abrir, la opción elegida a la vista: en una lista larga (los temas)
  // puede quedar por debajo del borde.
  useEffect(() => {
    if (!abierto) return;
    menuRef.current
      ?.querySelector<HTMLElement>('[data-elegida]')
      ?.scrollIntoView({block: 'nearest'});
  }, [abierto]);

  useEffect(() => {
    if (!abierto) return;
    // Esc cierra la lista y nada más: se oye en captura y no sigue, para que
    // no llegue a la pantalla de debajo, que podría usarla para otra cosa.
    const alPulsarTecla = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      setAbierto(false);
      e.stopPropagation();
      e.preventDefault();
    };
    const alPulsarFuera = (e: MouseEvent) => {
      const t = e.target as Node;
      if (!btnRef.current?.contains(t) && !menuRef.current?.contains(t))
        setAbierto(false);
    };
    window.addEventListener('scroll', colocar, true);
    window.addEventListener('resize', colocar);
    window.addEventListener('keydown', alPulsarTecla, true);
    document.addEventListener('mousedown', alPulsarFuera);
    return () => {
      window.removeEventListener('scroll', colocar, true);
      window.removeEventListener('resize', colocar);
      window.removeEventListener('keydown', alPulsarTecla, true);
      document.removeEventListener('mousedown', alPulsarFuera);
    };
  }, [abierto]);

  const alternar = () => {
    if (disabled) return;
    if (abierto) {
      setAbierto(false);
      return;
    }
    colocar();
    setAbierto(true);
  };

  // El título: el texto donde estaría sin botón (el margen negativo
  // compensa el relleno). El fondo al pasar, que dice que se pulsa, y el
  // foco, en `.titulo-desplegable` de `globals.css`.
  const estiloTitulo: CSSProperties = {
    maxWidth: 'calc(100% + 10px)',
    display: 'inline-flex',
    alignItems: 'center',
    gap: 8,
    margin: '-3px -10px',
    padding: '3px 10px',
    fontFamily: 'var(--font-display)',
    fontSize: 22,
    fontWeight: 600,
    letterSpacing: '-0.015em',
    background: abierto ? 'var(--bg-hover)' : undefined,
    color: 'var(--text-primary)',
    border: 'none',
    borderRadius: 7,
    cursor: disabled ? 'default' : 'pointer',
    opacity: disabled ? 0.6 : 1,
  };

  const estiloDisparador: CSSProperties = titulo ? estiloTitulo : {
    width: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    gap: 6,
    padding: sm ? '3px 8px' : '8px 12px',
    fontSize: sm ? 11 : 13,
    fontFamily,
    // El campo «papel» de `inputs.module.css`: en un formulario va
    // al lado de los de texto y tiene que parecer de la misma familia.
    background: 'var(--bg-card)',
    color: 'var(--text-primary)',
    border: `1px solid ${abierto ? 'var(--accent)' : 'var(--border-strong)'}`,
    borderRadius: sm ? 6 : 7,
    cursor: disabled ? 'default' : 'pointer',
    outline: 'none',
    opacity: disabled ? 0.6 : 1,
  };

  return (
    <div style={{position: 'relative', width: '100%', minWidth: 0}}>
      <button
        ref={btnRef}
        type="button"
        onClick={alternar}
        title={title}
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={abierto}
        className={titulo ? 'titulo-desplegable' : undefined}
        style={estiloDisparador}>
        <span
          style={{
            whiteSpace: 'nowrap',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            color: actual ? undefined : 'var(--text-tertiary)',
          }}>
          {actual?.label ?? placeholder ?? String(value)}
        </span>
        <span
          style={{
            flexShrink: 0,
            color: titulo ? 'var(--text-secondary)' : 'var(--text-tertiary)',
            display: 'flex',
            transform: abierto ? 'rotate(180deg)' : 'none',
            transition: 'transform .15s',
          }}>
          <Icon d={ICONOS.galonAbajo} size={sm ? 10 : titulo ? 16 : 12} />
        </span>
      </button>
      {abierto &&
        pos &&
        createPortal(
          <div
            ref={menuRef}
            style={{
              position: 'fixed',
              left: pos.left,
              top: pos.top,
              bottom: pos.bottom,
              minWidth: pos.width,
              width: 'max-content',
              maxWidth: 320,
              maxHeight: pos.alto,
              overflowY: 'auto',
              background: 'var(--bg-card)',
              border: '1px solid var(--border)',
              borderRadius: 8,
              boxShadow: '0 6px 20px oklch(0 0 0 / 0.18)',
              padding: 4,
              zIndex: 1000,
            }}>
            {opts.map((o, i) => {
              const sel = o.value === String(value);
              // Un grupo (las pendientes de otros cursos) empieza con su
              // encabezado, como en las tablas de Clases y Conexión.
              const empiezaGrupo = o.grupo !== undefined && o.grupo !== opts[i - 1]?.grupo;
              return (
                <Fragment key={o.value}>
                  {empiezaGrupo && (
                    <div
                      style={{
                        margin: i > 0 ? '4px -4px 0' : '0 -4px',
                        padding: '7px 12px 5px',
                        borderTop: i > 0 ? '1px solid var(--border)' : undefined,
                        fontSize: 10.5,
                        fontWeight: 600,
                        letterSpacing: 0.8,
                        textTransform: 'uppercase',
                        color: 'var(--text-tertiary)',
                        whiteSpace: 'nowrap',
                      }}>
                      {o.grupo}
                    </div>
                  )}
                  <button
                    type="button"
                    data-elegida={sel || undefined}
                    disabled={o.disabled}
                    onClick={() => {
                      if (!o.disabled) {
                        onChange(o.value);
                        setAbierto(false);
                      }
                    }}
                    onMouseEnter={(e) => {
                      if (!sel && !o.disabled)
                        e.currentTarget.style.background = 'var(--bg-hover)';
                    }}
                    onMouseLeave={(e) => {
                      if (!sel) e.currentTarget.style.background = 'transparent';
                    }}
                    style={{
                      width: '100%',
                      textAlign: 'left',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      gap: 8,
                      padding: '5px 8px',
                      fontSize: sm ? 11.5 : 12.5,
                      fontFamily,
                      border: 'none',
                      borderRadius: 5,
                      cursor: o.disabled ? 'default' : 'pointer',
                      opacity: o.disabled ? 0.45 : 1,
                      // El acento al 16 % es exactamente `--accent-wash` en
                      // oscuro; se usa el token en vez de repetir el color.
                      background: sel ? 'var(--accent-wash)' : 'transparent',
                      color: sel ? 'var(--accent-ink)' : 'var(--text-primary)',
                    }}>
                    <span
                      style={{
                        whiteSpace: 'nowrap',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                      }}>
                      {o.label}
                    </span>
                    {sel && <Icon d={ICONOS.comprobado} size={11} />}
                  </button>
                </Fragment>
              );
            })}
          </div>,
          document.body,
        )}
    </div>
  );
}
