/**
 * El Inicio (HUI, 2026-11-25): donde abre la app. Lo pidió el usuario el
 * 2026-11-24; hasta entonces abría en Clases, que hacía de resumen a medias.
 * Cuatro bloques, de lo que se hace ahora a lo que se mira:
 *
 * - **Seguir donde lo dejaste**: la última clase (y su ítem) abierta en
 *   Calificar, que recuerda `ajustes.json` (`ultima_clase`), con su avance.
 * - **Lo pendiente con Clickedu**: los ítems con notas por subir, y cuándo
 *   se trajeron las plantillas; y, del refresco solo (H11), lo que ha
 *   cambiado en Clickedu y espera a traerse. Sin color: tener notas por subir es lo
 *   normal mientras se califica, no un error (D6).
 * - **Cómo van las clases**: una tarjeta por clase en la evaluación de hoy,
 *   con la barra de Clases.
 * - **El dictado**, en dos cifras de todas las clases; el detalle, en
 *   Estadísticas.
 *
 * Todo sale de `listar_clases` (que se pone al día con cada nota y con la
 * carpeta), salvo la última clase y el dictado, que se leen al volver aquí.
 */

import {invoke} from '@tauri-apps/api/core';
import {useEffect, useMemo, useState} from 'react';

import {BarraAvance} from '@components/clases/BarraAvance';
import {Cifra} from '@components/Cifra';
import {AvisoPorTraer} from '@components/clickedu/Refresco';
import {Cabecera, DOS_COLUMNAS} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Icon, ICONOS} from '@components/ui/Icon';
import {claveDe, evaluacionesDe} from '@modules/clases/evaluaciones';
import {buscarItem, type Clase, type ItemClickedu, type ItemElegido} from '@modules/clases/tipos';
import {useClases} from '@modules/clases/useClases';
import {useEstadoRefresco, type PorTraer} from '@modules/clickedu/refresco';
import {NUM, tiempo} from '@modules/estadisticas/formato';
import type {Dictados} from '@modules/estadisticas/tipos';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';
import type {PantallaId} from '@/types/pantalla';

/** Lo que devuelve `ultima_clase`: es `ajustes::Ultima`. */
interface Ultima {
  fichero: string;
  item: ItemElegido | null;
}

export interface InicioProps {
  /** La pantalla está a la vista: al volver, se leen otra vez la última clase y el dictado. */
  visible: boolean;
  ir: (p: PantallaId) => void;
  /** Abre la clase en Calificar; con `item`, en ese ítem. */
  abrir: (fichero: string, item?: ItemElegido | null) => void;
}

