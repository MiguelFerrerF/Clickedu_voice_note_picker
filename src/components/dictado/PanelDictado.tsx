/**
 * El panel del dictado (H3b), encima de la cuadrícula y **pegado arriba** al
 * bajar por la lista: lo que se acaba de dictar se ve aunque el alumno esté
 * abajo del todo.
 *
 * Enseña lo último que ha pasado, con la regla de color (D6):
 *
 * - **Apuntada** sin preguntar: en verde, que se apaga sola, con
 *   «Deshacer» por si la app se ha equivocado de alumno.
 * - **Duda**: borde discontinuo y `--accent-wash` (provisional, falta que
 *   lo confirmes), por qué se duda y las opciones, lo más probable primero.
 *   Hasta que eliges no se apunta nada.
 * - **No entendida**: el punto ámbar de «aquí hay algo que mirar», no rojo:
 *   la app no ha fallado, es que no se ha oído bien.
 *
 * Y, encima, **la voz** (H3a): cómo se dicta, el «escuchando» mientras se
 * mantiene ESPACIO (en turquesa: está pasando, no ha terminado), el
 * «entendiendo» y lo que no ha ido bien. Si falta el modelo de voz, la
 * oferta de descargarlo, con su avance. En desarrollo, además, un campo
 * para escribir la frase como si se dictara.
 */

import {useLayoutEffect, useRef, useState} from 'react';

import {Punto} from '@components/clases/Avisos';
import {Button} from '@components/ui/Button';
import {Icon, ICONOS} from '@components/ui/Icon';
import {Progreso} from '@components/ui/Progreso';
import {TextInput} from '@components/ui/TextInput';
import {opciones, type Dictado, type UltimoDictado} from '@modules/dictado/tipos';
import type {Dictando} from '@modules/dictado/useDictado';
import {t} from '@modules/i18n';

export interface PanelDictadoProps {
  /** El push-to-talk: el modelo, el micrófono, si escucha. */
  voz: Dictando;
  /** Lo último de esta clase, si hay algo. */
  ultimo: UltimoDictado | undefined;
  /** La nota apuntada sigue en verde. */
  encendido: boolean;
  nombreDe: (clave: string) => string;
  elegir: (clave: string, nota: number) => void;
  descartar: () => void;
  deshacer: () => void;
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
  ultimo,
  encendido,
  nombreDe,
  elegir,
  descartar,
  deshacer,
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
      {probar && <ProbarFrase probar={probar} />}
      {ultimo &&
        (ultimo.apuntada ? (
          <Apuntada
            nota={ultimo.apuntada.nota}
            alumno={nombreDe(ultimo.apuntada.clave)}
            oido={ultimo.dictado.oido}
            encendido={encendido}
            deshacer={deshacer}
          />
        ) : ultimo.dictado.tipo === 'duda' ? (
          <Duda dictado={ultimo.dictado} nombreDe={nombreDe} elegir={elegir} descartar={descartar} />
        ) : ultimo.dictado.tipo === 'no_entendido' ? (
          <NoEntendido dictado={ultimo.dictado} descartar={descartar} />
        ) : null)}
    </div>
  );
}

const nota = (n: number) => String(n);

const MB = (bytes: number) => Math.round(bytes / 1e6);

/** Una línea de la voz: icono y texto, en el color que toque. */
function Linea({color, icono, children}: {color: string; icono: React.ReactNode; children: React.ReactNode}) {
  return (
    <div role="status" style={{display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, color, minWidth: 0}}>
      {icono}
      <span style={{overflowWrap: 'anywhere', minWidth: 0}}>{children}</span>
    </div>
  );
}

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
        <Button variant="ghost" onClick={voz.descargar}>
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
      <Linea color="var(--accent-ink)" icono={microfono({animation: 'pulse 1.2s ease-in-out infinite'})}>
        {t('dictado.escuchando')}
      </Linea>
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
      <Linea color="var(--text-tertiary)" icono={microfono()}>
        {t('dictado.entendiendo')}
      </Linea>
    );
  }
  if (voz.aviso) {
    return (
      <Linea color="var(--text-primary)" icono={<Punto />}>
        {voz.aviso === 'corto' ? t('dictado.corto') : t('dictado.nada_oido')}
      </Linea>
    );
  }
  return (
    <Linea color="var(--text-tertiary)" icono={microfono()}>
      {t('dictado.como')}
      {voz.microfono && ` ${t('dictado.microfono', {nombre: voz.microfono})}`}
    </Linea>
  );
}

/** Lo que se ha oído, en pequeño y en gris: para saber de dónde sale. */
function Oido({texto}: {texto: string}) {
  return (
    <span style={{fontSize: 12, color: 'var(--text-tertiary)', overflowWrap: 'anywhere'}}>
      {t('dictado.oido', {oido: texto.trim()})}
    </span>
  );
}

function Apuntada({
  nota: n,
  alumno,
  oido,
  encendido,
  deshacer,
}: {
  nota: number;
  alumno: string;
  oido: string;
  encendido: boolean;
  deshacer: () => void;
}) {
  return (
    <div
      role="status"
      style={{
        display: 'flex',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '6px 12px',
        fontSize: 13,
        color: encendido ? 'var(--success)' : 'var(--text-secondary)',
        transition: 'color 600ms',
      }}>
      <span style={{display: 'inline-flex', alignItems: 'center', gap: 6, minWidth: 0}}>
        <Icon d={ICONOS.microfono} size={14} style={{flexShrink: 0}} />
        <span style={{overflowWrap: 'anywhere'}}>{t('dictado.apuntada', {nota: nota(n), alumno})}</span>
      </span>
      <Oido texto={oido} />
      <Button variant="ghost" onClick={deshacer} style={{marginLeft: 'auto'}}>
        {t('dictado.deshacer')}
      </Button>
    </div>
  );
}

function Duda({
  dictado,
  nombreDe,
  elegir,
  descartar,
}: {
  dictado: Extract<Dictado, {tipo: 'duda'}>;
  nombreDe: (clave: string) => string;
  elegir: (clave: string, nota: number) => void;
  descartar: () => void;
}) {
  const variosAlumnos = dictado.alumnos.length > 1;
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
        {dictado.motivos.map((m) => t(`dictado.motivo_${m}`)).join(' ')} <Oido texto={dictado.oido} />
      </div>
      <div style={{display: 'flex', flexWrap: 'wrap', gap: 8}}>
        {opciones(dictado).map(({clave, nota: n}) => (
          <Button
            key={`${clave}|${n}`}
            variant="ghost"
            onClick={() => elegir(clave, n)}
            style={{maxWidth: '100%', minWidth: 0}}>
            <span style={{overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', minWidth: 0}}>
              {nombreDe(clave)}
            </span>
            <span style={{fontFamily: 'var(--font-mono)', fontWeight: 600, flexShrink: 0}}>{nota(n)}</span>
          </Button>
        ))}
        <Button variant="ghost" onClick={descartar} style={{background: 'transparent', borderColor: 'transparent'}}>
          {t('dictado.descartar')}
        </Button>
      </div>
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
      <Oido texto={dictado.oido} />
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
