/**
 * Las gráficas de las Estadísticas (H8), sin librería: divs.
 *
 * Traídas **una a una** de library-app (`src/components/Graficas.tsx`): las
 * columnas del reparto de las notas, la barra partida de los aprobados y las
 * barras de lo dictado. Todas se comportan igual:
 *
 * - Al pasar el ratón (o el foco), la marca se **oscurece** y las demás se
 *   apagan; la lectura de debajo da su detalle.
 * - Al pulsarla se elige, si hay `alPulsar`: en las notas, para ver quiénes
 *   son.
 *
 * El color sigue la regla de `02-sistema-visual.md` §3: una sola serie en el
 * turquesa, su tono oscuro para lo señalado y el gris para lo demás. El
 * suspenso **no es rojo** (D6): va en gris.
 */
import {useState} from 'react';

export interface Dato {
  clave: string;
  rotulo: React.ReactNode;
  /** El nombre entero: para el `title` y para quien no ve la gráfica. */
  titulo: string;
  valor: number;
  /** La cifra que se escribe junto a la marca. */
  texto: string;
  /** Lo que dice la lectura con el ratón encima. */
  detalle?: string;
  /** Lo que va en gris: el suspenso (D6), lo que no es un acierto. */
  gris?: boolean;
  /** El rótulo del eje, en las columnas: solo en los valores redondos. */
  marca?: string;
  /** Otro color con significado propio. */
  color?: string;
  /** Se pulsa aunque valga 0 (un ítem sin notas lleva igual a lo suyo); sin él, solo si vale algo. */
  pulsable?: boolean;
  /** En las barras: una raya con la que comparar, como la media de la clase. */
  referencia?: number | null;
}

interface Comun {
  datos: Dato[];
  /** La lectura sin ratón encima: el máximo, o una pista. */
  lectura?: string;
  /** La clave del dato elegido en esta gráfica, si lo hay. */
  elegida?: string | null;
  alPulsar?: (d: Dato) => void;
}

const GRIS: React.CSSProperties = {color: 'var(--text-tertiary)'};
const APAGADA = 0.45;

/** Lo que tiene el ratón encima y los manejadores de cada marca. */
function useSobre({elegida, alPulsar}: Pick<Comun, 'elegida' | 'alPulsar'>) {
  const [sobre, setSobre] = useState<string | null>(null);
  const sePulsa = (d: Dato) => alPulsar !== undefined && (d.pulsable ?? d.valor > 0);
  const de = (d: Dato) => {
    const pulsable = sePulsa(d);
    return {
      onMouseEnter: () => setSobre(d.clave),
      onFocus: () => setSobre(d.clave),
      onBlur: () => setSobre(null),
      ...(pulsable && {
        role: 'button',
        tabIndex: 0,
        'aria-pressed': d.clave === elegida,
        'aria-label': d.detalle ?? `${d.titulo}: ${d.texto}`,
        onClick: () => alPulsar?.(d),
        onKeyDown: (e: React.KeyboardEvent) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            alPulsar?.(d);
          }
        },
      }),
    };
  };
  const cursor = (d: Dato): React.CSSProperties => ({
    cursor: sePulsa(d) ? 'pointer' : 'default',
    outline: 'none',
  });
  /** Señalada: con el ratón encima o elegida. Apagada: hay otra señalada por el ratón. */
  const señalada = (d: Dato) => d.clave === sobre || d.clave === elegida;
  const opacidad = (d: Dato) => (sobre !== null && d.clave !== sobre ? APAGADA : 1);
  /** El relleno de la marca: el turquesa, oscuro si está señalada; el gris, igual. */
  const relleno = (d: Dato) => {
    const s = señalada(d);
    if (d.color) return d.color;
    if (d.gris) return s ? 'var(--text-tertiary)' : 'var(--border-strong)';
    return s ? 'var(--accent-ink)' : 'var(--accent)';
  };
  /** Los colores con significado propio no tienen un tono oscuro: se oscurecen. */
  const filtro = (d: Dato) => (d.color && señalada(d) ? 'brightness(0.8)' : undefined);
  const lectura = (datos: Dato[], fija?: string) => {
    const d = datos.find((x) => x.clave === (sobre ?? elegida));
    return d ? (d.detalle ?? `${d.titulo}: ${d.texto}`) : (fija ?? '');
  };
  return {sobre, salir: () => setSobre(null), de, cursor, señalada, opacidad, relleno, filtro, lectura};
}