const GRIS: React.CSSProperties = {color: 'var(--text-tertiary)'};
const CORTAR: React.CSSProperties = {overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'};
const TEXTO: React.CSSProperties = {margin: 0, fontSize: 13, color: 'var(--text-secondary)'};

/** Cuántos alumnos tienen nota en un ítem: en la app o ya en Clickedu (como la barra de Clases). */
const conNota = (i: ItemClickedu) => i.porSubir + i.publicadas;

/** Los ítems por subir que se enseñan; el resto, «y N más», en Clases. */
const MAX_POR_SUBIR = 5;

export function Inicio({visible, ir, abrir}: InicioProps) {
  useIdioma();
  const {lista, error} = useClases({contarNotas: true});
  const [refresco] = useEstadoRefresco();
  const clases = useMemo(() => {
    const todas = lista?.clases ?? [];
    return [...todas.filter((c) => !c.pendiente), ...todas.filter((c) => c.pendiente)];
  }, [lista]);

  const [ultima, setUltima] = useState<Ultima | null>();
  const [dictado, setDictado] = useState<Dictados | null>();
  useEffect(() => {
    if (!visible) return;
    invoke<Ultima | null>('ultima_clase').then(setUltima, () => setUltima(null));
    invoke<Dictados>('resumen_dictado').then(setDictado, () => setDictado(null));
  }, [visible]);

  const evaluaciones = useMemo(() => evaluacionesDe(clases), [clases]);
  const hoy = evaluaciones.find((e) => e.hoy) ?? evaluaciones[0];

  return (
    <div style={{padding: '28px 32px'}}>
      <Cabecera titulo={t('inicio.titulo')} intro={t('inicio.intro')} />
      {error && (
        <p role="alert" style={{margin: '0 0 16px', fontSize: 13, color: 'var(--danger)'}}>
          {error}
        </p>
      )}
      <div style={{display: 'flex', flexDirection: 'column', gap: 16}}>
        <div style={DOS_COLUMNAS}>
          <Seguir clases={clases} ultima={ultima} lista={!!lista} ir={ir} abrir={abrir} />
          <Pendiente clases={clases} porTraer={refresco?.porTraer ?? []} ir={ir} abrir={abrir} />
        </div>

        {clases.length > 0 && (
          <section aria-label={t('inicio.clases')}>
            <Seccion titulo={t('inicio.clases')} detalle={hoy?.nombre} />
            <div style={{display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: 12}}>
              {clases.map((c) => (
                <TarjetaClase key={c.fichero} c={c} evaluacion={hoy?.clave ?? null} abrir={abrir} />
              ))}
            </div>
          </section>
        )}

        <section aria-label={t('inicio.dictado')}>
          <Seccion titulo={t('inicio.dictado')} />
          {dictado && dictado.porVoz > 0 ? (
            <div style={{display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 260px))', gap: 12}}>
              <Cifra valor={NUM.format(dictado.porVoz)} rotulo={t('inicio.dictadas')} pie={t('inicio.en_todas')} />
              <Cifra
                valor={dictado.ahorro === null ? '—' : tiempo(dictado.ahorro)}
                rotulo={t('inicio.ahorrado')}
                pie={dictado.ahorro === null ? t('inicio.ahorrado_nada') : t('inicio.en_todas')}
              />
            </div>
          ) : (
            dictado !== undefined && <p style={{...TEXTO, ...GRIS}}>{t('inicio.sin_dictar')}</p>
          )}
          <div style={{marginTop: 10}}>
            <Enlace onClick={() => ir('estadisticas')}>{t('inicio.ver_estadisticas')}</Enlace>
          </div>
        </section>
      </div>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Los bloques
// ─────────────────────────────────────────────────────────────────────────────

function Seguir({
  clases,
  ultima,
  lista,
  ir,
  abrir,
}: {
  clases: Clase[];
  ultima: Ultima | null | undefined;
  lista: boolean;
  ir: (p: PantallaId) => void;
  abrir: InicioProps['abrir'];
}) {
  // Hasta saber cuál es, nada: que no salte de «ninguna» a la clase.
  if (ultima === undefined || !lista) return <Card title={t('inicio.seguir')} />;
  const c = ultima && clases.find((x) => x.fichero.toLowerCase() === ultima.fichero.toLowerCase());
  if (!c || c.lectura.tipo !== 'bien')
    return (
      <Card title={t('inicio.seguir')}>
        <p style={TEXTO}>{clases.length > 0 ? t('inicio.seguir_nada') : t('inicio.sin_clases')}</p>
        <div>
          {clases.length > 0 ? (
            <Button onClick={() => ir('clases')}>{t('inicio.ver_clases')}</Button>
          ) : (
            <Button onClick={() => ir('clickedu')}>{t('inicio.traer_clases')}</Button>
          )}
        </div>
      </Card>
    );

  const elegido = buscarItem(c.items, ultima?.item);
  const total = c.lectura.alumnos;
  // Como en Calificar: lo puesto en la app, y aparte lo que ya estaba en Clickedu.
  const n = elegido ? elegido.item.calificados : c.lectura.calificados;
  const enClickedu = elegido?.item.enClickedu ?? 0;
  return (
    <Card title={t('inicio.seguir')}>
      <div style={{minWidth: 0}}>
        <div style={{fontSize: 15, fontWeight: 600, color: 'var(--text-primary)', ...CORTAR}} title={c.fichero}>
          {c.nombre}
        </div>
        <div style={{marginTop: 2, fontSize: 12.5, color: 'var(--text-secondary)', ...CORTAR}}>
          {elegido ? `${elegido.evaluacion.nombre} · ${elegido.item.nombre}` : t('inicio.sin_item')}
        </div>
      </div>
      <div style={{display: 'flex', alignItems: 'center', gap: 10}}>
        <BarraAvance
          fraccion={total > 0 ? (elegido ? elegido.item.porSubir : n) / total : 0}
          enClickedu={elegido && total > 0 ? elegido.item.publicadas / total : 0}
          ancho={160}
        />
        <span style={{fontSize: 12.5, color: 'var(--text-secondary)'}}>
          {t('inicio.con_nota', {n, total})}
          {enClickedu > 0 && ` · ${t('clases.en_clickedu', {n: enClickedu})}`}
        </span>
      </div>
      <div>
        <Button onClick={() => abrir(c.fichero, elegido ? ultima?.item : null)}>
          <Icon d={ICONOS.calificar} size={14} />
          {t('inicio.seguir_calificando')}
        </Button>
      </div>
    </Card>
  );
}

/** Un ítem con notas por subir. */
interface PorSubir {
  c: Clase;
  evaluacion: string;
  evaluacionNombre: string;
  item: ItemClickedu;
}

function Pendiente({
  clases,
  porTraer,
  ir,
  abrir,
}: {
  clases: Clase[];
  porTraer: PorTraer[];
  ir: (p: PantallaId) => void;
  abrir: InicioProps['abrir'];
}) {
  const conItems = clases.filter((c) => c.items);
  const porSubir: PorSubir[] = conItems.flatMap((c) =>
    (c.items?.evaluaciones ?? []).flatMap((e) =>
      e.items.filter((i) => i.porSubir > 0).map((item) => ({c, evaluacion: e.id, evaluacionNombre: e.nombre, item})),
    ),
  );
  const traidos = Math.min(...conItems.map((c) => c.items!.traidos));

  return (
    <Card title={t('inicio.pendiente')}>
      <AvisoPorTraer porTraer={porTraer} />
      {conItems.length === 0 ? (
        <p style={TEXTO}>{t('inicio.sin_traer')}</p>
      ) : porSubir.length === 0 ? (
        <p style={TEXTO}>{t('inicio.nada_por_subir')}</p>
      ) : (
        <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden'}}>
          {porSubir.slice(0, MAX_POR_SUBIR).map((p, i) => (
            <button
              key={`${p.c.fichero}|${p.item.id}`}
              type="button"
              className="fila-clase"
              onClick={() => abrir(p.c.fichero, {evaluacion: p.evaluacion, item: p.item.id})}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 12,
                width: '100%',
                minHeight: 40,
                padding: '6px 12px',
                border: 'none',
                borderTop: i > 0 ? '1px solid var(--border)' : 'none',
                background: 'transparent',
                textAlign: 'left',
                font: 'inherit',
                cursor: 'pointer',
              }}>
              <span style={{flex: 1, minWidth: 0}}>
                <span style={{display: 'block', fontSize: 13, color: 'var(--text-primary)', ...CORTAR}}>
                  {p.item.nombre}
                </span>
                <span style={{display: 'block', fontSize: 11.5, ...GRIS, ...CORTAR}}>
                  {`${p.c.nombre} · ${p.evaluacionNombre}`}
                </span>
              </span>
              <span style={{flexShrink: 0, fontSize: 12, color: 'var(--text-secondary)', whiteSpace: 'nowrap'}}>
                {p.item.porSubir === 1 ? t('inicio.una_por_subir') : t('inicio.n_por_subir', {n: p.item.porSubir})}
              </span>
              <span style={{flexShrink: 0, display: 'inline-flex', ...GRIS}}>
                <Icon d={ICONOS.galonDerecha} size={14} />
              </span>
            </button>
          ))}
          {porSubir.length > MAX_POR_SUBIR && (
            <button
              type="button"
              className="fila-clase"
              onClick={() => ir('clases')}
              style={{
                display: 'block',
                width: '100%',
                padding: '8px 12px',
                border: 'none',
                borderTop: '1px solid var(--border)',
                background: 'transparent',
                textAlign: 'left',
                font: 'inherit',
                fontSize: 12,
                color: 'var(--text-secondary)',
                cursor: 'pointer',
              }}>
              {t('inicio.y_mas', {n: porSubir.length - MAX_POR_SUBIR})}
            </button>
          )}
        </div>
      )}
      <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, flexWrap: 'wrap'}}>
        <span style={{fontSize: 12, ...GRIS}}>
          {Number.isFinite(traidos) &&
            t('inicio.traidas', {fecha: new Date(traidos * 1000).toLocaleDateString('es-ES')})}
        </span>
        <Button variant="ghost" onClick={() => ir('clickedu')}>
          {conItems.length === 0 ? t('inicio.ir_a_conexion') : t('inicio.traer_otra_vez')}
        </Button>
      </div>
    </Card>
  );
}

