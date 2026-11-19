/**
 * El panel del dictado (H3b), encima de la cuadrícula y **pegado arriba** al
 * bajar por la lista: lo que se acaba de dictar se ve aunque el alumno esté
 * abajo del todo.
 *
 * Enseña lo que ha pasado, con la regla de color (D6):
 *
 * - **Duda**: borde discontinuo y `--accent-wash` (provisional, falta que
 *   lo confirmes), por qué se duda y las opciones, lo más probable primero.
 *   Hasta que eliges no se apunta nada.
 * - **No entendida**: el punto ámbar de «aquí hay algo que mirar», no rojo:
 *   la app no ha fallado, es que no se ha oído bien.
 * - **Las últimas apuntadas** (HUI), cada una con su «Deshacer» por si la
 *   app se ha equivocado de alumno. La nueva, grande (nota y nombre es lo
 *   que se lee de un vistazo mientras se dicta), con la nota en verde, que
 *   se apaga sola; las de antes, debajo, pequeñas y en gris.
 *
 * Y, encima, **la voz** (H3a): cómo se dicta (el micrófono se elige en
 * Ajustes; aquí solo se dice si el elegido falta), el «escuchando» mientras se
 * mantiene ESPACIO (en turquesa: está pasando, no ha terminado), el
 * «entendiendo» y lo que no ha ido bien. Si falta el modelo de voz, la
 * oferta de descargarlo, con su avance. En desarrollo, además, un campo
 * para escribir la frase como si se dictara.
 */

import {Fragment, useEffect, useLayoutEffect, useRef, useState} from 'react';

import {Punto} from '@components/clases/Avisos';
import {Button} from '@components/ui/Button';
import {Icon, ICONOS} from '@components/ui/Icon';
import {Progreso} from '@components/ui/Progreso';
import {conTeclas, Tecla} from '@components/ui/Tecla';
import {TextInput} from '@components/ui/TextInput';
import {deshacible, opciones, type Apuntado, type Dictado} from '@modules/dictado/tipos';
import type {Dictando} from '@modules/dictado/useDictado';
import {t} from '@modules/i18n';

export interface PanelDictadoProps {
  /** El push-to-talk: el modelo, el micrófono, si escucha. */
  voz: Dictando;
  /** Lo que espera respuesta en esta clase: una duda o algo no entendido. */
  pendiente: Dictado | undefined;
  /** Lo último apuntado por voz en esta clase, lo más nuevo primero. */
  apuntados: Apuntado[];
  /** La última nota apuntada sigue en verde. */
  encendido: boolean;
  nombreDe: (clave: string) => string;
  elegir: (clave: string, nota: number) => void;
  descartar: () => void;
  /** Deshace el apuntado del dictado con ese `id`. */
  deshacer: (id: string) => void;
  /** La pantalla está a la vista: las teclas de las dudas valen. */
  teclado: boolean;
  /** Solo en desarrollo: entender una frase escrita. */
  probar?: (texto: string) => void;
}

/**
 * Lo que mide el panel, en su contenedor: las filas lo usan para no quedar
 * debajo de él al traerlas a la vista (`scroll-margin-top`). Una duda con
 * cuatro opciones mide el doble que una nota apuntada.
 */
export const ALTO_PANEL = '--alto-panel-dictado';

