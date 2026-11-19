/**
 * La cuadrícula de alumnos de una clase, con su nota **para escribir** (H2).
 *
 * - Cada nota es un campo. Se escribe con coma o con punto; **Enter o ↓**
 *   la guardan y pasan al siguiente, **↑** al anterior. **Supr** vacía la
 *   celda y quita la nota al momento, se esté escribiendo o no: es la que
 *   sale en la ayuda. **Esc** devuelve lo que había antes de escribir (si
 *   se pisa una nota sin querer, es lo único que la recupera). Salir del
 *   campo también la guarda.
 * - Lo que no es una nota (`12`, `siete`, `7.555`) **no se guarda ni se
 *   corrige**: se queda escrito en rojo, con el motivo al lado del nombre, y
 *   el foco no se mueve.
 * - La regla de color (D6): sin nota en gris, una nota es texto normal
 *   (también un suspenso), y solo lo que no es nota va en rojo.
 * - **El dictado** (H3b): la nota recién dictada, en verde con el
 *   micrófono, y su fila entera teñida (HUI); se apagan solas. Si el
 *   dictado duda, la nota propuesta va junto al nombre con **borde
 *   discontinuo** (provisional) y pulsarla la apunta. La fila dictada se
 *   trae a la vista.
 *
 * - **Encima** (HUI), cuántos tienen nota, con una barra fina (turquesa:
 *   es progreso), «Siguiente sin nota» y el ⓘ que abre las teclas
 *   (`Teclado`): debajo de la tabla, con 30 alumnos, no se veían.
 *
 * Lo que se está escribiendo vive en cada campo hasta que se guarda; la nota
 * guardada vive en Calificar, que es quien la manda al backend.
 */

import {useEffect, useLayoutEffect, useRef, useState} from 'react';

import {BarraAvance} from '@components/clases/BarraAvance';
import {ID_TECLADO, Teclado} from '@components/clases/Teclado';
import {ALTO_PANEL} from '@components/dictado/PanelDictado';
import {Button} from '@components/ui/Button';
import {Icon, ICONOS} from '@components/ui/Icon';
import type {Alumno, Nota, Puestas} from '@modules/clases/tipos';
import {t} from '@modules/i18n';
import {leerEscrita, notaComoTexto, notaDe, type MotivoMal} from '@modules/notas/nota';
import styles from '@styles/cuadricula.module.css';

const COLUMNAS = '36px minmax(0, 1fr) 84px';

/**
 * Desde este ancho, los alumnos van en **dos columnas** (1 a la mitad a la
 * izquierda, el resto a la derecha): a 1280×800 se ve una clase de 30
 * entera sin bajar. Con menos, una sola.
 */
const ANCHO_DOS_COLUMNAS = 760;

/** Algo más baja que `--row-h` (36): caben más alumnos sin tocar la letra. */
const ALTO_FILA = 32;

export interface CuadriculaProps {
  alumnos: Alumno[];
  puestas: Puestas;
  /** Guarda la nota del alumno `clave`; `null` la quita. */
  poner: (clave: string, nota: number | null) => void;
  /** Las claves con una nota recién dictada: en verde mientras estén aquí. */
  dictadas?: ReadonlySet<string>;
  /** Lo que propone un dictado dudoso, por clave: espera que lo elijas. */
  propuestas?: Readonly<Record<string, number[]>>;
  elegir?: (clave: string, nota: number) => void;
  /** Trae a la vista la fila de `clave`; cada `vez` nueva, otra vez. */
  ver?: {clave: string; vez: number};
}

