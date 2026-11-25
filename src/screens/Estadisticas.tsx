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
 * Lo cuenta el backend (`leer_estadisticas`, con tests). Se vuelve a leer
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
import {Barras, Columnas, Proporcion, type Dato} from '@components/Graficas';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Card} from '@components/ui/Card';
import {Progreso} from '@components/ui/Progreso';
import {buscarItem, EVENTO_NOTAS, type ItemElegido} from '@modules/clases/tipos';
import {useClases} from '@modules/clases/useClases';
import {
  APROBADO,
  type Dictados,
  type Estadisticas as Datos,
  type NotasClase,
  type Quien,
} from '@modules/estadisticas/tipos';
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

const NUM = new Intl.NumberFormat('es-ES', {useGrouping: 'always'});
const NOTA = new Intl.NumberFormat('es-ES', {maximumFractionDigits: 2});
const SEGUNDOS = new Intl.NumberFormat('es-ES', {maximumFractionDigits: 1});
const GRIS: React.CSSProperties = {color: 'var(--text-tertiary)'};
const CIFRAS: React.CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
  gap: 10,
};
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

/** Los segundos, en minutos (y horas, si llega). */
function tiempo(segundos: number): string {
  const min = Math.round(segundos / 60);
  if (min < 60) return t('estadisticas.min', {n: NUM.format(min)});
  return t('estadisticas.horas', {h: NUM.format(Math.floor(min / 60)), n: NUM.format(min % 60)});
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

  const contenedor = (hijos: React.ReactNode) => <div style={{padding: '28px 32px'}}>{hijos}</div>;

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
  return contenedor(
    <>
      <Cabecera
        titulo={
          <TituloClase clases={clases} actual={actual} elegido={elegido} setClase={setClase} elegirItem={elegirItem} />
        }
        intro={<IntroClase actual={actual} elegido={elegido} />}
      />
      {aqui?.error ? (
        <Fallo texto={t('estadisticas.no_se_lee', {error: aqui.error})} />
      ) : !aqui?.datos ? (
        <Progreso rotulo={t('estadisticas.leyendo')} />
      ) : (
        <div style={{display: 'flex', flexDirection: 'column', gap: 14}}>
          <Seccion titulo={t('estadisticas.notas')} />
          <DeLasNotas key={juego} n={aqui.datos.notas} />
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

function DeLasNotas({n}: {n: NotasClase}) {
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
        <Cifra valor={nota(n.media)} rotulo={t('estadisticas.media')} />
        <Cifra valor={nota(n.mediana)} rotulo={t('estadisticas.mediana')} />
        <Cifra
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
            <Quienes n={n} elegida={elegida} quitar={() => setElegida(null)} />
          </div>
        </div>
      )}
    </>
  );
}

/** Quiénes están en lo elegido, con su nota; sin nada elegido, la pista. */
function Quienes({n, elegida, quitar}: {n: NotasClase; elegida: Elegida | null; quitar: () => void}) {
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
              <span
                title={q.nombre}
                style={{flex: 1, minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'}}>
                {q.nombre}
              </span>
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
// Las piezas
// ─────────────────────────────────────────────────────────────────────────────

/** Una cifra, como las de library-app. */
function Cifra({valor, rotulo, pie}: {valor: string; rotulo: string; pie?: string}) {
  return (
    <div
      style={{
        background: 'var(--bg-card)',
        border: '1px solid var(--border)',
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
      <div style={{marginTop: 3, fontSize: 10.5, letterSpacing: '0.07em', textTransform: 'uppercase', ...GRIS}}>
        {rotulo}
      </div>
      {pie && <div style={{marginTop: 2, fontSize: 11.5, color: 'var(--text-secondary)'}}>{pie}</div>}
    </div>
  );
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