/** La línea bajo una gráfica: el detalle de lo que tiene el ratón encima. */
export function Lectura({texto}: {texto: string}) {
  return <div style={{minHeight: 17, fontSize: 11.5, lineHeight: 1.45, color: 'var(--text-secondary)'}}>{texto}</div>;
}

const TRANSICION = 'opacity 120ms, background-color 120ms, filter 120ms';

// ─────────────────────────────────────────────────────────────────────────────
// Barras: un reparto entre nombres, de más a menos
// ─────────────────────────────────────────────────────────────────────────────

/** `tope`: el valor de la barra entera (una nota, 10); sin él, el mayor de `datos`. */
export function Barras({datos, lectura, elegida, alPulsar, tope}: Comun & {tope?: number}) {
  const s = useSobre({elegida, alPulsar});
  const max = tope ?? Math.max(1, ...datos.map((d) => d.valor));
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 8}}>
      <div
        data-barras
        onMouseLeave={s.salir}
        style={{display: 'grid', gridTemplateColumns: 'minmax(0, 132px) minmax(40px, 1fr) auto', rowGap: 2}}>
        {datos.map((d) => {
          const fuerte = s.señalada(d);
          const tinta = fuerte ? 'var(--text-primary)' : 'var(--text-secondary)';
          return (
            <div
              key={d.clave}
              {...s.de(d)}
              title={d.titulo}
              style={{
                ...s.cursor(d),
                display: 'grid',
                gridColumn: '1 / -1',
                gridTemplateColumns: 'subgrid',
                columnGap: 12,
                alignItems: 'center',
                padding: '2px 0',
                opacity: s.opacidad(d),
                transition: TRANSICION,
              }}>
              <span
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  minWidth: 0,
                  fontSize: 12.5,
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  color: tinta,
                  fontWeight: d.clave === elegida ? 500 : 400,
                }}>
                {d.rotulo}
              </span>
              <span style={{position: 'relative', height: 14}}>
                <span
                  style={{display: 'block', height: '100%', background: 'var(--bg-sunk)', borderRadius: 4, overflow: 'hidden'}}>
                  <span
                    style={{
                      display: 'block',
                      height: '100%',
                      width: `${(d.valor / max) * 100}%`,
                      minWidth: d.valor > 0 ? 2 : 0,
                      background: s.relleno(d),
                      borderRadius: '0 4px 4px 0',
                      transition: TRANSICION,
                    }}
                  />
                </span>
                {d.referencia != null && (
                  <span
                    data-referencia
                    style={{
                      position: 'absolute',
                      top: 0,
                      bottom: 0,
                      left: `clamp(0px, calc(${Math.min(1, d.referencia / max) * 100}% - 1px), calc(100% - 2px))`,
                      width: 2,
                      background: 'var(--text-primary)',
                    }}
                  />
                )}
              </span>
              <span
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: 12,
                  textAlign: 'right',
                  fontVariantNumeric: 'tabular-nums',
                  color: tinta,
                }}>
                {d.texto}
              </span>
            </div>
          );
        })}
      </div>
      {lectura !== undefined && <Lectura texto={s.lectura(datos, lectura)} />}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Las series en el tiempo: columnas (histograma) y línea
// ─────────────────────────────────────────────────────────────────────────────

/** Los rótulos del eje de una serie, bajo su columna. */
function Eje({datos}: {datos: Dato[]}) {
  const n = datos.length;
  return (
    <div style={{position: 'relative', height: 16, marginTop: 4}}>
      {datos.map((d, i) =>
        d.marca ? (
          <span
            key={d.clave}
            style={{
              position: 'absolute',
              left: `${((i + 0.5) / n) * 100}%`,
              transform: 'translateX(-50%)',
              fontFamily: 'var(--font-mono)',
              fontSize: 10,
              ...GRIS,
            }}>
            {d.marca}
          </span>
        ) : null,
      )}
    </div>
  );
}

