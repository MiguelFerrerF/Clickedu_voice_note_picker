/**
 * «Plantillas de Clickedu», bajo «Tu cuenta» en Conexión (H5, `docs/01`).
 *
 * - **Traer** baja una plantilla por materia, con avance por materias. Sin
 *   sesión, el backend entra solo con la cuenta guardada.
 * - **La primera vez, o si hay dudas, se enseña antes** qué pasa con cada
 *   materia y con lo que ya hay en la carpeta (a «antiguas», y a dónde
 *   pasan sus notas), y se confirma con «Hacerlo». Cada vieja con notas
 *   tiene su desplegable: a qué nueva pasan, **a ninguna** o, si era una
 *   duda, sin decidir (se queda en la carpeta). Cambiar una respuesta
 *   vuelve a planear en el backend: lo que se enseña es lo que se hará.
 * - **Hecho**, el resumen en verde que se apaga (evento, D6); los fallos,
 *   en rojo y con qué hacer. La lista de materias se queda, en gris.
 * - Lo leído «degradado» y lo que ha cambiado en las páginas va plegado,
 *   con el punto ámbar: es el aviso temprano de que Clickedu cambia.
 * - Las pendientes de otros cursos, aparte y al final.
 */

import {Channel, invoke} from '@tauri-apps/api/core';
import {useEffect, useRef, useState} from 'react';

import {Plegados, Punto} from '@components/clases/Avisos';
import {mensaje} from '@components/clickedu/TarjetaCuenta';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Dropdown} from '@components/ui/Dropdown';
import {Icon, ICONOS} from '@components/ui/Icon';
import {Progreso} from '@components/ui/Progreso';
import type {
  Apartada,
  AvanceTraer,
  Duda,
  FalloAplicar,
  FalloTraer,
  Hecho,
  Plan,
  PlanMateria,
  Respuesta,
  Traer,
} from '@modules/clickedu/tipos';
import {t} from '@modules/i18n';

const APAGAR_MS = 6000;
const COLUMNAS = 'minmax(200px, 1fr) 72px minmax(180px, 0.9fr)';
const SIN_DECIDIR = 'sin_decidir';
const NINGUNA = 'ninguna';

