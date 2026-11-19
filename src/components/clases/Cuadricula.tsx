/**
 * La cuadrícula de alumnos de una clase, con su nota **para escribir** (H2).
 *
 * - Cada nota es un campo. Se escribe con coma o con punto; **Enter o ↓**
 *   la guardan y pasan al siguiente, **↑** al anterior, **Esc** deshace lo
 *   escrito. Salir del campo también la guarda.
 * - Lo que no es una nota (`12`, `siete`, `7.555`) **no se guarda ni se
 *   corrige**: se queda escrito en rojo, con el motivo al lado del nombre, y
 *   el foco no se mueve.
 * - La regla de color (D6): sin nota en gris, una nota es texto normal
 *   (también un suspenso), y solo lo que no es nota va en rojo.
 * - **El dictado** (H3b): la nota recién dictada, en verde con el
 *   micrófono, y se apaga sola; si el dictado duda, la nota propuesta va
 *   junto al nombre con **borde discontinuo** (provisional) y pulsarla la
 *   apunta. La fila dictada se trae a la vista.
 *
 * Lo que se está escribiendo vive en cada campo hasta que se guarda; la nota
 * guardada vive en Calificar, que es quien la manda al backend.
 */

import {useEffect, useRef, useState} from 'react';

import {ALTO_PANEL} from '@components/dictado/PanelDictado';
import {Icon, ICONOS} from '@components/ui/Icon';
import type {Alumno, Nota, Puestas} from '@modules/clases/tipos';
import {t} from '@modules/i18n';
import {leerEscrita, notaComoTexto, notaDe, type MotivoMal} from '@modules/notas/nota';
import styles from '@styles/cuadricula.module.css';

const COLUMNAS = '52px minmax(200px, 1fr) 96px';

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

  return (
    <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden', background: 'var(--bg-card)'}}>
      <div
        role="row"
        style={{
          display: 'grid',
          gridTemplateColumns: COLUMNAS,
          alignItems: 'center',
          height: 32,
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
      {alumnos.map((a, i) => {
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
            style={{
              // Que al traerla a la vista no quede debajo del panel del dictado.
              scrollMarginTop: `calc(var(${ALTO_PANEL}, 0px) + 8px)`,
              scrollMarginBottom: 24,
              display: 'grid',
              gridTemplateColumns: COLUMNAS,
              alignItems: 'center',
              minHeight: 'var(--row-h)',
              padding: '4px var(--row-pad)',
              borderBottom: i < alumnos.length - 1 ? '1px solid var(--border)' : 'none',
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
              nota={notaDe(a, puestas)}
              dictada={dictadas?.has(a.clave) ?? false}
              poner={(n) => poner(a.clave, n)}
              setMotivo={(m) => setMotivos((antes) => (antes[a.fila] === m ? antes : {...antes, [a.fila]: m}))}
              anterior={() => ir(i - 1)}
              siguiente={() => ir(i + 1)}
            />
          </div>
        );
      })}
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
  anterior: () => void;
  siguiente: () => void;
}

function CampoNota({campoRef, nombre, nota, dictada, poner, setMotivo, anterior, siguiente}: CampoNotaProps) {
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
        onFocus={(e) => e.currentTarget.select()}
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
          }
        }}
      />
    </span>
  );
}
