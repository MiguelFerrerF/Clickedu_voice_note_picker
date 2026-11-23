/**
 * Las plantillas de Clickedu de la carpeta, una por clase (H1): su nombre
 * legible, cuántos alumnos y cuántos tienen nota. Pulsar una la abre en
 * Calificar.
 *
 * **Con los ítems de Clickedu** (H6, guardados al traer), arriba se elige
 * la evaluación (sale la de hoy) y cada clase, plegada, dice cuántos ítems
 * tiene en ella; desplegada, los ítems: la ponderación, lo puesto en la
 * app y lo que ya había en Clickedu. Pulsar un ítem abre Calificar en él; «Calificar sin ítem»,
 * como en la v1. Las evaluaciones se juntan por su nombre («2ª
 * evaluación»): su número en Clickedu cambia de una etapa a otra. Sin
 * ítems (quien no se conecta), como antes, y una línea dice dónde se traen.
 *
 * La lista se pone al día sola: si se añade, cambia o quita un Excel de la
 * carpeta, el backend avisa (`plantillas-cambiadas`).
 */

import {invoke} from '@tauri-apps/api/core';
import {useMemo, useState} from 'react';

import {Punto} from '@components/clases/Avisos';
import {BarraAvance} from '@components/clases/BarraAvance';
import {Ruta, SinPlantillas} from '@components/clases/SinPlantillas';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Icon, ICONOS} from '@components/ui/Icon';
import type {Clase, EvaluacionItems, ItemElegido} from '@modules/clases/tipos';
import {useClases} from '@modules/clases/useClases';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

/** La última, la `›` que dice que la fila se pulsa. */
const COLUMNAS = 'minmax(180px, 1fr) 80px 190px minmax(100px, 0.6fr) 20px';

/** Abre la clase en Calificar; con `item`, en ese ítem. */
type Abrir = (fichero: string, item?: ItemElegido) => void;

export interface ClasesProps {
  abrir: Abrir;
  irAAjustes: () => void;
}

/** Una evaluación del selector: la de varias clases con el mismo nombre. */
interface Evaluacion {
  clave: string;
  nombre: string;
  /** La de hoy en el mayor número de clases. */
  hoy: boolean;
}

/** Con qué se juntan las evaluaciones de clases distintas: su nombre. */
const claveDe = (nombre: string) => nombre.trim().toLowerCase().replace(/\s+/g, ' ');

/** Las evaluaciones de todas las clases, en el orden en que salen. */
function evaluacionesDe(clases: Clase[]): Evaluacion[] {
  const vistas = new Map<string, {nombre: string; enCurso: number}>();
  for (const c of clases)
    for (const e of c.items?.evaluaciones ?? []) {
      const v = vistas.get(claveDe(e.nombre)) ?? {nombre: e.nombre, enCurso: 0};
      v.enCurso += e.enCurso ? 1 : 0;
      vistas.set(claveDe(e.nombre), v);
    }
  const mas = Math.max(0, ...[...vistas.values()].map((v) => v.enCurso));
  return [...vistas].map(([clave, v]) => ({clave, nombre: v.nombre, hoy: mas > 0 && v.enCurso === mas}));
}

export function Clases({abrir, irAAjustes}: ClasesProps) {
  useIdioma();
  const {lista, error} = useClases({contarNotas: true});
  const hay = (lista?.clases.length ?? 0) > 0;
  const evaluaciones = useMemo(() => evaluacionesDe(lista?.clases ?? []), [lista]);
  // La elegida vive aquí, que la pantalla no se desmonta: se queda al
  // volver de Calificar. Sin elegir (o si ya no está), la de hoy.
  const [elegida, setElegida] = useState<string | null>(null);
  const actual = evaluaciones.find((e) => e.clave === elegida) ?? evaluaciones.find((e) => e.hoy) ?? evaluaciones[0];
  const traidos = Math.min(...(lista?.clases ?? []).flatMap((c) => (c.items ? [c.items.traidos] : [])));

  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera
        titulo={t('clases.titulo')}
        intro={t('clases.intro')}
        derecha={
          lista?.carpeta.existe && (
            <Button variant="ghost" onClick={() => invoke('abrir_carpeta_plantillas').catch(() => {})}>
              <Icon d={ICONOS.carpeta} size={14} />
              {t('clases.abrir_carpeta')}
            </Button>
          )
        }
      />
      {error && (
        <p role="alert" style={{margin: '0 0 16px', fontSize: 13, color: 'var(--danger)'}}>
          {error}
        </p>
      )}
      {lista && !hay && <SinPlantillas carpeta={lista.carpeta} irAAjustes={irAAjustes} />}
      {lista && hay && (
        <>
          <div style={{marginTop: -8, marginBottom: 14}}>
            <Ruta ruta={lista.carpeta.ruta} />
            {!actual && (
              <p style={{margin: '6px 0 0', fontSize: 12, color: 'var(--text-tertiary)', maxWidth: 640}}>
                {t('clases.sin_items')}
              </p>
            )}
          </div>
          {actual && <ElegirEvaluacion evaluaciones={evaluaciones} actual={actual} elegir={setElegida} />}
          <Tabla clases={lista.clases} evaluacion={actual?.clave ?? null} abrir={abrir} />
          {actual && Number.isFinite(traidos) && (
            <p style={{margin: '8px 0 0', fontSize: 11.5, color: 'var(--text-tertiary)'}}>
              {t('clases.items_traidos', {fecha: new Date(traidos * 1000).toLocaleDateString('es-ES')})}
            </p>
          )}
        </>
      )}
    </div>
  );
}

