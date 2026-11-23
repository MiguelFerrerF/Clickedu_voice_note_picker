/**
 * Las plantillas de Clickedu de la carpeta, una por clase (H1): su nombre
 * legible, el fichero, cuántos alumnos y cuántos tienen nota. Pulsar una la
 * abre en Calificar.
 *
 * **Con los ítems de Clickedu** (H6, guardados al traer), la clase se
 * despliega: sus evaluaciones (la de hoy, abierta) y, en cada una, sus
 * ítems, con la ponderación, lo puesto en la app y si ya tenía notas en
 * Clickedu. Pulsar un ítem abre Calificar en él. Sin ítems (quien no se
 * conecta), como antes, y una línea dice dónde se traen.
 *
 * La lista se pone al día sola: si se añade, cambia o quita un Excel de la
 * carpeta, el backend avisa (`plantillas-cambiadas`).
 */

import {invoke} from '@tauri-apps/api/core';
import {useState} from 'react';

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
const COLUMNAS = 'minmax(220px, 1fr) 96px 120px minmax(120px, 0.6fr) 20px';

/** Abre la clase en Calificar; con `item`, en ese ítem. */
type Abrir = (fichero: string, item?: ItemElegido) => void;

export interface ClasesProps {
  abrir: Abrir;
  irAAjustes: () => void;
}

export function Clases({abrir, irAAjustes}: ClasesProps) {
  useIdioma();
  const {lista, error} = useClases({contarNotas: true});
  const hay = (lista?.clases.length ?? 0) > 0;
  const conItems = lista?.clases.some((c) => c.items) ?? false;

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
            {!conItems && (
              <p style={{margin: '6px 0 0', fontSize: 12, color: 'var(--text-tertiary)', maxWidth: 640}}>
                {t('clases.sin_items')}
              </p>
            )}
          </div>
          <Tabla clases={lista.clases} abrir={abrir} />
        </>
      )}
    </div>
  );
}

