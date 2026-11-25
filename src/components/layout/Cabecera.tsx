/**
 * La cabecera de una pantalla: título, una línea de explicación y, a la
 * derecha, lo que la pantalla quiera (un contador, un botón).
 *
 * Es la de las pantallas de library-app (22 px, semibold, `-0.015em`), sacada
 * a un componente para que todas las pantallas la compartan en vez de
 * repetirla: con cinco pantallas es lo mismo, pero no se desalinean.
 *
 * Debajo, la primera vez que se entra en la pantalla, su aviso (H9): así no
 * lo tiene que pedir cada una.
 */

import {AvisoPrimeraVez} from '@components/ayuda/AvisoPrimeraVez';

/**
 * Tarjetas lado a lado cuando caben, y si no, una debajo de otra (HUI,
 * 2026-11-25): a 1280 van de una en una; a 1920, de dos en dos. Lo usan
 * Inicio, Conexión y Ajustes.
 */
export const DOS_COLUMNAS: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(min(540px, 100%), 1fr))',
  gap: 16,
  alignItems: 'start',
};

export interface CabeceraProps {
  /** Un texto o, si en la pantalla se elige qué se ve, un `Dropdown` con size="titulo". */
  titulo: React.ReactNode;
  intro?: React.ReactNode;
  derecha?: React.ReactNode;
}

export function Cabecera({titulo, intro, derecha}: CabeceraProps) {
  return (
    <>
      <header
        style={{
          display: 'flex',
          alignItems: 'flex-start',
          justifyContent: 'space-between',
          gap: 24,
          marginBottom: 20,
        }}>
        {/* Ocupa lo que deja `derecha`: un título que es un desplegable se
          corta con «…» solo cuando de verdad no cabe. */}
        <div style={{flex: 1, minWidth: 0}}>
          <h1
            style={{
              margin: 0,
              fontFamily: 'var(--font-display)',
              fontSize: 22,
              fontWeight: 600,
              color: 'var(--text-primary)',
              letterSpacing: '-0.015em',
            }}>
            {titulo}
          </h1>
          {intro && (
            <p style={{margin: '6px 0 0', fontSize: 13.5, color: 'var(--text-secondary)', maxWidth: 640}}>{intro}</p>
          )}
        </div>
        {derecha && <div style={{flexShrink: 0}}>{derecha}</div>}
      </header>
      <AvisoPrimeraVez />
    </>
  );
}
