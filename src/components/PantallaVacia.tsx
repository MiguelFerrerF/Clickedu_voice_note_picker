/**
 * Una pantalla que todavía no hace nada (H0): su título, qué hará, y en qué
 * hito llega.
 *
 * Desaparece pantalla a pantalla según se construyen. El texto de qué hará es
 * el que acabará en la ayuda de esa pantalla (H9), así que se escribe como
 * para el profesor, no como nota de desarrollo.
 */

import {Cabecera} from '@components/layout/Cabecera';
import {Icon} from '@components/ui/Icon';
import {t} from '@modules/i18n';

export interface PantallaVaciaProps {
  titulo: string;
  descripcion: string;
  icono: string | readonly string[];
  /** El hito del plan en que llega: `H1`, `H4–H6`… */
  hito: string;
}

export function PantallaVacia({titulo, descripcion, icono, hito}: PantallaVaciaProps) {
  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera titulo={titulo} />
      <div
        style={{
          display: 'flex',
          gap: 16,
          alignItems: 'flex-start',
          padding: '20px 22px',
          background: 'var(--bg-card)',
          // Continuo: el discontinuo significa «provisional, falta que lo
          // confirmes» (el dictado dudoso) y no se gasta en otra cosa.
          border: '1px solid var(--border)',
          borderRadius: 10,
        }}>
        <span style={{color: 'var(--accent-ink)', flexShrink: 0, marginTop: 1}}>
          <Icon d={icono} size={22} />
        </span>
        <div style={{minWidth: 0}}>
          <p style={{margin: 0, fontSize: 13.5, lineHeight: 1.55, color: 'var(--text-secondary)'}}>
            {descripcion}
          </p>
          <span
            style={{
              display: 'inline-block',
              marginTop: 12,
              padding: '3px 8px',
              borderRadius: 4,
              background: 'var(--bg-sunk)',
              color: 'var(--text-tertiary)',
              fontFamily: 'var(--font-mono)',
              fontSize: 11,
            }}>
            {t('comun.llega_en', {hito})}
          </span>
        </div>
      </div>
    </div>
  );
}