/** Columnas: un histograma. */
export function Columnas({datos, lectura, elegida, alPulsar, alto = 150}: Comun & {alto?: number}) {
  const s = useSobre({elegida, alPulsar});
  const max = Math.max(1, ...datos.map((d) => d.valor));
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 6}}>
      <div>
        <div
          data-columnas
          onMouseLeave={s.salir}
          style={{display: 'flex', alignItems: 'flex-end', height: alto, borderBottom: '1px solid var(--border)'}}>
          {datos.map((d) => (
            <div
              key={d.clave}
              {...s.de(d)}
              style={{
                ...s.cursor(d),
                flex: 1,
                minWidth: 0,
                height: '100%',
                padding: '0 1px',
                display: 'flex',
                alignItems: 'flex-end',
                justifyContent: 'center',
              }}>
              <div
                style={{
                  width: '100%',
                  maxWidth: 24,
                  height: d.valor > 0 ? `max(2px, ${(d.valor / max) * 100}%)` : 0,
                  background: s.relleno(d),
                  borderRadius: '3px 3px 0 0',
                  opacity: s.opacidad(d),
                  transition: TRANSICION,
                }}
              />
            </div>
          ))}
        </div>
        <Eje datos={datos} />
      </div>
      <Lectura texto={s.lectura(datos, lectura)} />
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Proporción: una barra partida, con su leyenda debajo
// ─────────────────────────────────────────────────────────────────────────────

/** Para pocas partes de un todo: los aprobados y los suspensos. */
export function Proporcion({datos, lectura, elegida, alPulsar}: Comun) {
  const s = useSobre({elegida, alPulsar});
  const total = datos.reduce((a, d) => a + d.valor, 0) || 1;
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 12}} onMouseLeave={s.salir}>
      <div data-proporcion style={{display: 'flex', gap: 2, height: 22}}>
        {datos
          .filter((d) => d.valor > 0)
          .map((d) => (
            <div
              key={d.clave}
              {...s.de(d)}
              title={`${d.titulo}: ${d.texto}`}
              style={{
                ...s.cursor(d),
                flex: `${d.valor} 0 0`,
                minWidth: 3,
                borderRadius: 4,
                background: s.relleno(d),
                filter: s.filtro(d),
                opacity: s.opacidad(d),
                transition: TRANSICION,
              }}
            />
          ))}
      </div>
      <div style={{display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) auto auto', rowGap: 2}}>
        {datos.map((d) => {
          const tinta = s.señalada(d) ? 'var(--text-primary)' : 'var(--text-secondary)';
          const num: React.CSSProperties = {
            fontFamily: 'var(--font-mono)',
            fontSize: 12,
            textAlign: 'right',
            fontVariantNumeric: 'tabular-nums',
          };
          return (
            <div
              key={d.clave}
              {...s.de(d)}
              style={{
                ...s.cursor(d),
                display: 'grid',
                gridColumn: '1 / -1',
                gridTemplateColumns: 'subgrid',
                columnGap: 14,
                alignItems: 'center',
                padding: '3px 0',
                opacity: d.valor > 0 ? s.opacidad(d) : 0.6,
                transition: TRANSICION,
              }}>
              <span style={{display: 'flex', alignItems: 'center', gap: 8, fontSize: 12.5, color: tinta}}>
                <span
                  style={{width: 10, height: 10, borderRadius: 3, background: s.relleno(d), filter: s.filtro(d), flexShrink: 0}}
                />
                {d.rotulo}
              </span>
              <span style={{...num, color: tinta}}>{d.texto}</span>
              <span style={{...num, ...GRIS, minWidth: 38}}>{Math.round((d.valor * 100) / total)} %</span>
            </div>
          );
        })}
      </div>
      <Lectura texto={s.lectura(datos, lectura)} />
    </div>
  );
}