/** Los botones de las evaluaciones, uno elegido; la de hoy lo dice. */
function ElegirEvaluacion({
  evaluaciones,
  actual,
  elegir,
}: {
  evaluaciones: Evaluacion[];
  actual: Evaluacion;
  elegir: (clave: string) => void;
}) {
  return (
    <div
      role="radiogroup"
      aria-label={t('clases.evaluacion')}
      style={{
        display: 'inline-flex',
        flexWrap: 'wrap',
        gap: 2,
        padding: 2,
        marginBottom: 14,
        border: '1px solid var(--border)',
        borderRadius: 8,
        background: 'var(--bg-sunk)',
      }}>
      {evaluaciones.map((e) => {
        const es = e.clave === actual.clave;
        return (
          <button
            key={e.clave}
            type="button"
            role="radio"
            aria-checked={es}
            onClick={() => elegir(e.clave)}
            style={{
              font: 'inherit',
              height: 28,
              padding: '0 12px',
              border: 'none',
              borderRadius: 6,
              background: es ? 'var(--bg-card)' : 'transparent',
              boxShadow: es ? '0 0 0 1px var(--border)' : 'none',
              color: es ? 'var(--text-primary)' : 'var(--text-secondary)',
              fontSize: 12.5,
              fontWeight: es ? 600 : 400,
              cursor: 'pointer',
              whiteSpace: 'nowrap',
            }}>
            {e.nombre}
            {e.hoy && (
              <span style={{marginLeft: 6, fontSize: 11, fontWeight: 400, color: 'var(--text-tertiary)'}}>
                {t('clases.hoy')}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}

interface TablaProps {
  clases: Clase[];
  /** La clave de la evaluación elegida; `null` si ninguna clase tiene ítems. */
  evaluacion: string | null;
  abrir: Abrir;
}

/** Las clases; las pendientes de otros cursos, aparte y al final, como en Conexión. */
function Tabla({clases, evaluacion, abrir}: TablaProps) {
  const normales = clases.filter((c) => !c.pendiente);
  const pendientes = clases.filter((c) => c.pendiente);
  // Las desplegadas, por fichero: plegadas al principio; vive aquí, que la
  // pantalla no se desmonta, y se queda al cambiar de evaluación.
  const [abiertas, setAbiertas] = useState<Record<string, boolean>>({});
  const fila = (c: Clase, ultima: boolean) =>
    evaluacion !== null && c.items && c.lectura.tipo === 'bien' ? (
      <Bloque
        key={c.fichero}
        c={c}
        evaluacion={evaluacion}
        abrir={abrir}
        ultima={ultima}
        abierta={!!abiertas[c.fichero]}
        alternar={() => setAbiertas((m) => ({...m, [c.fichero]: !m[c.fichero]}))}
      />
    ) : (
      <Fila key={c.fichero} c={c} abrir={abrir} ultima={ultima} />
    );
  return (
    <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden', background: 'var(--bg-card)'}}>
      <Encabezado>
        <span>{t('clases.col_clase')}</span>
        <span style={{textAlign: 'right'}}>{t('clases.col_alumnos')}</span>
        <span style={{textAlign: 'right'}}>{t('clases.col_calificados')}</span>
        <span style={{paddingLeft: 24}}>{t('clases.col_estado')}</span>
      </Encabezado>
      {normales.map((c, i) => fila(c, i === normales.length - 1 && pendientes.length === 0))}
      {pendientes.length > 0 && (
        <>
          <Encabezado arriba={normales.length > 0}>
            <span>{t('clases.pendientes')}</span>
          </Encabezado>
          {pendientes.map((c, i) => fila(c, i === pendientes.length - 1))}
        </>
      )}
    </div>
  );
}

function Encabezado({children, arriba}: {children: React.ReactNode; arriba?: boolean}) {
  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: COLUMNAS,
        alignItems: 'center',
        height: 32,
        padding: '0 var(--row-pad)',
        background: 'var(--bg-sunk)',
        borderTop: arriba ? '1px solid var(--border)' : undefined,
        borderBottom: '1px solid var(--border)',
        fontSize: 10.5,
        fontWeight: 600,
        letterSpacing: 0.8,
        textTransform: 'uppercase',
        color: 'var(--text-tertiary)',
      }}>
      {children}
    </div>
  );
}

/** Una clase sin ítems (o que no se lee): se abre al pulsarla, sin ítem. */
function Fila({c, abrir, ultima}: {c: Clase; abrir: Abrir; ultima: boolean}) {
  return (
    <button
      type="button"
      onClick={() => abrir(c.fichero)}
      className="fila-clase"
      style={{...fila, minHeight: 48, padding: '6px var(--row-pad)', borderBottom: ultima ? 'none' : '1px solid var(--border)'}}>
      <span style={{minWidth: 0}}>
        <span style={{display: 'block', fontSize: 13.5, color: 'var(--text-primary)', ...cortar}}>{c.nombre}</span>
        <span
          style={{display: 'block', fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-tertiary)', ...cortar}}
          title={c.fichero}>
          {c.fichero}
        </span>
      </span>
      {c.lectura.tipo === 'bien' ? (
        <>
          <span style={cifra}>{c.lectura.alumnos}</span>
          <Avance n={c.lectura.calificados} total={c.lectura.alumnos} />
          <span style={{paddingLeft: 24, ...textoAvisos}}>
            <Avisos n={c.lectura.avisos} />
          </span>
        </>
      ) : (
        <span
          style={{gridColumn: '2 / 5', textAlign: 'right', fontSize: 12, color: 'var(--danger)', ...cortar}}
          title={c.lectura.error}>
          {t('clases.no_se_lee', {error: c.lectura.error})}
        </span>
      )}
      <Galon />
    </button>
  );
}

interface BloqueProps {
  c: Clase;
  evaluacion: string;
  abrir: Abrir;
  ultima: boolean;
  abierta: boolean;
  alternar: () => void;
}

/**
 * Una clase con ítems: su nombre, que la despliega, y dentro los ítems de
 * la evaluación elegida y «Calificar sin ítem». Cuántos ítems tiene va
 * junto al nombre, no en «Con nota», que es de las notas. Si en esa evaluación no tiene ítems (o no la tiene), una sola
 * fila, que abre la clase sin ítem.
 */
function Bloque({c, evaluacion, abrir, ultima, abierta, alternar}: BloqueProps) {
  if (c.lectura.tipo !== 'bien') return null;
  const {alumnos, calificados, avisos} = c.lectura;
  const e:EvaluacionItems | undefined = c.items?.evaluaciones.find((x) => claveDe(x.nombre) === evaluacion);
  const n = e?.items.length ?? 0;
  const raya = ultima ? 'none' : '1px solid var(--border)';
  const nombre = (
    <span style={{minWidth: 0, display: 'flex', alignItems: 'baseline', gap: 10}} title={c.fichero}>
      <span style={{fontSize: 13.5, fontWeight: 600, color: 'var(--text-primary)', ...cortar}}>{c.nombre}</span>
      {n > 0 && (
        <span style={{flexShrink: 0, fontSize: 12, color: 'var(--text-tertiary)', whiteSpace: 'nowrap'}}>
          {n === 1 ? t('clases.un_item') : t('clases.n_items', {n})}
        </span>
      )}
      {avisos > 0 && (
        <span style={{flexShrink: 0, ...textoAvisos}}>
          <Avisos n={avisos} />
        </span>
      )}
    </span>
  );

  if (!e || e.items.length === 0)
    return (
      <button
        type="button"
        onClick={() => abrir(c.fichero)}
        className="fila-clase"
        style={{...fila, minHeight: 44, padding: '6px var(--row-pad)', borderBottom: raya}}>
        {nombre}
        <span style={cifra}>{alumnos}</span>
        {calificados > 0 ? <Avance n={calificados} total={alumnos} /> : <span />}
        <span style={{paddingLeft: 24, fontSize: 12, color: 'var(--text-tertiary)', ...cortar}}>
          {e ? t('clases.sin_items_aun') : t('clases.sin_esta_evaluacion')}
        </span>
        <Galon />
      </button>
    );

  return (
    <div style={{borderBottom: raya, paddingBottom: abierta ? 4 : 0}}>
      <button
        type="button"
        onClick={alternar}
        aria-expanded={abierta}
        className="fila-clase"
        style={{...fila, minHeight: 44, padding: abierta ? '10px var(--row-pad) 2px' : '6px var(--row-pad)'}}>
        {nombre}
        <span style={cifra}>{alumnos}</span>
        <span />
        <span />
        <Galon abierto={abierta} />
      </button>
      {abierta && e.items.map((i) => (
        <button
          key={i.id}
          type="button"
          onClick={() => abrir(c.fichero, {evaluacion: e.id, item: i.id})}
          className="fila-clase"
          style={{...fila, minHeight: 34, padding: '2px var(--row-pad)'}}>
          <span style={{paddingLeft: 16, minWidth: 0, display: 'flex', alignItems: 'baseline', gap: 8}}>
            <span style={{fontSize: 13, color: 'var(--text-primary)', ...cortar}} title={i.nombre}>
              {i.nombre}
            </span>
            {i.ponderacion !== null && (
              <span style={{flexShrink: 0, fontFamily: 'var(--font-mono)', fontSize: 11.5, color: 'var(--text-tertiary)'}}>
                {t('clases.ponderacion', {n: i.ponderacion})}
              </span>
            )}
          </span>
          <span />
          <Avance
            n={i.calificados}
            total={alumnos}
            enClickedu={i.enClickedu}
            publicadas={i.publicadas}
            porSubir={i.porSubir}
          />
          <span />
          <Galon />
        </button>
      ))}
      {abierta && (
      <button
        type="button"
        onClick={() => abrir(c.fichero)}
        className="fila-clase"
        style={{...fila, minHeight: 30, padding: '2px var(--row-pad)'}}>
        <span style={{paddingLeft: 16, fontSize: 12, color: 'var(--text-tertiary)', ...cortar}}>{t('clases.sin_item')}</span>
        <span />
        {calificados > 0 ? <Avance n={calificados} total={alumnos} /> : <span />}
        <span />
        <Galon />
      </button>
      )}
    </div>
  );
}

/**
 * Cuántos tienen nota en la app, como barra; el número, al pasar por
 * encima. En un ítem (H6), a su izquierda y en gris, cuántos la tenían ya en
 * Clickedu al traer: es un dato, no un evento (`docs/02` §3). Va a la
 * izquierda para que las barras sigan en columna.
 */
function Avance({
  n,
  total,
  enClickedu = 0,
  publicadas = 0,
  porSubir,
}: {
  n: number;
  total: number;
  enClickedu?: number;
  /** Los publicados (en Clickedu, y en la app nada o lo mismo): la barra los sigue, en turquesa apagado. */
  publicadas?: number;
  /** En un ítem, lo que queda por subir: lo único en turquesa. Sin ítem, todas las notas. */
  porSubir?: number;
}) {
  const texto = t('clases.con_nota', {n, total});
  return (
    <span style={{display: 'flex', alignItems: 'center', justifyContent: 'flex-end', gap: 10, minWidth: 0}}>
      {enClickedu > 0 && (
        <span
          title={t('clases.en_clickedu_ayuda', {n: enClickedu, total})}
          style={{fontSize: 12, color: 'var(--text-tertiary)', ...cortar}}>
          {t('clases.en_clickedu', {n: enClickedu})}
        </span>
      )}
      <span role="img" aria-label={texto} title={texto} style={{display: 'flex', flexShrink: 0}}>
        <BarraAvance
          fraccion={total > 0 ? (porSubir ?? n) / total : 0}
          enClickedu={total > 0 ? publicadas / total : 0}
          ancho={72}
        />
      </span>
    </span>
  );
}

function Avisos({n}: {n: number}) {
  if (n === 0) return null;
  return (
    <>
      <Punto />
      {n === 1 ? t('clases.un_aviso') : t('clases.n_avisos', {n})}
    </>
  );
}

/**
 * La `›` de la derecha: dice que la fila se pulsa. En las que se despliegan,
 * gira hacia abajo al abrirlas.
 */
function Galon({abierto}: {abierto?: boolean}) {
  return (
    <Icon
      d={ICONOS.galonDerecha}
      size={14}
      style={{
        justifySelf: 'end',
        color: 'var(--text-tertiary)',
        transform: abierto ? 'rotate(90deg)' : undefined,
        transition: 'transform 150ms',
      }}
    />
  );
}

/** Una fila de la rejilla de la tabla, sin raya. */
const fila: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: COLUMNAS,
  alignItems: 'center',
  width: '100%',
  border: 'none',
  background: 'transparent',
  textAlign: 'left',
  font: 'inherit',
  cursor: 'pointer',
};

const cortar: React.CSSProperties = {overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'};
const textoAvisos: React.CSSProperties = {
  fontSize: 12,
  fontWeight: 400,
  color: 'var(--text-secondary)',
  display: 'flex',
  alignItems: 'center',
  gap: 8,
};
const cifra: React.CSSProperties = {
  textAlign: 'right',
  fontFamily: 'var(--font-mono)',
  fontSize: 13,
  color: 'var(--text-primary)',
};