export function PanelDictado({
  voz,
  pendiente,
  apuntados,
  encendido,
  nombreDe,
  elegir,
  descartar,
  deshacer,
  teclado,
  probar,
}: PanelDictadoProps) {
  const panel = useRef<HTMLDivElement>(null);
  // Con cada pintado, antes de que la cuadrícula traiga una fila a la vista.
  useLayoutEffect(() => {
    const el = panel.current;
    el?.parentElement?.style.setProperty(ALTO_PANEL, `${el.offsetHeight}px`);
  });
  useLayoutEffect(() => {
    const padre = panel.current?.parentElement;
    return () => {
      padre?.style.removeProperty(ALTO_PANEL);
    };
  }, []);
  return (
    <div
      ref={panel}
      style={{
        position: 'sticky',
        top: 0,
        zIndex: 2,
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
        // El fondo de la página tapa las filas que pasan por debajo.
        margin: '0 -8px',
        padding: '10px 8px 12px',
        background: 'var(--bg)',
      }}>
      <Voz voz={voz} />
      {voz.microfonoFalta && (
        <Linea color="var(--text-primary)" icono={<Punto />}>
          {t('dictado.falta_microfono', {nombre: voz.microfonoFalta})}
        </Linea>
      )}
      {probar && <ProbarFrase probar={probar} />}
      {pendiente?.tipo === 'duda' && (
        <Duda dictado={pendiente} nombreDe={nombreDe} elegir={elegir} descartar={descartar} teclado={teclado} />
      )}
      {pendiente?.tipo === 'no_entendido' && <NoEntendido dictado={pendiente} descartar={descartar} />}
      {apuntados.length > 0 && (
        <Apuntados apuntados={apuntados} encendido={encendido} nombreDe={nombreDe} deshacer={deshacer} />
      )}
    </div>
  );
}

const nota = (n: number) => String(n);

const MB = (bytes: number) => Math.round(bytes / 1e6);

/** Una línea de la voz: icono y texto, en el color que toque. */
function Linea({color, icono, children}: {color: string; icono: React.ReactNode; children: React.ReactNode}) {
  return (
    <div
      role="status"
      style={{display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, lineHeight: 1.6, color, minWidth: 0}}>
      {icono}
      <span style={{overflowWrap: 'anywhere', minWidth: 0}}>{children}</span>
    </div>
  );
}

/**
 * Cómo se oye mientras se mantiene ESPACIO (HUI): si la barra no se mueve
 * al hablar, el micrófono no llega, y se ve antes de que Whisper escriba
 * «[Música]». En turquesa, que es para el progreso: está pasando.
 */
function Medidor({nivel}: {nivel: number}) {
  return (
    <div
      role="meter"
      aria-label={t('dictado.nivel')}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(nivel * 100)}
      style={{
        width: 140,
        height: 6,
        flexShrink: 0,
        borderRadius: 3,
        overflow: 'hidden',
        background: 'var(--bg-sunk)',
        boxShadow: 'inset 0 0 0 1px var(--border)',
      }}>
      <div
        style={{
          height: '100%',
          background: 'var(--accent)',
          transformOrigin: 'left',
          transform: `scaleX(${nivel})`,
          transition: 'transform 80ms linear',
        }}
      />
    </div>
  );
}

/** Un texto con `{tecla}`: ahí va ESPACIO, dibujado como una tecla. */
const conTecla = (texto: string) => conTeclas(texto, {tecla: t('dictado.tecla_espacio')});

const microfono = (estilo?: React.CSSProperties) => (
  <Icon d={ICONOS.microfono} size={14} style={{flexShrink: 0, ...estilo}} />
);