/**
 * Una clase en la evaluación de hoy: cuántos de sus ítems tienen todas las
 * notas y la barra de todas juntas. Sin ítems (quien no se conecta, o una
 * evaluación sin ítems), la barra de la clase, como en Clases.
 */
function TarjetaClase({c, evaluacion, abrir}: {c: Clase; evaluacion: string | null; abrir: InicioProps['abrir']}) {
  let linea: React.ReactNode;
  let barra: React.ReactNode = null;
  if (c.lectura.tipo !== 'bien') {
    linea = (
      <span style={{color: 'var(--danger)', ...CORTAR}} title={c.lectura.error}>
        {t('inicio.no_se_lee')}
      </span>
    );
  } else {
    const total = c.lectura.alumnos;
    const e = evaluacion !== null ? c.items?.evaluaciones.find((x) => claveDe(x.nombre) === evaluacion) : undefined;
    if (e && e.items.length > 0) {
      const completos = e.items.filter((i) => total > 0 && conNota(i) >= total).length;
      const huecos = e.items.length * total;
      const suma = (f: (i: ItemClickedu) => number) => e.items.reduce((s, i) => s + f(i), 0);
      linea =
        e.items.length === 1
          ? t(completos === 1 ? 'inicio.item_completo' : 'inicio.item_sin_completar')
          : t('inicio.items_completos', {n: completos, total: e.items.length});
      barra = (
        <BarraAvance
          fraccion={huecos > 0 ? suma((i) => i.porSubir) / huecos : 0}
          enClickedu={huecos > 0 ? suma((i) => i.publicadas) / huecos : 0}
          ancho={120}
        />
      );
    } else {
      linea = t('inicio.con_nota', {n: c.lectura.calificados, total});
      barra = <BarraAvance fraccion={total > 0 ? c.lectura.calificados / total : 0} ancho={120} />;
    }
  }

  return (
    <button
      type="button"
      className="fila-clase"
      onClick={() => abrir(c.fichero)}
      title={c.fichero}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
        minWidth: 0,
        padding: '12px 14px',
        border: '1px solid var(--border)',
        borderRadius: 8,
        background: 'var(--bg-card)',
        textAlign: 'left',
        font: 'inherit',
        cursor: 'pointer',
      }}>
      <span style={{width: '100%', fontSize: 13.5, fontWeight: 500, color: 'var(--text-primary)', ...CORTAR}}>
        {c.nombre}
      </span>
      <span
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          width: '100%',
          fontSize: 12,
          color: 'var(--text-secondary)',
        }}>
        {barra}
        <span style={{minWidth: 0, ...CORTAR}}>{linea}</span>
      </span>
    </button>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Las piezas
// ─────────────────────────────────────────────────────────────────────────────

function Seccion({titulo, detalle}: {titulo: string; detalle?: string}) {
  return (
    <h2 style={{margin: '8px 0 12px', fontSize: 14, fontWeight: 600, color: 'var(--text-primary)'}}>
      {titulo}
      {detalle && <span style={{fontWeight: 400, ...GRIS}}>{` · ${detalle}`}</span>}
    </h2>
  );
}

function Enlace({onClick, children}: {onClick: () => void; children: React.ReactNode}) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        all: 'unset',
        fontSize: 12.5,
        color: 'var(--text-secondary)',
        textDecoration: 'underline',
        textUnderlineOffset: 2,
        cursor: 'pointer',
      }}>
      {children}
    </button>
  );
}
