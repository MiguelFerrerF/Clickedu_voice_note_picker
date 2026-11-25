/**
 * Las estadísticas de la clase y del dictado (H8).
 *
 * - La clase y el ítem son **los de Calificar** (viven en `App`): se cambian
 *   aquí o allí, con el mismo título.
 * - **Las notas** del ítem: media, mediana (la de verdad, D6), aprobados y
 *   cómo se reparten, punto a punto. Cuenta la nota que vale, la de la app
 *   o la de la plantilla y, en un ítem, la que ya estaba en Clickedu; lo
 *   que no es nota se dice aparte. Al pulsar una columna, los aprobados o
 *   los suspensos, al lado sale **quiénes son**, con su nota: solo en la
 *   pantalla, los nombres no se guardan en ningún sitio (D8).
 * - **El dictado** de la clase, solo si se ha dictado algo en ella, en todos sus ítems (el registro no apunta
 *   el ítem de cada dictado): cuántos por voz, cuántos a la primera, cuánto
 *   se tarda por nota y el **tiempo ahorrado** frente a ponerlas a mano en
 *   Clickedu. Solo cuentas: ni nombres ni frases (D8).
 *
 * - En una clase traída de Clickedu, además de **el ítem**, **la
 *   evaluación** (cada ítem con su media, y la nota de la evaluación de
 *   cada alumno: la calculada, provisional y con borde discontinuo, y la
 *   «Nota final» de Clickedu) y **el alumno** (sus notas frente a la media
 *   de la clase, de una evaluación a otra, en barras con la raya de la
 *   media). Al alumno se llega también pulsando su nombre en «quiénes son»;
 *   y hacia atrás, al pulsar un ítem (en la evaluación o en el alumno) se
 *   va a ese ítem, y al pulsar una evaluación del alumno, a esa evaluación.
 *
 * Lo cuenta el backend (`leer_estadisticas` y `leer_curso`, con tests). Se vuelve a leer
 * al cambiar de clase o de ítem, al cambiar una nota y al volver a la
 * pantalla (lo exportado no avisa).
 *
 * El suspenso **no es rojo** (D6): va en gris, en las columnas y en la
 * barra.
 */

import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {useEffect, useRef, useState} from 'react';

import {SinPlantillas} from '@components/clases/SinPlantillas';
import {IntroClase, TituloClase} from '@components/clases/TituloClase';
import {Cifra} from '@components/Cifra';
import {Barras, Columnas, Proporcion, type Dato} from '@components/Graficas';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Dropdown} from '@components/ui/Dropdown';
import {Progreso} from '@components/ui/Progreso';
import {buscarItem, EVENTO_NOTAS, type ItemElegido} from '@modules/clases/tipos';
import {useClases} from '@modules/clases/useClases';
import {
  APROBADO,
  type Curso,
  type Dictados,
  type EvaluacionCurso,
  type Estadisticas as Datos,
  type NotasClase,
  type Quien,
} from '@modules/estadisticas/tipos';
import {NUM, tiempo} from '@modules/estadisticas/formato';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export interface EstadisticasProps {
  /** La pantalla está a la vista: al volver, se lee otra vez. */
  visible: boolean;
  clase: string | null;
  setClase: (fichero: string) => void;
  items: Record<string, ItemElegido | null>;
  elegirItem: (fichero: string, item: ItemElegido | null) => void;
  irAAjustes: () => void;
}

const NOTA = new Intl.NumberFormat('es-ES', {maximumFractionDigits: 2});
const SEGUNDOS = new Intl.NumberFormat('es-ES', {maximumFractionDigits: 1});
const GRIS: React.CSSProperties = {color: 'var(--text-tertiary)'};
const CIFRAS: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
  gap: 10,
};
type Vista = 'item' | 'evaluacion' | 'alumno';
/** De lo general a lo concreto, como el título: la evaluación, el ítem, el alumno. */
const VISTAS: Vista[] = ['evaluacion', 'item', 'alumno'];

const REJILLA: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(340px, 1fr))',
  gap: 14,
  alignItems: 'start',
};

/** `1 nota`, `4 notas`: la clave `_uno` o `_n`. */
function cuantas(clave: string, n: number): string {
  return n === 1 ? t(`${clave}_uno`) : t(`${clave}_n`, {n: NUM.format(n)});
}

