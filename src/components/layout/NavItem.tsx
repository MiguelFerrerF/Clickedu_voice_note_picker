/**
 * Una fila de navegación dentro de un `SidebarSection`.
 *
 * COPIADO de la Quality Test App. La franja de 2 px a la izquierda es **lo
 * único** que marca la fila activa, y así se queda: el color marca eventos y
 * excepciones, no estados normales (`07-sistema-visual.md` §3).
 */

import {Icon} from '@components/ui/Icon';

export interface NavItemProps {
  activo?: boolean;
  icono: string | readonly string[];
  rotulo: string;
  /** Contador a la derecha — p. ej. los 4.051 pendientes. */
  insignia?: string;
  /**
   * Un punto de color a la derecha: aquí hay algo que mirar (hoy, que toca
   * copia de seguridad). Es una excepción, así que sí lleva color.
   */
  punto?: boolean;
  onClick?: () => void;
  /** Fila apagada y no pulsable, para lo que todavía no existe. */
  deshabilitado?: boolean;
  /** `title` nativo; sirve sobre todo para explicar un item deshabilitado. */
  ayuda?: string;
}

export function NavItem({
  activo = false,
  icono,
  rotulo,
  insignia,
  punto = false,
  onClick,
  deshabilitado = false,
  ayuda,
}: NavItemProps) {
  return (
    <button
      type="button"
      disabled={deshabilitado}
      title={ayuda}
      onClick={deshabilitado ? undefined : onClick}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        width: '100%',
        padding: '9px 12px',
        borderRadius: 8,
        border: 'none',
        cursor: deshabilitado ? 'not-allowed' : 'pointer',
        opacity: deshabilitado ? 0.45 : 1,
        background: activo ? 'var(--bg-active)' : 'transparent',
        color: activo ? 'var(--text-primary)' : 'var(--text-secondary)',
        fontSize: 13.5,
        fontWeight: activo ? 500 : 400,
        fontFamily: 'var(--font-body)',
        textAlign: 'left',
        position: 'relative',
        transition: 'background 120ms, color 120ms',
      }}
      onMouseEnter={(e) => {
        if (!activo && !deshabilitado) e.currentTarget.style.background = 'var(--bg-hover)';
      }}
      onMouseLeave={(e) => {
        if (!activo && !deshabilitado) e.currentTarget.style.background = 'transparent';
      }}>
      {activo && (
        <div
          style={{
            position: 'absolute',
            left: -8,
            top: 8,
            bottom: 8,
            width: 2,
            background: 'var(--accent)',
            borderRadius: 2,
          }}
        />
      )}
      <Icon d={icono} />
      <span style={{flex: 1}}>{rotulo}</span>
      {punto && (
        <span
          aria-hidden
          style={{
            width: 7,
            height: 7,
            borderRadius: '50%',
            background: 'var(--warn)',
            flexShrink: 0,
          }}
        />
      )}
      {insignia && (
        <span
          style={{
            fontSize: 10.5,
            fontFamily: 'var(--font-mono)',
            padding: '2px 6px',
            borderRadius: 4,
            background: 'var(--bg-active)',
            color: 'var(--text-secondary)',
          }}>
          {insignia}
        </span>
      )}
    </button>
  );
}
