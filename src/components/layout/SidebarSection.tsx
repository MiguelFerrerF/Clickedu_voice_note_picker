/**
 * Grupo plegable de `NavItem`s que recuerda si estaba abierto.
 *
 * COPIADO de library-app (y esta, de la Quality Test App). El estado se guarda por sección en
 * `localStorage`; si el almacenamiento falla, la sección se abre y ya está —
 * no es un dato que merezca romper nada.
 */

import {useState} from 'react';

export interface SidebarSectionProps {
  id: string;
  titulo: string;
  abiertaPorDefecto?: boolean;
  children?: React.ReactNode;
}

export function SidebarSection({
  id,
  titulo,
  abiertaPorDefecto = true,
  children,
}: SidebarSectionProps) {
  const clave = `gestornotas-sidebar-${id}`;
  const [abierta, setAbierta] = useState<boolean>(() => {
    try {
      const v = localStorage.getItem(clave);
      return v == null ? abiertaPorDefecto : v === '1';
    } catch {
      return abiertaPorDefecto;
    }
  });

  const alternar = () =>
    setAbierta((a) => {
      try {
        localStorage.setItem(clave, a ? '0' : '1');
      } catch {
        /* sin persistencia, pero la sección se pliega igual */
      }
      return !a;
    });

  return (
    <div style={{marginTop: 16}}>
      <button
        type="button"
        onClick={alternar}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          width: '100%',
          padding: '6px 12px',
          marginBottom: 4,
          background: 'transparent',
          border: 'none',
          cursor: 'pointer',
          fontSize: 10.5,
          fontWeight: 600,
          letterSpacing: 1,
          textTransform: 'uppercase',
          color: 'var(--text-tertiary)',
          fontFamily: 'var(--font-body)',
          textAlign: 'left',
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.color = 'var(--text-secondary)';
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.color = 'var(--text-tertiary)';
        }}>
        <svg
          width="9"
          height="9"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          style={{
            transform: abierta ? 'rotate(90deg)' : 'rotate(0deg)',
            transition: 'transform 160ms',
            flexShrink: 0,
          }}
          aria-hidden="true">
          <path d="M9 6l6 6-6 6" />
        </svg>
        <span style={{flex: 1}}>{titulo}</span>
      </button>
      {abierta && (
        <nav style={{display: 'flex', flexDirection: 'column', gap: 2}}>{children}</nav>
      )}
    </div>
  );
}