function porciento(parte: number, todo: number): string {
  return `${todo > 0 ? Math.round((parte * 100) / todo) : 0} %`;
}

export function Estadisticas({visible, clase, setClase, items, elegirItem, irAAjustes}: EstadisticasProps) {
  useIdioma();
  const {lista, error, cambios} = useClases();
  const todas = lista?.clases ?? [];
  const clases = [...todas.filter((c) => !c.pendiente), ...todas.filter((c) => c.pendiente)];
  const actual = clases.find((c) => c.fichero === clase) ?? clases[0];
  const elegido = actual ? buscarItem(actual.items, items[actual.fichero]) : undefined;
  const item = elegido?.item.id ?? null;
  const juego = actual ? `${actual.fichero}|${item ?? ''}` : undefined;

  // Una nota cambiada (a mano, dictada o subida) cambia las cuentas.
  const [notas, setNotas] = useState(0);
  useEffect(() => {
    const quitar = listen(EVENTO_NOTAS, () => setNotas((n) => n + 1));
    return () => {
      quitar.then((f) => f());
    };
  }, []);

  // La vista; la evaluación elegida, la misma al cambiar de clase (por su
  // nombre, y si no está, por su sitio: los ids cambian de una etapa a
  // otra); y el alumno elegido en cada clase.
  const [vista, setVista] = useState<Vista>('item');
  const [evaluacionElegida, setEvaluacionElegida] = useState<{nombre: string; sitio: number} | null>(null);
  const [alumnos, setAlumnos] = useState<Record<string, string>>({});
  const conItems = Boolean(actual?.items);
  const aLaVista: Vista = conItems ? vista : 'item';

  const [leidas, setLeidas] = useState<{juego: string; datos?: Datos; error?: string}>();
  useEffect(() => {
    if (!visible || !actual || !juego) return;
    let vivo = true;
    invoke<Datos>('leer_estadisticas', {fichero: actual.fichero, item}).then(
      (datos) => vivo && setLeidas({juego, datos}),
      (e) => vivo && setLeidas({juego, error: String(e)}),
    );
    return () => {
      vivo = false;
    };
  }, [visible, juego, cambios, notas]);

  // Lo de cada evaluación y alumno: solo en una clase traída.
  const [curso, setCurso] = useState<{fichero: string; datos?: Curso | null; error?: string}>();
  useEffect(() => {
    if (!visible || !actual || !conItems) return;
    const fichero = actual.fichero;
    let vivo = true;
    invoke<Curso | null>('leer_curso', {fichero}).then(
      (datos) => vivo && setCurso({fichero, datos}),
      (e) => vivo && setCurso({fichero, error: String(e)}),
    );
    return () => {
      vivo = false;
    };
  }, [visible, actual?.fichero, conItems, cambios, notas]);

  // Al ir de una vista a otra, arriba del todo: si no, se queda en el sitio de la de antes.
  const arriba = useRef<HTMLDivElement>(null);
  const cambiarVista = (v: Vista) => {
    setVista(v);
    arriba.current?.scrollIntoView({block: 'start'});
  };
  const contenedor = (hijos: React.ReactNode) => (
    <div ref={arriba} style={{padding: '28px 32px'}}>
      {hijos}
    </div>
  );

  if (!lista || !actual) {
    return contenedor(
      <>
        <Cabecera titulo={t('estadisticas.titulo')} />
        {error && <Fallo texto={t('calificar.no_se_lee', {error})} />}
        {lista && <SinPlantillas carpeta={lista.carpeta} irAAjustes={irAAjustes} />}
      </>,
    );
  }

  const aqui = leidas?.juego === juego ? leidas : undefined;
  const delCurso = curso?.fichero === actual.fichero ? curso : undefined;
  const irAlAlumno = conItems
    ? (clave: string) => {
        setAlumnos((a) => ({...a, [actual.fichero]: clave}));
        cambiarVista('alumno');
      }
    : undefined;
  // Ir al ítem (desde la evaluación o el alumno): es el de Calificar, como el título.
  const alItem = (evaluacion: string, id: string) =>
    buscarItem(actual.items, {evaluacion, item: id})
      ? () => {
          elegirItem(actual.fichero, {evaluacion, item: id});
          cambiarVista('item');
        }
      : undefined;
  const elegirEvaluacion = (c: Curso, id: string) => {
    const sitio = c.evaluaciones.findIndex((e) => e.id === id);
    if (sitio >= 0) setEvaluacionElegida({nombre: c.evaluaciones[sitio].nombre, sitio});
  };
  // La evaluación: la elegida aquí, la del ítem de Calificar, la de hoy o la primera.
  const evaluacion = (c: Curso) =>
    (evaluacionElegida &&
      (c.evaluaciones.find((e) => e.nombre === evaluacionElegida.nombre) ?? c.evaluaciones[evaluacionElegida.sitio])) ??
    c.evaluaciones.find((e) => e.id === elegido?.evaluacion.id) ??
    c.evaluaciones.find((e) => e.enCurso) ??
    c.evaluaciones[0];
  return contenedor(
    <>
      <Cabecera
        titulo={
          <TituloClase clases={clases} actual={actual} elegido={elegido} setClase={setClase} elegirItem={elegirItem} />
        }
        intro={<IntroClase actual={actual} elegido={elegido} />}
      />
      {conItems && (
        <div role="tablist" style={{display: 'flex', gap: 4, flexWrap: 'wrap', margin: '0 0 14px'}}>
          {VISTAS.map((v) => (
            <button
              key={v}
              type="button"
              role="tab"
              aria-selected={v === aLaVista}
              onClick={() => setVista(v)}
              style={pestana(v === aLaVista)}>
              {t(`estadisticas.vista_${v}`)}
            </button>
          ))}
        </div>
      )}
      {aLaVista !== 'item' ? (
        delCurso?.error ? (
          <Fallo texto={t('estadisticas.no_se_lee', {error: delCurso.error})} />
        ) : delCurso?.datos === undefined ? (
          <Progreso rotulo={t('estadisticas.leyendo')} />
        ) : delCurso.datos === null || delCurso.datos.evaluaciones.length === 0 ? (
          <Vacio texto={t('estadisticas.sin_evaluaciones')} />
        ) : aLaVista === 'evaluacion' ? (
          <DeLaEvaluacion
            curso={delCurso.datos}
            e={evaluacion(delCurso.datos)}
            elegir={(id) => elegirEvaluacion(delCurso.datos!, id)}
            irAlAlumno={irAlAlumno}
            alItem={alItem}
          />
        ) : (
          <DelAlumnoVista
            curso={delCurso.datos}
            clave={alumnos[actual.fichero] ?? delCurso.datos.alumnos[0]?.clave}
            elegir={(clave) => setAlumnos((a) => ({...a, [actual.fichero]: clave}))}
            alItem={alItem}
            irALaEvaluacion={(id) => {
              elegirEvaluacion(delCurso.datos!, id);
              cambiarVista('evaluacion');
            }}
          />
        )
      ) : aqui?.error ? (
        <Fallo texto={t('estadisticas.no_se_lee', {error: aqui.error})} />
      ) : !aqui?.datos ? (
        <Progreso rotulo={t('estadisticas.leyendo')} />
      ) : (
        <div style={{display: 'flex', flexDirection: 'column', gap: 14}}>
          <Seccion titulo={t('estadisticas.notas')} />
          <DeLasNotas key={juego} n={aqui.datos.notas} irAlAlumno={irAlAlumno} />
          {aqui.datos.dictado.clase.porVoz > 0 && (
            <>
              <Seccion titulo={t('estadisticas.dictado')} />
              <DelDictado d={aqui.datos.dictado.clase} todas={aqui.datos.dictado.todas} />
            </>
          )}
        </div>
      )}
    </>,
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Las notas
// ─────────────────────────────────────────────────────────────────────────────

/** Lo elegido en una de las gráficas de las notas: una columna (su punto) o una parte de la barra. */
type Elegida = {grafica: 'columnas'; clave: string} | {grafica: 'proporcion'; clave: 'aprobados' | 'suspensos'};

function DeLasNotas({
  n,
  irAlAlumno,
  provisional,
}: {
  n: NotasClase;
  irAlAlumno?: (clave: string) => void;
  /** La nota calculada: las cifras, con borde discontinuo (D6). */
  provisional?: boolean;
}) {
  const [elegida, setElegida] = useState<Elegida | null>(null);
  const elegir = (e: Elegida) =>
    setElegida((antes) => (antes?.grafica === e.grafica && antes.clave === e.clave ? null : e));
  const nota = (v: number | null) => (v === null ? '—' : NOTA.format(v));
  const aparte = [
    n.sinNota > 0 && cuantas('estadisticas.sin_nota', n.sinNota),
    n.noSonNota > 0 && cuantas('estadisticas.no_son_nota', n.noSonNota),
  ].filter(Boolean);
  return (
    <>
      <div style={CIFRAS}>
        <Cifra valor={nota(n.media)} rotulo={t('estadisticas.media')} provisional={provisional} />
        <Cifra valor={nota(n.mediana)} rotulo={t('estadisticas.mediana')} provisional={provisional} />
        <Cifra
          provisional={provisional}
          valor={NUM.format(n.aprobados)}
          rotulo={t('estadisticas.aprobados')}
          pie={n.conNota > 0 ? t('estadisticas.aprobados_pie', {p: porciento(n.aprobados, n.conNota)}) : undefined}
        />
        <Cifra
          valor={`${NUM.format(n.conNota)} / ${NUM.format(n.alumnos)}`}
          rotulo={t('estadisticas.con_nota')}
          pie={aparte.length > 0 ? aparte.join(' · ') : undefined}
        />
      </div>
      {n.conNota === 0 ? (
        <Vacio texto={t('estadisticas.sin_notas')} />
      ) : (
        <div style={REJILLA}>
          <Card title={t('estadisticas.reparto')} subtitle={t('estadisticas.reparto_pie')}>
            <Columnas
              datos={columnas(n)}
              lectura={masRepetida(n)}
              elegida={elegida?.grafica === 'columnas' ? elegida.clave : null}
              alPulsar={(d) => elegir({grafica: 'columnas', clave: d.clave})}
            />
          </Card>
          <div style={{display: 'flex', flexDirection: 'column', gap: 14, minWidth: 0}}>
            <Card title={t('estadisticas.aprobados')} subtitle={cuantas('estadisticas.de_notas', n.conNota)}>
              <Proporcion
                elegida={elegida?.grafica === 'proporcion' ? elegida.clave : null}
                alPulsar={(d) => elegir({grafica: 'proporcion', clave: d.clave as 'aprobados' | 'suspensos'})}
                datos={[
                  {
                    clave: 'aprobados',
                    rotulo: t('estadisticas.aprobados'),
                    titulo: t('estadisticas.aprobados'),
                    valor: n.aprobados,
                    texto: NUM.format(n.aprobados),
                  },
                  {
                    clave: 'suspensos',
                    rotulo: t('estadisticas.suspensos'),
                    titulo: t('estadisticas.suspensos'),
                    valor: n.conNota - n.aprobados,
                    texto: NUM.format(n.conNota - n.aprobados),
                    gris: true,
                  },
                ]}
              />
            </Card>
            <Quienes n={n} elegida={elegida} quitar={() => setElegida(null)} irAlAlumno={irAlAlumno} />
          </div>
        </div>
      )}
    </>
  );
}

/** Quiénes están en lo elegido, con su nota; sin nada elegido, la pista. */
function Quienes({
  n,
  elegida,
  quitar,
  irAlAlumno,
}: {
  n: NotasClase;
  elegida: Elegida | null;
  quitar: () => void;
  /** Con él, cada nombre lleva a lo del alumno. */
  irAlAlumno?: (clave: string) => void;
}) {
  // En una ventana estrecha queda debajo: se trae a la vista al elegir.
  const caja = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (elegida) caja.current?.scrollIntoView({block: 'nearest', behavior: 'smooth'});
  }, [elegida]);
  if (!elegida) return <div style={{fontSize: 12.5, ...GRIS}}>{t('estadisticas.quienes_pista')}</div>;
  const dentro = (q: Quien) =>
    elegida.grafica === 'columnas'
      ? String(q.punto) === elegida.clave
      : q.nota >= APROBADO === (elegida.clave === 'aprobados');
  const lista = n.quienes.filter(dentro);
  const titulo = elegida.grafica === 'columnas' ? tramo(Number(elegida.clave)) : t(`estadisticas.${elegida.clave}`);
  return (
    <div ref={caja}>
      <Card
        title={titulo}
        subtitle={cuantas('estadisticas.alumnos', lista.length)}
        right={
          <Button variant="ghost" onClick={quitar}>
            {t('estadisticas.quienes_cerrar')}
          </Button>
        }>
        <ul
          data-quienes
          style={{
            margin: 0,
            padding: 0,
            listStyle: 'none',
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
            columnGap: 18,
          }}>
          {lista.map((q, i) => (
            <li
              key={`${q.nombre}-${i}`}
              style={{display: 'flex', gap: 10, alignItems: 'baseline', padding: '3px 0', fontSize: 13, minWidth: 0}}>
              {irAlAlumno ? (
                <button
                  type="button"
                  className="nombre-pulsable"
                  onClick={() => irAlAlumno(q.clave)}
                  title={t('estadisticas.ir_al_alumno', {nombre: q.nombre})}
                  style={NOMBRE}>
                  {q.nombre}
                </button>
              ) : (
                <span title={q.nombre} style={NOMBRE}>
                  {q.nombre}
                </span>
              )}
              <span style={{fontFamily: 'var(--font-mono)', fontSize: 12.5, fontVariantNumeric: 'tabular-nums'}}>
                {NOTA.format(q.nota)}
              </span>
            </li>
          ))}
        </ul>
      </Card>
    </div>
  );
}