/** Las clases; las pendientes de otros cursos, aparte y al final, como en Conexión. */
function Tabla({clases, abrir}: {clases: Clase[]; abrir: Abrir}) {
  const normales = clases.filter((c) => !c.pendiente);
  const pendientes = clases.filter((c) => c.pendiente);
  // Lo desplegado, por fichero y por `fichero|evaluación`: vive aquí, que
  // la pantalla no se desmonta, y así se queda al volver de Calificar.
  const [abiertas, setAbiertas] = useState<Record<string, boolean>>({});
  const alternar = (k: string, ahora: boolean) => setAbiertas((m) => ({...m, [k]: !ahora}));
  const fila = (c: Clase, ultima: boolean) => (
    <Fila key={c.fichero} c={c} abrir={abrir} ultima={ultima} abiertas={abiertas} alternar={alternar} />
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

interface FilaProps {
  c: Clase;
  abrir: Abrir;
  ultima: boolean;
  abiertas: Record<string, boolean>;
  alternar: (k: string, ahora: boolean) => void;
}

/** Una clase: sin ítems, se abre al pulsarla; con ítems, se despliega. */
function Fila({c, abrir, ultima, abiertas, alternar}: FilaProps) {
  const lectura = c.lectura;
  const items = lectura.tipo === 'bien' ? c.items : null;
  const abierta = !!items && !!abiertas[c.fichero];
  const nItems = items?.evaluaciones.reduce((n, e) => n + e.items.length, 0) ?? 0;
  return (
    <>
      <button
        type="button"
        onClick={() => (items ? alternar(c.fichero, abierta) : abrir(c.fichero))}
        aria-expanded={items ? abierta : undefined}
        className="fila-clase"
        style={{
          display: 'grid',
          gridTemplateColumns: COLUMNAS,
          alignItems: 'center',
          width: '100%',
          minHeight: 48,
          padding: '6px var(--row-pad)',
          border: 'none',
          borderBottom: ultima && !abierta ? 'none' : '1px solid var(--border)',
          background: 'transparent',
          textAlign: 'left',
          font: 'inherit',
          cursor: 'pointer',
        }}>
        <span style={{minWidth: 0}}>
          <span style={{display: 'block', fontSize: 13.5, color: 'var(--text-primary)', ...cortar}}>{c.nombre}</span>
          <span
            style={{
              display: 'block',
              fontFamily: 'var(--font-mono)',
              fontSize: 11,
              color: 'var(--text-tertiary)',
              ...cortar,
            }}
            title={c.fichero}>
            {c.fichero}
          </span>
        </span>
        {c.lectura.tipo === 'bien' ? (
          <>
            <span style={cifra}>{c.lectura.alumnos}</span>
            {items ? (
              <span style={{textAlign: 'right', fontSize: 12, color: 'var(--text-tertiary)'}}>
                {nItemsEnPalabras(nItems)}
              </span>
            ) : (
              <span
                role="img"
                aria-label={t('clases.con_nota', {n: c.lectura.calificados, total: c.lectura.alumnos})}
                title={t('clases.con_nota', {n: c.lectura.calificados, total: c.lectura.alumnos})}
                style={{display: 'flex', justifyContent: 'flex-end'}}>
                <BarraAvance
                  fraccion={c.lectura.alumnos > 0 ? c.lectura.calificados / c.lectura.alumnos : 0}
                  ancho={72}
                />
              </span>
            )}
            <span
              style={{
                paddingLeft: 24,
                fontSize: 12,
                color: 'var(--text-secondary)',
                display: 'flex',
                alignItems: 'center',
                gap: 8,
              }}>
              {c.lectura.avisos > 0 && (
                <>
                  <Punto />
                  {c.lectura.avisos === 1 ? t('clases.un_aviso') : t('clases.n_avisos', {n: c.lectura.avisos})}
                </>
              )}
            </span>
          </>
        ) : (
          <span
            style={{gridColumn: '2 / 5', textAlign: 'right', fontSize: 12, color: 'var(--danger)', ...cortar}}
            title={c.lectura.error}>
            {t('clases.no_se_lee', {error: c.lectura.error})}
          </span>
        )}
        <Galon abierto={items ? abierta : undefined} />
      </button>
      {abierta && items && lectura.tipo === 'bien' && (
        <div style={{borderBottom: ultima ? 'none' : '1px solid var(--border)'}}>
          {items.evaluaciones.map((e) => (
            <Evaluacion
              key={e.id}
              fichero={c.fichero}
              e={e}
              alumnos={lectura.alumnos}
              abierta={abiertas[`${c.fichero}|${e.id}`] ?? e.enCurso}
              alternar={alternar}
              abrir={abrir}
            />
          ))}
          <p
            style={{margin: 0, padding: '8px var(--row-pad) 10px 40px', fontSize: 11.5, color: 'var(--text-tertiary)'}}>
            {t('clases.items_traidos', {fecha: new Date(items.traidos * 1000).toLocaleDateString('es-ES')})}
          </p>
        </div>
      )}
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

interface EvaluacionProps {
  fichero: string;
  e: EvaluacionItems;
  alumnos: number;
  abierta: boolean;
  alternar: (k: string, ahora: boolean) => void;
  abrir: Abrir;
}

/** Una evaluación de la clase desplegada: se pliega; dentro, sus ítems. */
function Evaluacion({fichero, e, alumnos, abierta, alternar, abrir}: EvaluacionProps) {
  return (
    <>
      <button
        type="button"
        onClick={() => alternar(`${fichero}|${e.id}`, abierta)}
        aria-expanded={abierta}
        className="fila-clase"
        style={{
          ...filaHija,
          minHeight: 34,
          fontSize: 11,
          fontWeight: 600,
          letterSpacing: 0.6,
          textTransform: 'uppercase',
          color: 'var(--text-secondary)',
        }}>
        <span style={{paddingLeft: 16, ...cortar}}>{e.nombre}</span>
        <span />
        <span
          style={{
            textAlign: 'right',
            fontWeight: 400,
            letterSpacing: 0,
            textTransform: 'none',
            color: 'var(--text-tertiary)',
          }}>
          {nItemsEnPalabras(e.items.length)}
        </span>
        <span />
        <Galon abierto={abierta} />
      </button>
      {abierta && e.items.length === 0 && (
        <p style={{margin: 0, padding: '4px var(--row-pad) 10px 44px', fontSize: 12, color: 'var(--text-tertiary)'}}>
          {t('clases.items_ninguno')}
        </p>
      )}
      {abierta &&
        e.items.map((i) => (
          <button
            key={i.id}
            type="button"
            onClick={() => abrir(fichero, {evaluacion: e.id, item: i.id})}
            className="fila-clase"
            style={{...filaHija, minHeight: 38}}>
            <span style={{paddingLeft: 28, minWidth: 0, display: 'flex', alignItems: 'baseline', gap: 8}}>
              <span style={{fontSize: 13, color: 'var(--text-primary)', ...cortar}} title={i.nombre}>
                {i.nombre}
              </span>
              {i.ponderacion !== null && (
                <span
                  style={{
                    flexShrink: 0,
                    fontFamily: 'var(--font-mono)',
                    fontSize: 11.5,
                    color: 'var(--text-tertiary)',
                  }}>
                  {t('clases.ponderacion', {n: i.ponderacion})}
                </span>
              )}
            </span>
            <span />
            <span
              role="img"
              aria-label={t('clases.con_nota', {n: i.calificados, total: alumnos})}
              title={t('clases.con_nota', {n: i.calificados, total: alumnos})}
              style={{display: 'flex', justifyContent: 'flex-end'}}>
              <BarraAvance fraccion={alumnos > 0 ? i.calificados / alumnos : 0} ancho={72} />
            </span>
            <span style={{paddingLeft: 24, fontSize: 12, color: 'var(--text-tertiary)', ...cortar}}>
              {i.enClickedu > 0 && t('clases.en_clickedu', {n: i.enClickedu})}
            </span>
            <Galon />
          </button>
        ))}
    </>
  );
}

/** Las filas de dentro de una clase desplegada: la rejilla de la tabla, sin raya. */
const filaHija: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: COLUMNAS,
  alignItems: 'center',
  width: '100%',
  padding: '2px var(--row-pad)',
  border: 'none',
  background: 'transparent',
  textAlign: 'left',
  font: 'inherit',
  cursor: 'pointer',
};

const cortar: React.CSSProperties = {overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'};
const cifra: React.CSSProperties = {
  textAlign: 'right',
  fontFamily: 'var(--font-mono)',
  fontSize: 13,
  color: 'var(--text-primary)',
};

function nItemsEnPalabras(n: number): string {
  if (n === 0) return t('clases.sin_items_n');
  return n === 1 ? t('clases.un_item') : t('clases.n_items', {n});
}
