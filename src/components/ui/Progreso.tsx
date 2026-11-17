/**
 * Barra de avance para cualquier proceso largo.
 *
 * COPIADA de la Quality Test App: el barrido indeterminado es el de
 * `screens/Airtable` (misma animación `progressIndeterminate` de
 * `globals.css`) y el grosor, el de `UpdateDialog`. Lo único propio es la
 * pista en `--bg-sunk`: con este papel, `--bg-input` sobre una tarjeta no se
 * distingue.
 *
 * Sin `total` (o con `total` 0) no se sabe cuánto falta, y la barra se mueve
 * sin prometer un porcentaje. **Regla de la app: nada largo ocurre sin una de
 * estas delante**, y nada largo corre en el hilo de la ventana.
 */

export interface ProgresoProps {
  /** Qué se está haciendo, en una línea. */
  rotulo: string;
  hecho?: number;
  total?: number;
}

const NUM = new Intl.NumberFormat('es-ES', {useGrouping: 'always'});

export function Progreso({rotulo, hecho = 0, total = 0}: ProgresoProps) {
  const pct = total > 0 ? Math.min(100, Math.round((hecho / total) * 100)) : null;

  return (
    <div role="progressbar" aria-valuenow={pct ?? undefined} aria-valuemin={0} aria-valuemax={100}>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          gap: 12,
          fontSize: 11,
          fontFamily: 'var(--font-mono)',
          color: 'var(--text-tertiary)',
          marginBottom: 6,
        }}>
        <span>{rotulo}</span>
        {pct != null && (
          <span>
            {NUM.format(hecho)} / {NUM.format(total)} · {pct}%
          </span>
        )}
      </div>
      <div
        style={{
          height: 6,
          background: 'var(--bg-sunk)',
          borderRadius: 3,
          overflow: 'hidden',
          position: 'relative',
        }}>
        {pct == null ? (
          <div
            style={{
              position: 'absolute',
              left: 0,
              top: 0,
              bottom: 0,
              width: '35%',
              borderRadius: 3,
              background: 'var(--accent)',
              animation: 'progressIndeterminate 1.4s ease-in-out infinite',
            }}
          />
        ) : (
          <div
            style={{
              width: `${pct}%`,
              height: '100%',
              borderRadius: 3,
              background: 'var(--accent)',
              transition: 'width 150ms ease',
            }}
          />
        )}
      </div>
    </div>
  );
}