const NOMBRE: React.CSSProperties = {
  flex: 1,
  minWidth: 0,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
};

/** Una columna por punto, del 0 al 10; los suspensos, en gris. */
function columnas(n: NotasClase): Dato[] {
  return n.porPunto.map((valor, p) => {
    const titulo = tramo(p);
    return {
      clave: String(p),
      rotulo: String(p),
      titulo,
      valor,
      texto: NUM.format(valor),
      detalle: `${titulo}: ${cuantas('estadisticas.notas_de', valor)}`,
      marca: String(p),
      gris: p < APROBADO,
    };
  });
}

/** «De 7 a 7,99», y el 10 solo. */
function tramo(p: number): string {
  return p === 10 ? t('estadisticas.tramo_diez') : t('estadisticas.tramo', {p, hasta: NOTA.format(p + 0.99)});
}

function masRepetida(n: NotasClase): string {
  const max = Math.max(...n.porPunto);
  const p = n.porPunto.indexOf(max);
  return t('estadisticas.mas_repetida', {tramo: tramo(p).toLowerCase(), n: cuantas('estadisticas.notas_de', max)});
}

// ─────────────────────────────────────────────────────────────────────────────
// El dictado
// ─────────────────────────────────────────────────────────────────────────────

function DelDictado({d, todas}: {d: Dictados; todas: Dictados}) {
  return (
    <>
      <div style={CIFRAS}>
        <Cifra valor={NUM.format(d.porVoz)} rotulo={t('estadisticas.por_voz')} />
        <Cifra
          valor={d.porVoz > 0 ? porciento(d.aLaPrimera, d.porVoz) : '—'}
          rotulo={t('estadisticas.a_la_primera')}
          pie={d.porVoz > 0 ? cuantas('estadisticas.dictados', d.aLaPrimera) : undefined}
        />
        <Cifra
          valor={d.segundosPorNota === null ? '—' : t('estadisticas.s', {n: SEGUNDOS.format(d.segundosPorNota)})}
          rotulo={t('estadisticas.por_nota')}
          pie={t('estadisticas.por_nota_pie')}
        />
        <Cifra
          valor={d.ahorro === null ? '—' : tiempo(d.ahorro)}
          rotulo={t('estadisticas.ahorrado')}
          pie={
            todas.ahorro === null
              ? t('estadisticas.ahorrado_nada')
              : t('estadisticas.ahorrado_todas', {t: tiempo(todas.ahorro)})
          }
        />
      </div>
      <div style={REJILLA}>
        <Card title={t('estadisticas.como_fue')} subtitle={t('estadisticas.como_fue_pie')}>
          <Barras
            datos={(
              [
                ['a_la_primera', d.aLaPrimera, false],
                ['preguntados', d.preguntados, true],
                ['corregidos', d.corregidos, true],
                ['no_entendidos', d.noEntendidos, true],
              ] as const
            ).map(([clave, valor, gris]) => {
              const titulo = t(`estadisticas.${clave}`);
              return {
                clave,
                rotulo: titulo,
                titulo,
                valor,
                texto: NUM.format(valor),
                detalle: `${t(`estadisticas.${clave}_ayuda`)}: ${porciento(valor, d.porVoz)}`,
                gris,
              };
            })}
            lectura={t('estadisticas.como_fue_lectura')}
          />
        </Card>
        <Card title={t('estadisticas.ahorrado')} subtitle={t('estadisticas.ahorrado_como')}>
          <Ahorro d={d} />
        </Card>
      </div>
    </>
  );
}