export function Cuadricula({alumnos, puestas, poner, dictadas, propuestas, elegir, ver}: CuadriculaProps) {
  const campos = useRef<(HTMLInputElement | null)[]>([]);
  const filas = useRef<Record<string, HTMLDivElement | null>>({});
  useEffect(() => {
    if (ver) filas.current[ver.clave]?.scrollIntoView({block: 'nearest', behavior: 'smooth'});
  }, [ver?.vez]);
  // El motivo por el que lo escrito no es nota, por fila: se pinta junto al nombre.
  const [motivos, setMotivos] = useState<Record<number, MotivoMal | undefined>>({});

  const ir = (i: number) => {
    const campo = campos.current[i];
    if (!campo) return;
    campo.focus();
    campo.select();
  };

  // El último campo tocado: «Siguiente sin nota» sigue desde él.
  const ultimo = useRef(-1);
  const notas = alumnos.map((a) => notaDe(a, puestas));
  const conNota = notas.filter((n) => n.tipo === 'valor').length;
  const sinNota = notas.map((n, i) => (n.tipo === 'vacia' ? i : -1)).filter((i) => i >= 0);

  /** Al siguiente sin nota tras el último tocado; del final, vuelve al principio. */
  function irAlSiguienteSinNota() {
    const i = sinNota.find((i) => i > ultimo.current) ?? sinNota[0];
    if (i === undefined) return;
    // La fila, no el campo, a la vista: así respeta el hueco del panel del dictado.
    filas.current[alumnos[i].clave]?.scrollIntoView({block: 'nearest', behavior: 'smooth'});
    campos.current[i]?.focus({preventScroll: true});
    campos.current[i]?.select();
  }

  // Una o dos columnas, según el ancho. Pantalla sin visitar (`display:
  // none`): mide 0, una; al enseñarla, el observador vuelve a medir.
  const caja = useRef<HTMLDivElement>(null);
  const [columnas, setColumnas] = useState(1);
  useLayoutEffect(() => {
    const el = caja.current;
    if (!el) return;
    const medir = () => setColumnas(el.clientWidth >= ANCHO_DOS_COLUMNAS ? 2 : 1);
    medir();
    const observador = new ResizeObserver(medir);
    observador.observe(el);
    return () => observador.disconnect();
  }, []);
  // Por columnas, en el orden de la lista: Enter y ↓ siguen al siguiente.
  const porColumna = Math.max(1, Math.ceil(alumnos.length / columnas));
  const grupos = Array.from({length: columnas}, (_, k) => k * porColumna).filter(
    (desde) => desde < alumnos.length || desde === 0,
  );

  return (
    <>
      <Avance
        conNota={conNota}
        total={alumnos.length}
        siguiente={sinNota.length > 0 ? irAlSiguienteSinNota : undefined}
      />
      <div
        ref={caja}
        style={{
          display: 'grid',
          gridTemplateColumns: `repeat(${grupos.length}, minmax(0, 1fr))`,
          border: '1px solid var(--border)',
          borderRadius: 8,
          overflow: 'hidden',
          background: 'var(--bg-card)',
        }}>
        {grupos.map((desde, k) => (
          <div
            key={desde}
            role="rowgroup"
            style={{minWidth: 0, borderLeft: k > 0 ? '1px solid var(--border)' : 'none'}}>
            <Encabezado />
            {alumnos.slice(desde, desde + porColumna).map((a, j, grupo) => fila(a, desde + j, j === grupo.length - 1))}
          </div>
        ))}
      </div>
    </>
  );

  function fila(a: Alumno, i: number, ultima: boolean) {
    const nombre = a.nombre.trim() ? a.nombre : t('cuadricula.sin_nombre', {id: a.id});
    const motivo = motivos[a.fila];
    const propuesta = propuestas?.[a.clave];
    return (
      <div
        key={a.fila}
        ref={(el) => {
          filas.current[a.clave] = el;
        }}
        role="row"
        className={dictadas?.has(a.clave) ? `${styles.fila} ${styles.filaDictada}` : styles.fila}
        style={{
          // Que al traerla a la vista no quede debajo del panel del dictado.
          scrollMarginTop: `calc(var(${ALTO_PANEL}, 0px) + 8px)`,
          scrollMarginBottom: 24,
          display: 'grid',
          gridTemplateColumns: COLUMNAS,
          alignItems: 'center',
          minHeight: ALTO_FILA,
          padding: '2px var(--row-pad)',
          borderBottom: ultima ? 'none' : '1px solid var(--border)',
        }}>
        <span style={{fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-tertiary)'}}>{i + 1}</span>
        <span style={{display: 'flex', alignItems: 'baseline', gap: 10, minWidth: 0, paddingRight: 12}}>
          <span
            style={{
              fontSize: 13,
              color: a.nombre.trim() ? 'var(--text-primary)' : 'var(--text-tertiary)',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              minWidth: 0,
            }}
            title={a.nombre.trim() || undefined}>
            {nombre}
          </span>
          {(motivo || propuesta) && (
            <span style={{marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 6, flexShrink: 0}}>
              {propuesta?.map((n) => (
                <button
                  key={n}
                  type="button"
                  className={styles.propuesta}
                  onClick={() => elegir?.(a.clave, n)}
                  title={t('cuadricula.apuntar_propuesta', {nota: String(n), nombre})}>
                  {t('cuadricula.propuesta', {nota: String(n)})}
                </button>
              ))}
              {motivo && (
                <span role="alert" style={{fontSize: 12, color: 'var(--danger)'}}>
                  {t(`cuadricula.mal_${motivo}`)}
                </span>
              )}
            </span>
          )}
        </span>
        <CampoNota
          campoRef={(el) => {
            campos.current[i] = el;
          }}
          nombre={nombre}
          nota={notas[i]}
          dictada={dictadas?.has(a.clave) ?? false}
          poner={(n) => poner(a.clave, n)}
          alEnfocar={() => (ultimo.current = i)}
          setMotivo={(m) => setMotivos((antes) => (antes[a.fila] === m ? antes : {...antes, [a.fila]: m}))}
          anterior={() => ir(i - 1)}
          siguiente={() => ir(i + 1)}
        />
      </div>
    );
  }
}

/**
 * Encima de la tabla (HUI): cuántos tienen nota, con una barra fina en
 * `--accent` (el color del progreso, `docs/02`), el botón que lleva al
 * siguiente sin nota y, a la derecha, el ⓘ que abre las teclas. Sin
 * `siguiente`, no queda ninguno sin nota.
 */
function Avance({conNota, total, siguiente}: {conNota: number; total: number; siguiente?: () => void}) {
  const fraccion = total > 0 ? conNota / total : 0;
  const [teclado, setTeclado] = useState(false);
  return (
    <>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: '8px 16px',
          marginBottom: 10,
        }}>
        <span style={{display: 'inline-flex', alignItems: 'center', gap: 16}}>
          <span style={{display: 'inline-flex', alignItems: 'center', gap: 10}}>
            <BarraAvance fraccion={fraccion} ancho={120} />
            <span style={{fontSize: 13, color: 'var(--text-secondary)', whiteSpace: 'nowrap'}}>
              {t('cuadricula.con_nota', {n: conNota, total})}
            </span>
          </span>
          <Button variant="ghost" onClick={siguiente} disabled={!siguiente} title={t('cuadricula.siguiente_ayuda')}>
            {t('cuadricula.siguiente')}
            <Icon d={ICONOS.galonAbajo} size={14} />
          </Button>
        </span>
        <button
          type="button"
          onClick={() => setTeclado((a) => !a)}
          aria-expanded={teclado}
          aria-controls={ID_TECLADO}
          aria-label={t('teclado.titulo')}
          title={t('teclado.titulo')}
          style={{
            display: 'inline-flex',
            padding: 6,
            border: 'none',
            borderRadius: 6,
            background: teclado ? 'var(--bg-sunk)' : 'none',
            color: teclado ? 'var(--text-primary)' : 'var(--text-secondary)',
            cursor: 'pointer',
          }}>
          <Icon d={ICONOS.info} size={16} />
        </button>
      </div>
      {teclado && <Teclado cerrar={() => setTeclado(false)} />}
    </>
  );
}