/** `hayCuenta`: sin cuenta guardada no se puede traer (`undefined`: aún no se sabe). */
export function TarjetaTraer({hayCuenta}: {hayCuenta?: boolean}) {
  const [trayendo, setTrayendo] = useState(false);
  const [avance, setAvance] = useState<AvanceTraer>();
  const [traido, setTraido] = useState<Traer>();
  const [fallo, setFallo] = useState<FalloTraer | string>();
  const [respuestas, setRespuestas] = useState<Record<string, Respuesta>>({});
  /** Las que eran duda al traer: siguen pudiendo quedarse sin decidir. */
  const [dudosas, setDudosas] = useState<string[]>([]);
  const [confirmando, setConfirmando] = useState(false);
  const [encendido, setEncendido] = useState(false);
  /** Para no pintar un replaneo que ha llegado tarde. */
  const ultimo = useRef(0);

  useEffect(() => {
    if (!encendido) return;
    const apagar = setTimeout(() => setEncendido(false), APAGAR_MS);
    return () => clearTimeout(apagar);
  }, [encendido]);

  async function traer() {
    setFallo(undefined);
    setTraido(undefined);
    setRespuestas({});
    setAvance(undefined);
    setTrayendo(true);
    const canal = new Channel<AvanceTraer>();
    canal.onmessage = setAvance;
    try {
      const r = await invoke<Traer>('traer_plantillas', {avance: canal});
      setTraido(r);
      setDudosas(r.plan.dudas.map((d) => d.fichero));
      if (r.hecho) setEncendido(true);
    } catch (e) {
      setFallo(e as FalloTraer | string);
    } finally {
      setTrayendo(false);
    }
  }

  function responder(fichero: string, valor: string) {
    const nuevas = {...respuestas};
    if (valor === SIN_DECIDIR) delete nuevas[fichero];
    else nuevas[fichero] = valor === NINGUNA ? {tipo: 'ninguna'} : {tipo: 'pasar', a: valor.slice('pasar:'.length)};
    setRespuestas(nuevas);
    const yo = ++ultimo.current;
    invoke<Plan>('replanear_traer', {respuestas: nuevas}).then(
      (plan) => yo === ultimo.current && setTraido((r) => r && {...r, plan}),
      (e) => setFallo(String(e)),
    );
  }

  async function confirmar() {
    setFallo(undefined);
    setConfirmando(true);
    try {
      const r = await invoke<Traer>('confirmar_traer', {respuestas});
      // Los avisos de la lectura son los de cuando se trajo.
      setTraido((antes) => ({...r, estadoMaterias: antes?.estadoMaterias ?? null, cambios: antes?.cambios ?? []}));
      setEncendido(true);
    } catch (e) {
      setFallo(String(e));
    } finally {
      setConfirmando(false);
    }
  }

  function cancelar() {
    setFallo(undefined);
    setTraido(undefined);
    invoke('cancelar_traer').catch((e) => setFallo(String(e)));
  }

  const esperando = traido !== undefined && traido.hecho === null;

  return (
    <Card
      title={t('traer.titulo')}
      subtitle={t('traer.sub')}
      right={
        !esperando && (
          <Button
            variant={traido?.hecho || !hayCuenta ? 'ghost' : 'primary'}
            onClick={traer}
            disabled={trayendo || !hayCuenta}
            title={hayCuenta === false ? t('traer.fallo_sin_entrar') : undefined}>
            {t('traer.traer')}
          </Button>
        )
      }>
      {trayendo && (
        <Progreso
          rotulo={avance?.total ? t('traer.bajando') : t('traer.preguntando')}
          hecho={avance?.hechas}
          total={avance?.total}
        />
      )}

      {traido && esperando && (
        <p style={{margin: 0, fontSize: 13, lineHeight: 1.5, color: 'var(--text-primary)', maxWidth: 720}}>
          {traido.primeraVez ? t('traer.primera_vez') : t('traer.con_dudas')}
        </p>
      )}
      {traido?.hecho && <Resumen hecho={traido.hecho} encendido={encendido} />}
      {traido && <AvisosLectura traido={traido} />}
      {traido && <Materias materias={traido.plan.materias} hecho={traido.hecho} />}
      {traido && (
        <Carpeta
          plan={traido.plan}
          hecho={traido.hecho}
          respuestas={respuestas}
          dudosas={dudosas}
          responder={esperando && !confirmando ? responder : undefined}
        />
      )}

      {traido && esperando && (
        <div style={{display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: 8}}>
          <Button onClick={confirmar} disabled={confirmando}>
            {confirmando ? t('traer.haciendo') : t('traer.hacer')}
          </Button>
          <Button variant="ghost" onClick={cancelar} disabled={confirmando}>
            {t('clickedu.cancelar')}
          </Button>
          {traido.plan.dudas.length > 0 && (
            <span style={{fontSize: 12.5, color: 'var(--text-secondary)'}}>
              {traido.plan.dudas.length === 1
                ? t('traer.sin_decidir_uno')
                : t('traer.sin_decidir_n', {n: traido.plan.dudas.length})}
            </span>
          )}
        </div>
      )}

      {fallo !== undefined && (
        <p role="alert" style={{margin: 0, fontSize: 13, lineHeight: 1.5, color: 'var(--danger)', maxWidth: 720}}>
          {mensajeTraer(fallo)}
        </p>
      )}
    </Card>
  );
}

function mensajeTraer(fallo: FalloTraer | string): string {
  if (typeof fallo === 'string') return fallo;
  switch (fallo.tipo) {
    case 'conexion':
      return mensaje(fallo.fallo);
    case 'materias':
      return t('traer.fallo_materias', {motivo: fallo.roto.motivo});
    case 'sin_entrar':
      return t('traer.fallo_sin_entrar');
    case 'otro':
      return fallo.detalle;
  }
}