/** Cómo se ha llegado al ahorro, en frases. */
function Ahorro({d}: {d: Dictados}) {
  if (d.ahorro === null) return <div style={{fontSize: 13, ...GRIS}}>{t('estadisticas.ahorrado_nada')}</div>;
  const lineas = [
    cuantas('estadisticas.llevadas', d.notasLlevadas),
    d.subidas > 0 && cuantas('estadisticas.subidas', d.subidas),
    d.exportadas > 0 && cuantas('estadisticas.exportadas', d.exportadas),
  ].filter(Boolean);
  return (
    <ul style={{margin: 0, paddingLeft: 18, fontSize: 13, lineHeight: 1.6, color: 'var(--text-secondary)'}}>
      {lineas.map((l) => (
        <li key={String(l)}>{l}</li>
      ))}
    </ul>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// La evaluación
// ─────────────────────────────────────────────────────────────────────────────

function notaO(v: number | null): string {
  return v === null ? '—' : NOTA.format(v);
}

/** Lo que va en gris: el suspenso (D6). */
function suspenso(v: number | null): boolean {
  return v !== null && v < APROBADO;
}

function DeLaEvaluacion({
  curso,
  e,
  elegir,
  irAlAlumno,
  alItem,
}: {
  curso: Curso;
  e: EvaluacionCurso;
  elegir: (id: string) => void;
  irAlAlumno?: (clave: string) => void;
  /** Lo que lleva a un ítem, si está entre los de Calificar. */
  alItem: (evaluacion: string, id: string) => (() => void) | undefined;
}) {
  const [cual, setCual] = useState<'calculada' | 'final'>('calculada');
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 14}}>
      {curso.evaluaciones.length > 1 && (
        <>
          <Card title={t('estadisticas.curso')} subtitle={t('estadisticas.curso_pie')}>
            <Barras
              tope={10}
              elegida={e.id}
              alPulsar={(d) => elegir(d.clave)}
              datos={curso.evaluaciones.map((x) => ({
                clave: x.id,
                rotulo: <Cortado texto={x.nombre} />,
                titulo: x.nombre,
                valor: x.calculada.media ?? 0,
                texto: notaO(x.calculada.media),
                detalle: t('estadisticas.curso_detalle', {
                  evaluacion: x.nombre,
                  calculada: notaO(x.calculada.media),
                  final: notaO(x.finalClickedu.media),
                }),
                gris: suspenso(x.calculada.media),
              }))}
            />
          </Card>
          <div role="tablist" style={{display: 'flex', gap: 4, flexWrap: 'wrap'}}>
            {curso.evaluaciones.map((x) => (
              <button
                key={x.id}
                type="button"
                role="tab"
                aria-selected={x.id === e.id}
                onClick={() => elegir(x.id)}
                style={pestana(x.id === e.id)}>
                {x.nombre}
              </button>
            ))}
          </div>
        </>
      )}
      <Seccion titulo={e.nombre} />
      {e.mediaSimple && e.items.length > 0 && <Vacio texto={t('estadisticas.media_simple')} />}
      {e.noCuentan.length > 0 && <Vacio texto={t('estadisticas.no_cuentan', {items: e.noCuentan.join(', ')})} />}
      {e.items.length === 0 ? (
        <Vacio texto={t('estadisticas.sin_items')} />
      ) : (
        <Card title={t('estadisticas.items')} subtitle={t('estadisticas.items_pie')}>
          <Barras
            tope={10}
            lectura={t('estadisticas.items_lectura')}
            alPulsar={(d) => alItem(e.id, d.clave)?.()}
            datos={e.items.map((i) => ({
              pulsable: alItem(e.id, i.id) !== undefined,
              clave: i.id,
              rotulo: <Cortado texto={i.nombre} />,
              titulo: i.dentroDe ? `${i.dentroDe} › ${i.nombre}` : i.nombre,
              valor: i.media ?? 0,
              texto: notaO(i.media),
              detalle: t('estadisticas.item_detalle', {
                item: i.dentroDe ? `${i.dentroDe} › ${i.nombre}` : i.nombre,
                ponderacion:
                  i.ponderacion === null ? '' : t('estadisticas.ponderacion', {p: NOTA.format(i.ponderacion)}),
                media: notaO(i.media),
                aprobados: NUM.format(i.aprobados),
                conNota: NUM.format(i.conNota),
              }),
              gris: suspenso(i.media),
            }))}
          />
        </Card>
      )}
      <div role="tablist" style={{display: 'flex', gap: 4, flexWrap: 'wrap', marginTop: 4}}>
        {(['calculada', 'final'] as const).map((c) => (
          <button
            key={c}
            type="button"
            role="tab"
            aria-selected={c === cual}
            onClick={() => setCual(c)}
            style={pestana(c === cual)}>
            {t(c === 'calculada' ? 'estadisticas.nota_calculada' : 'estadisticas.nota_final')}
          </button>
        ))}
      </div>
      <Vacio texto={t(cual === 'calculada' ? 'estadisticas.nota_calculada_pie' : 'estadisticas.nota_final_pie')} />
      <DeLasNotas
        key={`${e.id}|${cual}`}
        n={cual === 'calculada' ? e.calculada : e.finalClickedu}
        provisional={cual === 'calculada'}
        irAlAlumno={irAlAlumno}
      />
      <Traidas curso={curso} />
    </div>
  );
}

