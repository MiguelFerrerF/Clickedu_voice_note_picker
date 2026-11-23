/**
 * Cuántos alumnos tienen nota, como una barra fina: encima de la tabla de
 * Calificar y en la lista de Clases. Turquesa, que se permite para el
 * progreso (`docs/02` §3). Es un dibujo: quien la usa dice la cifra.
 *
 * En un ítem (H6), `enClickedu` es la parte de los que tienen nota solo en
 * Clickedu: sigue a la de la app, en el mismo turquesa apagado (mezclado
 * con el fondo de la barra, sin token nuevo). Es progreso que ya está
 * hecho, no un evento: un ítem ya subido entero sale con la barra llena.
 */

/** El turquesa apagado de lo que ya está en Clickedu. */
const APAGADO = 'color-mix(in oklab, var(--accent) 38%, var(--bg-sunk))';

export function BarraAvance({fraccion, ancho, enClickedu = 0}: {fraccion: number; ancho: number; enClickedu?: number}) {
  const a = acotar(fraccion);
  const c = acotar(enClickedu, 1 - a);
  const tramo = (desde: number, largo: number, color: string): React.CSSProperties => ({
    position: 'absolute',
    top: 0,
    bottom: 0,
    left: `${desde * 100}%`,
    width: `${largo * 100}%`,
    background: color,
    transition: 'left 150ms ease, width 150ms ease',
  });
  return (
    <span
      aria-hidden
      style={{
        position: 'relative',
        display: 'block',
        flexShrink: 0,
        width: ancho,
        height: 6,
        borderRadius: 3,
        background: 'var(--bg-sunk)',
        overflow: 'hidden',
      }}>
      <span style={tramo(a, c, APAGADO)} />
      <span style={{...tramo(0, a, 'var(--accent)'), borderRadius: 3}} />
    </span>
  );
}

function acotar(v: number, max = 1): number {
  return Math.min(max, Math.max(0, v));
}
