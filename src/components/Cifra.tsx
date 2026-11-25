/**
 * Una cifra grande con su rótulo, como las de library-app: la de las
 * Estadísticas (H8), sacada aquí para el Inicio (HUI).
 */

/** Una cifra, como las de library-app. La provisional, con borde discontinuo (D6). */
export function Cifra({
  valor,
  rotulo,
  pie,
  provisional,
}: {
  valor: string;
  rotulo: string;
  pie?: string;
  provisional?: boolean;
}) {
  return (
    <div
      data-provisional={provisional || undefined}
      style={{
        background: 'var(--bg-card)',
        border: provisional ? '1px dashed var(--border-strong)' : '1px solid var(--border)',
        borderRadius: 10,
        padding: '13px 15px',
        minWidth: 0,
      }}>
      <div
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: 21,
          fontWeight: 500,
          letterSpacing: '-0.02em',
          color: 'var(--text-primary)',
        }}>
        {valor}
      </div>
      <div
        style={{
          marginTop: 3,
          fontSize: 10.5,
          letterSpacing: '0.07em',
          textTransform: 'uppercase',
          color: 'var(--text-tertiary)',
        }}>
        {rotulo}
      </div>
      {pie && <div style={{marginTop: 2, fontSize: 11.5, color: 'var(--text-secondary)'}}>{pie}</div>}
    </div>
  );
}