/** Cómo está la voz: lo primero del panel. */
function Voz({voz}: {voz: Dictando}) {
  const {modelo} = voz;
  if (modelo.tipo === 'comprobando') return null;
  if (modelo.tipo === 'falta') {
    return (
      <div style={{display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: '8px 16px'}}>
        <Linea color="var(--text-secondary)" icono={microfono()}>
          {t('dictado.sin_modelo')}
        </Linea>
        <Button onClick={voz.descargar}>
          {modelo.fallo ? t('dictado.reintentar') : t('dictado.descargar')}
        </Button>
        {modelo.fallo && (
          <p role="alert" style={{margin: 0, flexBasis: '100%', fontSize: 13, color: 'var(--danger)'}}>
            {t('dictado.no_descarga', {error: modelo.fallo})}
          </p>
        )}
      </div>
    );
  }
  if (modelo.tipo === 'descargando') {
    return <Progreso rotulo={t('dictado.descargando')} hecho={MB(modelo.hecho)} total={MB(modelo.total)} />;
  }
  if (voz.escuchando) {
    return (
      <div style={{display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: '6px 16px'}}>
        <Linea color="var(--accent-ink)" icono={microfono({animation: 'pulse 1.2s ease-in-out infinite'})}>
          {conTecla(t('dictado.escuchando'))}
        </Linea>
        <Medidor nivel={voz.nivel} />
      </div>
    );
  }
  if (voz.fallo) {
    return (
      <p role="alert" style={{margin: 0, fontSize: 13, color: 'var(--danger)', overflowWrap: 'anywhere'}}>
        {voz.fallo.tipo === 'microfono'
          ? t('dictado.sin_microfono', {error: voz.fallo.error})
          : t('dictado.fallo', {error: voz.fallo.error})}
      </p>
    );
  }
  if (voz.entendiendo > 0) {
    return (
      <Linea color="var(--text-secondary)" icono={microfono()}>
        {t('dictado.entendiendo')}
      </Linea>
    );
  }
  if (voz.aviso) {
    return (
      <Linea color="var(--text-primary)" icono={<Punto />}>
        {voz.aviso === 'corto' ? conTecla(t('dictado.corto')) : t('dictado.nada_oido')}
      </Linea>
    );
  }
  return (
    <Linea color="var(--text-secondary)" icono={microfono()}>
      {conTecla(t('dictado.como'))}
    </Linea>
  );
}

/**
 * Lo que se ha oído, en pequeño y en gris: para saber de dónde sale. Si
 * Whisper repitió la frase en bucle, una vez, y se dice.
 */
function Oido({dictado: {oido, repetida}}: {dictado: Dictado}) {
  return (
    <span style={{fontSize: 12, color: 'var(--text-tertiary)', overflowWrap: 'anywhere'}}>
      {repetida ? t('dictado.oido_repetida', {oido: repetida.trim()}) : t('dictado.oido', {oido: oido.trim()})}
    </span>
  );
}

/** Lo que ocupan el micrófono y su hueco: las de antes van debajo de la nota. */
const SANGRIA = 24;

const recortado: React.CSSProperties = {
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  minWidth: 0,
};

/**
 * Las últimas notas apuntadas. Cada «Deshacer» dice de qué nota es (son
 * varios), y solo lo lleva la última dictada a cada alumno: la de antes,
 * hasta que se deshaga la nueva, ya no está puesta.
 */
function Apuntados({
  apuntados,
  encendido,
  nombreDe,
  deshacer,
}: {
  apuntados: Apuntado[];
  encendido: boolean;
  nombreDe: (clave: string) => string;
  deshacer: (id: string) => void;
}) {
  const [nuevo, ...antes] = apuntados;
  const alumno = nombreDe(nuevo.clave);
  const botonDeshacer = (a: Apuntado, alumno: string, estilo?: React.CSSProperties) => (
    <Button
      variant="ghost"
      onClick={() => deshacer(a.dictado.id)}
      aria-label={t('dictado.deshacer_de', {nota: nota(a.nota), alumno})}
      title={t('dictado.deshacer_de', {nota: nota(a.nota), alumno})}
      style={{marginLeft: 'auto', flexShrink: 0, ...estilo}}>
      {t('dictado.deshacer')}
    </Button>
  );
  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 4}}>
      <div role="status">
        <div style={{display: 'flex', alignItems: 'center', gap: 12, minWidth: 0}}>
          <span
            style={{
              display: 'flex',
              alignItems: 'baseline',
              gap: 8,
              minWidth: 0,
              color: encendido ? 'var(--success)' : 'var(--text-primary)',
              transition: 'color 600ms',
            }}>
            <Icon
              d={ICONOS.microfono}
              size={16}
              style={{flexShrink: 0, alignSelf: 'center', color: encendido ? undefined : 'var(--text-tertiary)'}}
            />
            <span style={{fontFamily: 'var(--font-mono)', fontSize: 18, fontWeight: 600, flexShrink: 0}}>
              {nota(nuevo.nota)}
            </span>
            <span
              style={{...recortado, marginLeft: 4, fontSize: 16, fontWeight: 500, color: 'var(--text-primary)'}}
              title={alumno}>
              {alumno}
            </span>
          </span>
          {botonDeshacer(nuevo, alumno)}
        </div>
        <div style={{paddingLeft: SANGRIA, fontSize: 12, color: 'var(--text-tertiary)', overflowWrap: 'anywhere'}}>
          {t('dictado.apuntado')} <Oido dictado={nuevo.dictado} />
        </div>
      </div>
      {antes.length > 0 && (
        <ol
          aria-label={t('dictado.anteriores')}
          style={{listStyle: 'none', margin: 0, padding: `0 0 0 ${SANGRIA}px`, display: 'flex', flexDirection: 'column'}}>
          {antes.map((a, k) => {
            const suyo = nombreDe(a.clave);
            return (
              <li
                key={a.dictado.id}
                style={{display: 'flex', alignItems: 'center', gap: 10, minHeight: 28, fontSize: 13}}>
                <span
                  style={{fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)', flexShrink: 0, minWidth: '4ch'}}>
                  {nota(a.nota)}
                </span>
                <span style={{...recortado, color: 'var(--text-secondary)'}} title={suyo}>
                  {suyo}
                </span>
                {deshacible(apuntados, k + 1) && botonDeshacer(a, suyo, {padding: '4px 10px', fontSize: 12})}
              </li>
            );
          })}
        </ol>
      )}
    </div>
  );
}