/** Uno o varios: la clave `_uno` o la `_n` (o la que se dé) con `{n}`. */
function contar(n: number, uno: string, varios: string, vars: Record<string, string | number> = {}) {
  return n === 1 ? t(uno, vars) : t(varios, {...vars, n});
}

// ───────────────────────── Hecho ─────────────────────────

/** Lo hecho, en verde mientras está encendido; los fallos, en rojo. */
function Resumen({hecho, encendido}: {hecho: Hecho; encendido: boolean}) {
  const partes = [
    hecho.escritas.length > 0 && contar(hecho.escritas.length, 'traer.hecho_escritas_uno', 'traer.hecho_escritas'),
    hecho.movidas.length > 0 && contar(hecho.movidas.length, 'traer.hecho_movidas_uno', 'traer.hecho_movidas'),
    hecho.pasan > 0 && contar(hecho.pasan, 'traer.hecho_pasan_uno', 'traer.hecho_pasan'),
  ].filter((p): p is string => typeof p === 'string');
  const limpio = hecho.fallos.length === 0;
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 6}}>
      <p
        role="status"
        style={{
          margin: 0,
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          fontSize: 13,
          color: encendido && limpio ? 'var(--success)' : 'var(--text-secondary)',
          transition: 'color 600ms',
        }}>
        <Icon d={ICONOS.comprobado} size={14} style={{flexShrink: 0}} />
        {partes.length > 0 ? partes.join(' · ') : t('traer.hecho_nada')}
      </p>
      {hecho.fallos.map((f, i) => (
        <p
          key={i}
          role="alert"
          style={{margin: 0, fontSize: 13, lineHeight: 1.5, color: 'var(--danger)', maxWidth: 720}}>
          {falloAplicar(f)}
        </p>
      ))}
    </div>
  );
}

function falloAplicar(f: FalloAplicar): string {
  switch (f.tipo) {
    case 'mover':
      return t('traer.fallo_mover', {fichero: f.fichero});
    case 'escribir':
      return f.error
        ? t('traer.fallo_escribir_error', {fichero: f.fichero, error: f.error})
        : t('traer.fallo_escribir', {fichero: f.fichero});
    case 'notas':
      return t('traer.fallo_notas', {fichero: f.fichero, a: f.a});
    case 'guardar':
      return t('traer.fallo_guardar', {error: f.error});
  }
}

// ───────────────────────── Avisos ─────────────────────────

/** Lo leído de otra manera y lo que ha cambiado: el aviso temprano. */
function AvisosLectura({traido}: {traido: Traer}) {
  const textos: string[] = [];
  if (traido.estadoMaterias?.estado === 'degradado')
    textos.push(t('traer.aviso_materias', {motivo: traido.estadoMaterias.motivo}));
  for (const m of traido.plan.materias) {
    if (m.estado_evaluaciones?.estado === 'degradado')
      textos.push(t('traer.aviso_evaluaciones', {fichero: m.fichero, motivo: m.estado_evaluaciones.motivo}));
    for (const a of m.avisos)
      textos.push(
        a.tipo === 'curso_tal_cual'
          ? t('traer.aviso_curso_tal_cual', {fichero: m.fichero, encabezado: a.encabezado})
          : t('traer.aviso_sin_curso', {fichero: m.fichero}),
      );
  }
  for (const cambio of traido.cambios) textos.push(t('traer.aviso_cambio', {cambio}));
  if (textos.length === 0) return null;
  return <Plegados rotulo={contar(textos.length, 'traer.avisos_uno', 'traer.avisos_n')} textos={textos} />;
}

// ───────────────────────── Las materias ─────────────────────────