/** Los rótulos de una columna de alumnos. */
function Encabezado() {
  return (
    <div
      role="row"
      style={{
        display: 'grid',
        gridTemplateColumns: COLUMNAS,
        alignItems: 'center',
        height: 28,
        padding: '0 var(--row-pad)',
        background: 'var(--bg-sunk)',
        borderBottom: '1px solid var(--border)',
        fontSize: 10.5,
        fontWeight: 600,
        letterSpacing: 0.8,
        textTransform: 'uppercase',
        color: 'var(--text-tertiary)',
      }}>
      <span>{t('cuadricula.n')}</span>
      <span>{t('cuadricula.alumno')}</span>
      <span style={{textAlign: 'right', paddingRight: 8}}>{t('cuadricula.nota')}</span>
    </div>
  );
}

interface CampoNotaProps {
  campoRef: (el: HTMLInputElement | null) => void;
  nombre: string;
  /** La nota que vale ahora. */
  nota: Nota;
  /** Recién dictada: en verde, con el micrófono. */
  dictada: boolean;
  poner: (nota: number | null) => void;
  setMotivo: (motivo: MotivoMal | undefined) => void;
  alEnfocar: () => void;
  anterior: () => void;
  siguiente: () => void;
}

function CampoNota({
  campoRef,
  nombre,
  nota,
  dictada,
  poner,
  setMotivo,
  alEnfocar,
  anterior,
  siguiente,
}: CampoNotaProps) {
  // `null`: no se está escribiendo y se enseña la nota que vale. Va también
  // en una ref porque Enter guarda y mueve el foco, y el `blur` que eso
  // provoca llega antes de que React pinte: sin la ref se guardaría dos veces.
  const [borrador, setBorradorEstado] = useState<string | null>(null);
  const borradorRef = useRef<string | null>(null);
  const setBorrador = (b: string | null) => {
    borradorRef.current = b;
    setBorradorEstado(b);
  };
  const mal = borrador !== null ? leerEscrita(borrador).tipo === 'mal' : nota.tipo === 'invalida';

  /** Guarda lo escrito si es una nota. `false` si no lo es: se queda. */
  function guardar(): boolean {
    if (borradorRef.current === null) return true;
    const escrita = leerEscrita(borradorRef.current);
    if (escrita.tipo === 'mal') {
      setMotivo(escrita.motivo);
      return false;
    }
    const nueva = escrita.tipo === 'valor' ? escrita.valor : null;
    const antes = nota.tipo === 'valor' ? nota.valor : nota.tipo === 'vacia' ? null : undefined;
    setBorrador(null);
    setMotivo(undefined);
    if (nueva !== antes) poner(nueva);
    return true;
  }

  const verde = dictada && borrador === null && !mal;
  return (
    <span style={{position: 'relative', display: 'block', minWidth: 0}}>
      <Icon d={ICONOS.microfono} size={12} className={styles.microfono} style={{opacity: verde ? 1 : 0}} />
      <input
        ref={campoRef}
        type="text"
        inputMode="decimal"
        autoComplete="off"
        spellCheck={false}
        className={[styles.nota, verde ? styles.dictada : '', mal ? styles.mal : ''].filter(Boolean).join(' ')}
        value={borrador ?? notaComoTexto(nota)}
        placeholder="—"
        aria-label={t('cuadricula.nota_de', {nombre})}
        aria-invalid={mal || undefined}
        title={nota.tipo === 'invalida' && borrador === null ? t('cuadricula.no_es_nota') : undefined}
        onFocus={(e) => {
          alEnfocar();
          e.currentTarget.select();
        }}
        onChange={(e) => {
          setBorrador(e.target.value);
          setMotivo(undefined);
        }}
        onBlur={guardar}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === 'ArrowDown') {
            e.preventDefault();
            if (guardar()) siguiente();
          } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            if (guardar()) anterior();
          } else if (e.key === 'Escape') {
            setBorrador(null);
            setMotivo(undefined);
          } else if (e.key === 'Delete') {
            // Vacía la celda, también lo escrito a medias, y quita la nota.
            e.preventDefault();
            setBorrador(null);
            setMotivo(undefined);
            if (nota.tipo !== 'vacia') poner(null);
          }
        }}
      />
    </span>
  );
}