/**
 * Una duda: la pregunta, por qué se duda y **una fila por alumno con sus
 * notas** como botones cortos (HUI), numerados: con `teclado`, 1–4 eligen y
 * Esc descarta. Las teclas no valen escribiendo en un campo, donde una cifra
 * es parte de la nota y Esc deshace lo escrito.
 */
function Duda({
  dictado,
  nombreDe,
  elegir,
  descartar,
  teclado,
}: {
  dictado: Extract<Dictado, {tipo: 'duda'}>;
  nombreDe: (clave: string) => string;
  elegir: (clave: string, nota: number) => void;
  descartar: () => void;
  teclado: boolean;
}) {
  const lista = opciones(dictado);
  const variosAlumnos = dictado.alumnos.length > 1;
  // Sin dependencias: con lo de este render (`elegir` guarda la nota de antes).
  useEffect(() => {
    if (!teclado) return;
    const alPulsar = (e: KeyboardEvent) => {
      if (e.defaultPrevented || e.repeat || e.ctrlKey || e.altKey || e.metaKey) return;
      if (e.target instanceof Element && e.target.closest('input, textarea, select, [contenteditable]')) return;
      const elegida = /^[1-9]$/.test(e.key) ? lista[Number(e.key) - 1] : undefined;
      if (elegida) {
        e.preventDefault();
        elegir(elegida.clave, elegida.nota);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        descartar();
      }
    };
    // En burbuja: el menú y el modal abiertos se quedan su Esc antes.
    window.addEventListener('keydown', alPulsar);
    return () => window.removeEventListener('keydown', alPulsar);
  });
  // Por alumno, en el orden de las opciones.
  const filas: {clave: string; notas: {nota: number; tecla: number}[]}[] = [];
  lista.forEach(({clave, nota: n}, i) => {
    const fila = filas.find((f) => f.clave === clave) ?? filas[filas.push({clave, notas: []}) - 1];
    fila.notas.push({nota: n, tecla: i + 1});
  });
  const variasNotas = dictado.notas.length > 1;
  const pregunta =
    variosAlumnos && variasNotas
      ? t('dictado.duda_quien_y_nota')
      : variosAlumnos
        ? t('dictado.duda_quien', {nota: nota(dictado.notas[0])})
        : variasNotas
          ? t('dictado.duda_nota', {alumno: nombreDe(dictado.alumnos[0])})
          : t('dictado.duda_si', {nota: nota(dictado.notas[0]), alumno: nombreDe(dictado.alumnos[0])});
  return (
    <div
      role="alertdialog"
      aria-label={pregunta}
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 8,
        padding: '12px 14px',
        border: '1px dashed var(--accent)',
        borderRadius: 8,
        background: 'var(--accent-wash)',
      }}>
      <div style={{fontSize: 13, fontWeight: 600, color: 'var(--accent-ink)', overflowWrap: 'anywhere'}}>
        {pregunta}
      </div>
      <div style={{fontSize: 12.5, color: 'var(--text-secondary)', lineHeight: 1.5}}>
        {dictado.motivos.map((m) => t(`dictado.motivo_${m}`)).join(' ')} <Oido dictado={dictado} />
      </div>
      <div
        style={{
          display: 'grid',
          // Con un solo alumno, su nombre ya va en la pregunta.
          gridTemplateColumns: variosAlumnos ? 'minmax(0, max-content) max-content' : 'max-content',
          alignItems: 'center',
          gap: '6px 16px',
        }}>
        {filas.map(({clave, notas}) => {
          const alumno = nombreDe(clave);
          return (
            <Fragment key={clave}>
              {variosAlumnos && (
                <span style={{...recortado, fontSize: 14, fontWeight: 500, color: 'var(--text-primary)'}} title={alumno}>
                  {alumno}
                </span>
              )}
              <span style={{display: 'flex', gap: 6}}>
                {notas.map(({nota: n, tecla}) => (
                  <Button
                    key={n}
                    variant="ghost"
                    onClick={() => elegir(clave, n)}
                    aria-label={t('dictado.elegir', {nota: nota(n), alumno})}
                    title={t('dictado.elegir', {nota: nota(n), alumno})}
                    style={{padding: '5px 10px 5px 6px'}}>
                    <Tecla>{tecla}</Tecla>
                    <span style={{fontFamily: 'var(--font-mono)', fontWeight: 600}}>{nota(n)}</span>
                  </Button>
                ))}
              </span>
            </Fragment>
          );
        })}
      </div>
      <Button
        variant="ghost"
        onClick={descartar}
        style={{alignSelf: 'flex-start', padding: '5px 6px', background: 'transparent', borderColor: 'transparent'}}>
        {t('dictado.descartar')}
        <Tecla>{t('dictado.tecla_esc')}</Tecla>
      </Button>
    </div>
  );
}