/** Las materias de Clickedu; las pendientes, aparte y al final. */
function Materias({materias, hecho}: {materias: PlanMateria[]; hecho: Hecho | null}) {
  const noEscritas = new Set(hecho?.fallos.flatMap((f) => (f.tipo === 'escribir' ? [f.fichero] : [])));
  const normales = materias.filter((m) => !m.pendiente);
  const pendientes = materias.filter((m) => m.pendiente);
  return (
    <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden'}}>
      <Encabezado>
        <span>{t('traer.col_plantilla')}</span>
        <span style={{textAlign: 'right'}}>{t('traer.col_alumnos')}</span>
        <span style={{paddingLeft: 24}}>{t('traer.col_que_pasa')}</span>
      </Encabezado>
      {normales.map((m, i) => (
        <FilaMateria
          key={m.materia.id}
          m={m}
          noEscrita={noEscritas.has(m.fichero)}
          ultima={i === normales.length - 1 && pendientes.length === 0}
        />
      ))}
      {pendientes.length > 0 && (
        <>
          <Encabezado arriba={normales.length > 0}>
            <span>{t('traer.pendientes')}</span>
          </Encabezado>
          {pendientes.map((m, i) => (
            <FilaMateria
              key={m.materia.id}
              m={m}
              noEscrita={noEscritas.has(m.fichero)}
              ultima={i === pendientes.length - 1}
            />
          ))}
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
        height: 30,
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

function FilaMateria({m, noEscrita, ultima}: {m: PlanMateria; noEscrita: boolean; ultima: boolean}) {
  const clickedu = [m.materia.nombre, m.materia.grupo, m.materia.curso].filter(Boolean).join(' · ');
  const fallo = m.estado.tipo === 'fallo' || noEscrita;
  const alumnos = 'alumnos' in m.estado ? m.estado.alumnos : undefined;
  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: COLUMNAS,
        alignItems: 'center',
        minHeight: 46,
        padding: '6px var(--row-pad)',
        borderBottom: ultima ? 'none' : '1px solid var(--border)',
      }}>
      <span style={{minWidth: 0}}>
        <span style={{display: 'block', fontSize: 13.5, color: 'var(--text-primary)', ...cortar}} title={m.fichero}>
          {m.fichero}
        </span>
        <span
          style={{display: 'block', fontSize: 11.5, color: 'var(--text-tertiary)', ...cortar}}
          title={t('traer.en_clickedu', {texto: clickedu})}>
          {t('traer.en_clickedu', {texto: clickedu})}
        </span>
      </span>
      <span style={cifra}>{alumnos ?? '—'}</span>
      <span
        style={{
          paddingLeft: 24,
          minWidth: 0,
          fontSize: 12.5,
          lineHeight: 1.45,
          color: fallo ? 'var(--danger)' : 'var(--text-secondary)',
        }}>
        {noEscrita ? t('traer.no_escrita') : queLePasa(m)}
      </span>
    </div>
  );
}

function queLePasa(m: PlanMateria): string {
  switch (m.estado.tipo) {
    case 'nueva':
      return t('traer.nueva');
    case 'igual':
      return t('traer.igual');
    case 'cambia':
      return t('traer.cambia');
    case 'vacia':
      return t('traer.vacia');
    case 'fallo': {
      const f = m.estado.fallo;
      if (f.tipo === 'no_se_lee') return t('traer.fallo_no_se_lee', {error: f.error});
      return f.fallo.tipo === 'evaluaciones'
        ? t('traer.fallo_evaluaciones', {motivo: f.fallo.roto.motivo})
        : t('traer.fallo_no_es_excel', {estado: f.fallo.estado});
    }
  }
}

// ───────────────────────── Lo que ya hay en la carpeta ─────────────────────────

type Fila = {fichero: string; apartada?: Apartada; duda?: Duda};

/**
 * Lo de la carpeta que va a «antiguas», y las dudas, por orden de nombre:
 * así una duda respondida no salta de sitio. Con `responder`, cada una con
 * notas lleva su desplegable.
 */
function Carpeta({
  plan,
  hecho,
  respuestas,
  dudosas,
  responder,
}: {
  plan: Plan;
  hecho: Hecho | null;
  respuestas: Record<string, Respuesta>;
  dudosas: string[];
  responder?: (fichero: string, valor: string) => void;
}) {
  const filas: Fila[] = [
    ...plan.apartadas.map((a) => ({fichero: a.fichero, apartada: a})),
    ...plan.dudas.map((d) => ({fichero: d.fichero, duda: d})),
  ].sort((a, b) => a.fichero.localeCompare(b.fichero, 'es'));
  if (filas.length === 0) return null;
  // A dónde pueden pasar notas: las que quedan en la raíz con lo traído.
  const destinos = plan.materias
    .filter((m) => m.estado.tipo === 'nueva' || m.estado.tipo === 'igual' || m.estado.tipo === 'cambia')
    .map((m) => m.fichero);

  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 6}}>
      <h4 style={{margin: 0, fontSize: 12.5, fontWeight: 600, color: 'var(--text-secondary)'}}>{t('traer.carpeta')}</h4>
      <div style={{border: '1px solid var(--border)', borderRadius: 8, overflow: 'hidden'}}>
        {filas.map((f, i) => (
          <FilaCarpeta
            key={f.fichero}
            fila={f}
            dudosa={dudosas.includes(f.fichero)}
            ultima={i === filas.length - 1}
            hecho={hecho !== null}
            notasSinPasar={hecho?.fallos.some((x) => x.tipo === 'notas' && x.fichero === f.fichero) ?? false}
            movida={hecho?.movidas.find((m) => m.fichero === f.fichero)?.a}
            valor={valorDe(f, respuestas)}
            opciones={
              responder && (f.duda || f.apartada?.notas || f.apartada?.descartadas)
                ? [
                    ...(dudosas.includes(f.fichero)
                      ? [{value: SIN_DECIDIR, label: t('traer.opcion_sin_decidir')}]
                      : []),
                    ...primero(f.duda?.candidatas ?? [], destinos).map((d) => ({
                      value: `pasar:${d}`,
                      label: t('traer.opcion_pasar', {a: d.replace(/\.xlsx$/i, '')}),
                    })),
                    {value: NINGUNA, label: t('traer.opcion_ninguna')},
                  ]
                : undefined
            }
            responder={responder && ((v) => responder(f.fichero, v))}
          />
        ))}
      </div>
    </div>
  );
}

