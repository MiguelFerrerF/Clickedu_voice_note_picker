/**
 * Cuántos alumnos tienen nota, como una barra fina: encima de la tabla de
 * Calificar y en la lista de Clases. Turquesa, que se permite para el
 * progreso (`docs/02` §3). Es un dibujo: quien la usa dice la cifra.
 */

export function BarraAvance({fraccion, ancho}: {fraccion: number; ancho: number}) {
  return (
    <span
      aria-hidden
      style={{
        display: 'block',
        flexShrink: 0,
        width: ancho,
        height: 6,
        borderRadius: 3,
        background: 'var(--bg-sunk)',
        overflow: 'hidden',
      }}>
      <span
        style={{
          display: 'block',
          height: '100%',
          borderRadius: 3,
          background: 'var(--accent)',
          transformOrigin: 'left',
          transform: `scaleX(${Math.min(1, Math.max(0, fraccion))})`,
          transition: 'transform 150ms ease',
        }}
      />
    </span>
  );
}