function NoEntendido({dictado, descartar}: {dictado: Extract<Dictado, {tipo: 'no_entendido'}>; descartar: () => void}) {
  return (
    <div role="status" style={{display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: '6px 12px', fontSize: 13}}>
      <span style={{display: 'inline-flex', alignItems: 'center', gap: 8, color: 'var(--text-primary)'}}>
        <Punto />
        {t(`dictado.no_entendido_${dictado.motivo}`)}
      </span>
      <Oido dictado={dictado} />
      <Button
        variant="ghost"
        onClick={descartar}
        aria-label={t('dictado.cerrar')}
        title={t('dictado.cerrar')}
        style={{marginLeft: 'auto', padding: 7}}>
        <Icon d={ICONOS.cerrar} size={13} />
      </Button>
    </div>
  );
}

/**
 * Solo en desarrollo: escribir lo que se dictaría, para probar sin hablar.
 * Aquí ESPACIO escribe un espacio (`data-sin-dictado`).
 */
function ProbarFrase({probar}: {probar: (texto: string) => void}) {
  const [texto, setTexto] = useState('');
  return (
    <label data-sin-dictado style={{display: 'flex', alignItems: 'center', gap: 10, fontSize: 12, color: 'var(--text-tertiary)'}}>
      <span style={{flexShrink: 0}}>{t('dictado.probar')}</span>
      <TextInput
        value={texto}
        placeholder={t('dictado.probar_ejemplo')}
        onChange={(e) => setTexto(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && texto.trim()) {
            probar(texto);
            setTexto('');
          }
        }}
        style={{flex: 1, minWidth: 0}}
      />
    </label>
  );
}