/** Un rótulo de las barras que, si no cabe, acaba en «…». */
function Cortado({texto}: {texto: string}) {
  return <span style={{minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis'}}>{texto}</span>;
}

function Traidas({curso}: {curso: Curso}) {
  return <Vacio texto={t('estadisticas.traidas', {dia: new Date(curso.traidos * 1000).toLocaleDateString('es-ES')})} />;
}

// ─────────────────────────────────────────────────────────────────────────────
// El alumno
// ─────────────────────────────────────────────────────────────────────────────

function DelAlumnoVista({
  curso,
  clave,
  elegir,
  alItem,
  irALaEvaluacion,
}: {
  curso: Curso;
  clave?: string;
  elegir: (clave: string) => void;
  alItem: (evaluacion: string, id: string) => (() => void) | undefined;
  irALaEvaluacion: (id: string) => void;
}) {
  const a = curso.alumnos.find((x) => x.clave === clave) ?? curso.alumnos[0];
  if (!a) return <Vacio texto={t('estadisticas.sin_notas')} />;
  const conCalculada = curso.evaluaciones.filter((_, k) => a.evaluaciones[k].calculada !== null);
  // Lo suyo en todos los ítems del curso: con nota, aprobados y sobre la media de la clase.
  const pares = curso.evaluaciones.flatMap((e, k) => e.items.map((i, n) => ({v: a.evaluaciones[k].notas[n], media: i.media})));
  const conNota = pares.filter((p) => p.v !== null);
  const aprobados = conNota.filter((p) => p.v! >= APROBADO).length;
  const comparables = conNota.filter((p) => p.media !== null);
  const sobreLaMedia = comparables.filter((p) => p.v! >= p.media!).length;
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 14}}>
      <div style={{display: 'flex', flexWrap: 'wrap', alignItems: 'center', columnGap: 14, rowGap: 6}}>
        <div style={{width: 320, maxWidth: '100%'}}>
          <Dropdown
            value={a.clave}
            onChange={elegir}
            title={t('estadisticas.elegir_alumno')}
            options={curso.alumnos.map((x) => ({value: x.clave, label: x.nombre}))}
          />
        </div>
        <Vacio texto={t('estadisticas.alumno_pista')} />
      </div>
      <div style={CIFRAS}>
        <Cifra
          valor={`${NUM.format(conNota.length)} / ${NUM.format(pares.length)}`}
          rotulo={t('estadisticas.items_con_nota')}
          pie={t('estadisticas.en_el_curso')}
        />
        <Cifra
          valor={NUM.format(aprobados)}
          rotulo={t('estadisticas.items_aprobados')}
          pie={conNota.length > 0 ? t('estadisticas.aprobados_pie', {p: porciento(aprobados, conNota.length)}) : undefined}
        />
        <Cifra
          valor={comparables.length > 0 ? `${NUM.format(sobreLaMedia)} / ${NUM.format(comparables.length)}` : '—'}
          rotulo={t('estadisticas.sobre_la_media')}
          pie={t('estadisticas.sobre_la_media_pie')}
        />
      </div>
      {conCalculada.length > 1 && (
        <Card title={t('estadisticas.evolucion')} subtitle={t('estadisticas.evolucion_pie')}>
          <Barras
            tope={10}
            lectura={t('estadisticas.evolucion_lectura')}
            alPulsar={(d) => irALaEvaluacion(d.clave)}
            datos={curso.evaluaciones.map((e, k) => {
              const v = a.evaluaciones[k].calculada;
              return {
                clave: e.id,
                rotulo: <Cortado texto={e.nombre} />,
                titulo: e.nombre,
                valor: v ?? 0,
                texto: notaO(v),
                detalle: t('estadisticas.alumno_detalle', {
                  que: e.nombre,
                  nota: notaO(v),
                  media: notaO(e.calculada.media),
                }),
                referencia: e.calculada.media,
                pulsable: true,
                gris: suspenso(v),
              };
            })}
          />
        </Card>
      )}
      <div style={REJILLA}>
        {curso.evaluaciones.map((e, k) => {
          const suya = a.evaluaciones[k];
          // Una evaluación sin ítems ni nota final no dice nada.
          if (e.items.length === 0 && e.finalClickedu.conNota === 0) return null;
          return (
            <Card
              key={e.id}
              title={e.nombre}
              subtitle={suya.faltan > 0 ? cuantas('estadisticas.faltan', suya.faltan) : undefined}>
              <div style={{display: 'flex', flexDirection: 'column', gap: 12}}>
                <div style={CIFRAS}>
                  <Cifra
                    provisional
                    valor={notaO(suya.calculada)}
                    rotulo={t('estadisticas.calculada')}
                    pie={t('estadisticas.media_clase_pie', {n: notaO(e.calculada.media)})}
                  />
                  <Cifra
                    valor={notaO(suya.finalClickedu)}
                    rotulo={t('estadisticas.final')}
                    pie={t('estadisticas.media_clase_pie', {n: notaO(e.finalClickedu.media)})}
                  />
                </div>
                {e.items.length === 0 ? (
                  <Vacio texto={t('estadisticas.sin_items')} />
                ) : (
                  <div data-notas-alumno>
                    <Barras
                      tope={10}
                      lectura={t('estadisticas.alumno_items_lectura')}
                      alPulsar={(d) => alItem(e.id, d.clave)?.()}
                      datos={e.items.map((i, n) => {
                        const v = suya.notas[n];
                        const nombre = i.dentroDe ? `${i.dentroDe} › ${i.nombre}` : i.nombre;
                        return {
                          clave: i.id,
                          rotulo: <Cortado texto={i.nombre} />,
                          titulo: nombre,
                          valor: v ?? 0,
                          texto: notaO(v),
                          detalle: t('estadisticas.alumno_detalle', {que: nombre, nota: notaO(v), media: notaO(i.media)}),
                          referencia: i.media,
                          pulsable: alItem(e.id, i.id) !== undefined,
                          gris: suspenso(v),
                        };
                      })}
                    />
                  </div>
                )}
              </div>
            </Card>
          );
        })}
      </div>
      <Traidas curso={curso} />
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Las piezas
// ─────────────────────────────────────────────────────────────────────────────

/** El estilo de una pestaña: el de las de library-app. */
function pestana(activa: boolean): React.CSSProperties {
  return {
    padding: '5px 12px',
    border: '1px solid',
    borderColor: activa ? 'var(--border-strong)' : 'transparent',
    borderRadius: 7,
    background: activa ? 'var(--bg-card)' : 'transparent',
    color: activa ? 'var(--text-primary)' : 'var(--text-secondary)',
    fontFamily: 'var(--font-body)',
    fontSize: 12.5,
    fontWeight: activa ? 500 : 400,
    cursor: 'pointer',
  };
}

function Seccion({titulo}: {titulo: string}) {
  return <h2 style={{margin: '8px 0 0', fontSize: 14, fontWeight: 600, color: 'var(--text-primary)'}}>{titulo}</h2>;
}

function Vacio({texto}: {texto: string}) {
  return <div style={{fontSize: 13, ...GRIS}}>{texto}</div>;
}

function Fallo({texto}: {texto: string}) {
  return (
    <p role="alert" style={{margin: '0 0 16px', fontSize: 13, color: 'var(--danger)'}}>
      {texto}
    </p>
  );
}
