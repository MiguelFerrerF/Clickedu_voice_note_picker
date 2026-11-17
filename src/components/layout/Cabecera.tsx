/**
 * La cabecera de una pantalla: título, una línea de explicación y, a la
 * derecha, lo que la pantalla quiera (un contador, un botón).
 *
 * Es la de las pantallas de library-app (22 px, semibold, `-0.015em`), sacada
 * a un componente para que todas las pantallas la compartan en vez de
 * repetirla: con cinco pantallas es lo mismo, pero no se desalinean.
 */

export interface CabeceraProps {
  titulo: string;
  intro?: React.ReactNode;
  derecha?: React.ReactNode;
}

export function Cabecera({titulo, intro, derecha}: CabeceraProps) {
  return (
    <header
      style={{
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'space-between',
        gap: 24,
        marginBottom: 20,
      }}>
      <div style={{minWidth: 0}}>
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
          <p style={{margin: '6px 0 0', fontSize: 13.5, color: 'var(--text-secondary)', maxWidth: 640}}>
            {intro}
          </p>
        )}
      </div>
      {derecha && <div style={{flexShrink: 0}}>{derecha}</div>}
    </header>
  );
}