/** `destinos`, con las `candidatas` delante. */
function primero(candidatas: string[], destinos: string[]): string[] {
  return [...candidatas.filter((c) => destinos.includes(c)), ...destinos.filter((d) => !candidatas.includes(d))];
}

function valorDe(f: Fila, respuestas: Record<string, Respuesta>): string {
  const r = respuestas[f.fichero];
  if (r) return r.tipo === 'ninguna' ? NINGUNA : `pasar:${r.a}`;
  if (f.apartada?.notas) return `pasar:${f.apartada.notas.a}`;
  if (f.apartada?.descartadas) return NINGUNA;
  return SIN_DECIDIR;
}

function FilaCarpeta({
  fila,
  dudosa,
  ultima,
  hecho,
  notasSinPasar,
  movida,
  valor,
  opciones,
  responder,
}: {
  fila: Fila;
  /** Era una duda al traer. */
  dudosa: boolean;
  ultima: boolean;
  /** Ya se ha hecho: lo que no está en `movida` no se ha podido apartar. */
  hecho: boolean;
  /** Apartada, pero sus notas no han pasado: siguen en la sesión. */
  notasSinPasar: boolean;
  movida?: string;
  valor: string;
  opciones?: {value: string; label: string}[];
  responder?: (valor: string) => void;
}) {
  const {apartada, duda} = fila;
  const paso = apartada?.notas;
  const hecha = movida !== undefined;
  const sinApartar = hecho && apartada !== undefined && !hecha;
  const mal = sinApartar || notasSinPasar;
  const lineas: string[] = [];
  let pregunta: string | undefined;

  // Con desplegable, la pregunta (la de una duda no cambia al responderla)
  // y, debajo, qué pasará con lo elegido: con el Excel y con sus notas.
  const conNota = duda?.notas ?? (paso ? paso.pasan + paso.sin_alumno + paso.sin_id : (apartada?.descartadas ?? 0));
  if (opciones) {
    pregunta = dudosa
      ? contar(conNota, 'traer.duda_uno', 'traer.duda')
      : contar(conNota, 'traer.que_hago_uno', 'traer.que_hago');
    if (valor === SIN_DECIDIR) lineas.push(t('traer.efecto_sin_decidir'));
    else if (valor === NINGUNA) lineas.push(contar(conNota, 'traer.efecto_ninguna_uno', 'traer.efecto_ninguna'));
    else if (paso) {
      lineas.push(contar(paso.pasan, 'traer.efecto_pasar_uno', 'traer.efecto_pasar'));
      if (paso.por !== 'respuesta') lineas.push(t(`traer.por_${paso.por}`));
      if (paso.sin_alumno) lineas.push(contar(paso.sin_alumno, 'traer.sin_alumno_uno', 'traer.sin_alumno'));
      if (paso.sin_id) lineas.push(contar(paso.sin_id, 'traer.sin_id_uno', 'traer.sin_id'));
    }
  } else if (duda) {
    lineas.push(t('traer.se_queda'));
  } else if (sinApartar) {
    // Sus notas siguen en la sesión, y la nueva de su nombre no se escribe.
    lineas.push(t('traer.sin_apartar'));
  } else if (apartada) {
    if (!hecho) lineas.push(t('traer.a_antiguas'));
    if (paso) {
      if (notasSinPasar) lineas.push(t('traer.notas_sin_pasar', {a: paso.a}));
      else lineas.push(contar(paso.pasan, 'traer.ha_pasado', 'traer.han_pasado', {a: paso.a}));
      lineas.push(t(`traer.por_${paso.por}`));
      if (paso.sin_alumno) lineas.push(contar(paso.sin_alumno, 'traer.sin_alumno_uno', 'traer.sin_alumno'));
      if (paso.sin_id) lineas.push(contar(paso.sin_id, 'traer.sin_id_uno', 'traer.sin_id'));
    } else if (apartada.descartadas) {
      lineas.push(contar(apartada.descartadas, 'traer.descartadas_uno', 'traer.descartadas'));
    } else {
      lineas.push(t('traer.sin_notas'));
    }
  }
  // Sin respuesta, una duda: el punto ámbar.
  const abierta = duda !== undefined && opciones !== undefined;

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: 'minmax(180px, 0.8fr) minmax(240px, 1.2fr)',
        gap: 16,
        alignItems: 'start',
        padding: '10px var(--row-pad)',
        borderBottom: ultima ? 'none' : '1px solid var(--border)',
      }}>
      <span style={{minWidth: 0, display: 'flex', alignItems: 'center', gap: 8}}>
        {abierta && <Punto />}
        <span style={{minWidth: 0}}>
          <span
            style={{
              display: 'block',
              fontFamily: 'var(--font-mono)',
              fontSize: 12.5,
              color: 'var(--text-primary)',
              ...cortar,
            }}
            title={fila.fichero}>
            {fila.fichero}
          </span>
          {hecha && (
            <span
              style={{
                display: 'block',
                fontFamily: 'var(--font-mono)',
                fontSize: 11,
                color: 'var(--text-tertiary)',
                ...cortar,
              }}
              title={movida}>
              {t('traer.movida_a', {a: movida})}
            </span>
          )}
        </span>
      </span>
      <div style={{display: 'flex', flexDirection: 'column', gap: 6, minWidth: 0}}>
        {pregunta && <span style={{fontSize: 12.5, lineHeight: 1.45, color: 'var(--text-primary)'}}>{pregunta}</span>}
        {opciones && responder && (
          <div style={{maxWidth: 480}}>
            <Dropdown value={valor} options={opciones} onChange={responder} size="sm" />
          </div>
        )}
        {lineas.length > 0 && (
          <span
            style={{
              fontSize: 12.5,
              lineHeight: 1.5,
              color: mal ? 'var(--danger)' : 'var(--text-secondary)',
            }}>
            {lineas.join(' ')}
          </span>
        )}
      </div>
    </div>
  );
}

const cortar: React.CSSProperties = {overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'};
const cifra: React.CSSProperties = {
  textAlign: 'right',
  fontFamily: 'var(--font-mono)',
  fontSize: 13,
  color: 'var(--text-primary)',
};
