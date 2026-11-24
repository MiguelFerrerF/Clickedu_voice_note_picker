/**
 * **Subir las notas a Clickedu** (H6, `docs/01`): desde Calificar, las notas
 * de la clase (de su ítem, o las de sin ítem) a un ítem de Clickedu.
 *
 * 1. **Elegir el ítem**, solo si se califica sin ítem (como la v1).
 * 2. **Leer Clickedu** otra vez (`preparar_subida`): lo de al traer puede
 *    ser viejo. Exporta además el Excel, que es el plan B.
 * 3. **Lo que se va a hacer**: cuántas notas, cuántas ya están igual y,
 *    **si alguna pisaría otra, cuáles** («antes → nueva», con el punto
 *    ámbar de la cuadrícula). Todo o nada (decidido el 2026-11-23): o se
 *    suben todas, o ninguna.
 * 4. **Subir, importar y comprobar** (`confirmar_subida`), con el paso en
 *    el `Progreso`. Mientras, el diálogo no se cierra.
 * 5. **Hecho**: cuántas están en Clickedu, comprobadas una a una; las que
 *    no, por nombre y en rojo. **Si falla**, el plan B: el Excel ya
 *    exportado y la página de importar de Clickedu en el navegador.
 */

import {Channel, invoke} from '@tauri-apps/api/core';
import {useEffect, useState} from 'react';

import {Modal} from '@components/dialogs/Modal';
import {mensaje} from '@components/clickedu/TarjetaCuenta';
import {Button} from '@components/ui/Button';
import {Dropdown} from '@components/ui/Dropdown';
import {Icon, ICONOS} from '@components/ui/Icon';
import {Progreso} from '@components/ui/Progreso';
import {alumnosEnPalabras} from '@modules/clases/avisos';
import type {ItemsClase} from '@modules/clases/tipos';
import type {FalloSubir, PasoSubir, PreviaSubir, Subido} from '@modules/clickedu/tipos';
import {t} from '@modules/i18n';

export interface DialogoSubirProps {
  fichero: string;
  /** El ítem en el que se califica; `null`, sin ítem: se elige aquí. */
  item: string | null;
  /** Los ítems de la clase, para elegir. */
  items: ItemsClase;
  onClose: () => void;
}

type Fase =
  | {tipo: 'elegir'}
  | {tipo: 'leyendo'}
  | {tipo: 'previa'; previa: PreviaSubir}
  | {tipo: 'subiendo'; previa: PreviaSubir; paso: PasoSubir}
  | {tipo: 'hecho'; previa: PreviaSubir; subido: Subido}
  // `paso`: si falló subiendo, en qué paso (la lista de pasos se queda).
  | {tipo: 'fallo'; fallo: FalloSubir | string; previa?: PreviaSubir; paso?: PasoSubir};

const PASOS: PasoSubir[] = ['subir', 'importar', 'comprobar'];

export function DialogoSubir({fichero, item, items, onClose}: DialogoSubirProps) {
  const [destino, setDestino] = useState(item ?? '');
  const [fase, setFase] = useState<Fase>(item ? {tipo: 'leyendo'} : {tipo: 'elegir'});

  function leer(d: string) {
    setFase({tipo: 'leyendo'});
    invoke<PreviaSubir>('preparar_subida', {fichero, item, destino: d}).then(
      (previa) => setFase({tipo: 'previa', previa}),
      (e) => setFase({tipo: 'fallo', fallo: e as FalloSubir | string}),
    );
  }

  // Con ítem, se lee nada más abrir.
  useEffect(() => {
    if (item) leer(item);
  }, []);

  function subir(previa: PreviaSubir) {
    setFase({tipo: 'subiendo', previa, paso: 'subir'});
    let ultimo: PasoSubir = 'subir';
    const canal = new Channel<PasoSubir>();
    canal.onmessage = (paso) => {
      ultimo = paso;
      setFase((f) => (f.tipo === 'subiendo' ? {...f, paso} : f));
    };
    invoke<Subido>('confirmar_subida', {avance: canal}).then(
      (subido) => setFase({tipo: 'hecho', previa, subido}),
      (e) => {
        const fallo = e as FalloSubir | string;
        // El paso del fallo, si lo dice; si no, el último que llegó.
        const paso = typeof fallo === 'object' && 'paso' in fallo && fallo.paso !== 'leer' ? fallo.paso : ultimo;
        setFase({tipo: 'fallo', fallo, previa, paso});
      },
    );
  }

  function cerrar() {
    if (fase.tipo === 'previa') invoke('cancelar_subida').catch(() => {});
    onClose();
  }

  const ocupado = fase.tipo === 'leyendo' || fase.tipo === 'subiendo';
  const opciones = items.evaluaciones.flatMap((e) =>
    e.items.map((i) => ({value: i.id, label: i.nombre, grupo: e.nombre})),
  );

  return (
    <Modal open onClose={ocupado ? undefined : cerrar} width={520} ariaLabel={t('subir.titulo')}>
      <div style={{padding: '20px 22px 16px', overflowY: 'auto', minHeight: 0}}>
        <h2 style={{margin: 0, fontSize: 15, fontWeight: 600}}>{t('subir.titulo')}</h2>
        {fase.tipo === 'elegir' && (
          <>
            <Texto>{t('subir.elegir_texto')}</Texto>
            <div style={{marginTop: 12}}>
              <Dropdown value={destino} onChange={setDestino} placeholder={t('subir.elegir_item')} options={opciones} />
            </div>
          </>
        )}
        {fase.tipo === 'leyendo' && (
          <div style={{marginTop: 14}}>
            <Progreso rotulo={t('subir.leyendo')} />
          </div>
        )}
        {fase.tipo === 'previa' && <Previa previa={fase.previa} />}
        {fase.tipo === 'subiendo' && <Pasos paso={fase.paso} estado="curso" />}
        {/* El resultado, debajo de los pasos: la misma ventana, no otra. */}
        {fase.tipo === 'hecho' && (
          <>
            <Pasos paso="comprobar" estado="hecho" />
            <Hecho previa={fase.previa} subido={fase.subido} />
          </>
        )}
        {fase.tipo === 'fallo' && (
          <>
            {fase.paso && <Pasos paso={fase.paso} estado="fallo" />}
            <Fallo fallo={fase.fallo} previa={fase.previa} fichero={fichero} destino={destino} />
          </>
        )}
      </div>
      <Pie>
        {fase.tipo === 'elegir' && (
          <>
            <Button variant="ghost" onClick={cerrar}>
              {t('subir.cancelar')}
            </Button>
            <Button disabled={!destino} onClick={() => leer(destino)}>
              {t('subir.seguir')}
            </Button>
          </>
        )}
        {fase.tipo === 'previa' && (
          <>
            <Button variant="ghost" onClick={cerrar}>
              {t('subir.cancelar')}
            </Button>
            <Button onClick={() => subir(fase.previa)}>
              <Icon d={ICONOS.clickedu} size={14} />
              {textoBoton(fase.previa)}
            </Button>
          </>
        )}
        {ocupado && (
          <Button variant="ghost" disabled>
            {t('subir.cancelar')}
          </Button>
        )}
        {(fase.tipo === 'hecho' || fase.tipo === 'fallo') && <Button onClick={onClose}>{t('subir.cerrar')}</Button>}
      </Pie>
    </Modal>
  );
}

const ROTULOS: Record<PasoSubir, string> = {
  leer: 'subir.leyendo',
  subir: 'subir.paso_subir',
  importar: 'subir.paso_importar',
  comprobar: 'subir.paso_comprobar',
};

/**
 * Los pasos de subir. Mientras sube (`curso`): la barra se mueve siempre
 * (subir el Excel es lo que más tarda y, con el porcentaje, se quedaba
 * vacía y quieta, como colgada), ✓ en los hechos y los segundos que lleva.
 * Al acabar se quedan, con el resultado debajo: todos con ✓ (`hecho`) o el
 * que falló con ✗ en rojo (`fallo`).
 */
function Pasos({paso, estado}: {paso: PasoSubir; estado: 'curso' | 'hecho' | 'fallo'}) {
  const [segundos, setSegundos] = useState(0);
  useEffect(() => {
    const reloj = setInterval(() => setSegundos((s) => s + 1), 1000);
    return () => clearInterval(reloj);
  }, []);
  const actual = estado === 'hecho' ? PASOS.length : PASOS.indexOf(paso);
  const fallo = estado === 'fallo';
  return (
    <div style={{marginTop: estado === 'curso' ? 14 : 4}}>
      {estado === 'curso' && <Progreso rotulo={t(ROTULOS[paso])} />}
      <ol style={{margin: estado === 'curso' ? '12px 0 0' : 0, padding: 0, listStyle: 'none', fontSize: 13}}>
        {PASOS.map((p, i) => (
          <li
            key={p}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '2px 0',
              color:
                i < actual
                  ? 'var(--text-secondary)'
                  : i === actual
                    ? fallo
                      ? 'var(--danger)'
                      : 'var(--text-primary)'
                    : 'var(--text-tertiary)',
              fontWeight: i === actual ? 500 : 400,
            }}>
            <span style={{width: 14, display: 'inline-flex', justifyContent: 'center', flexShrink: 0}}>
              {i < actual ? <Icon d={ICONOS.comprobado} size={12} /> : i === actual ? (fallo ? '✗' : '›') : ''}
            </span>
            {t(`subir.paso_${p}_corto`)}
          </li>
        ))}
      </ol>
      {estado === 'curso' && segundos >= 2 && <Texto color="var(--text-tertiary)">{t('subir.lleva', {n: segundos})}</Texto>}
    </div>
  );
}

function Texto({children, color}: {children: React.ReactNode; color?: string}) {
  return (
    <p style={{margin: '10px 0 0', fontSize: 13, lineHeight: 1.55, color: color ?? 'var(--text-secondary)'}}>
      {children}
    </p>
  );
}

function Pie({children}: {children: React.ReactNode}) {
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'flex-end',
        gap: 8,
        padding: '12px 22px',
        borderTop: '1px solid var(--border)',
        background: 'var(--bg-sunk)',
      }}>
      {children}
    </div>
  );
}

/** El botón de subir: dice si pisa o borra algo (todo o nada). */
function textoBoton(previa: PreviaSubir): string {
  const subir =
    previa.cambiarian.length > 0
      ? t('subir.subir_y_cambiar', {n: previa.conNota})
      : previa.conNota === 1
        ? t('subir.subir_una')
        : t('subir.subir_n', {n: previa.conNota});
  const borrar = previa.seBorrarian.length;
  return borrar > 0 ? t('subir.y_borrar', {subir, n: borrar}) : subir;
}

/** Lo que se va a subir y, si alguna pisaría o borraría otra, cuáles. */
function Previa({previa}: {previa: PreviaSubir}) {
  const vacias = previa.alumnos - previa.conNota - previa.seQuedan - previa.seBorrarian.length;
  const cambiarian = previa.cambiarian.length;
  const borrarian = previa.seBorrarian.length;
  return (
    <>
      <Texto>
        {previa.conNota === 1
          ? t('subir.se_subira', {item: previa.item, evaluacion: previa.evaluacion})
          : t('subir.se_subiran', {n: previa.conNota, item: previa.item, evaluacion: previa.evaluacion})}
        {vacias > 0 && ` ${vacias === 1 ? t('subir.vacia') : t('subir.vacias', {n: vacias})}`}
        {previa.iguales > 0 && ` ${previa.iguales === 1 ? t('subir.igual') : t('subir.iguales', {n: previa.iguales})}`}
        {previa.seQuedan > 0 &&
          ` ${previa.seQuedan === 1 ? t('subir.se_queda') : t('subir.se_quedan', {n: previa.seQuedan})}`}
      </Texto>
      {cambiarian > 0 && (
        <>
          <Texto color="var(--text-primary)">
            <Punto />
            {cambiarian === 1 ? t('subir.cambiaria') : t('subir.cambiarian', {n: cambiarian})}
          </Texto>
          <Lista filas={previa.cambiarian.map((c) => ({nombre: c.nombre, antes: c.antes, nueva: String(c.nueva)}))} />
        </>
      )}
      {borrarian > 0 && (
        <>
          <Texto color="var(--text-primary)">
            <Punto />
            {borrarian === 1 ? t('subir.se_borraria') : t('subir.se_borrarian', {n: borrarian})}
          </Texto>
          <Lista filas={previa.seBorrarian.map((c) => ({nombre: c.nombre, antes: c.antes}))} />
        </>
      )}
    </>
  );
}

/** Cada alumno con la nota de Clickedu tachada y, si la hay, la que la sustituye. */
function Lista({filas}: {filas: {nombre: string; antes: string; nueva?: string}[]}) {
  return (
    <ul
      style={{
        margin: '8px 0 0',
        padding: '6px 10px',
        listStyle: 'none',
        maxHeight: 180,
        overflowY: 'auto',
        background: 'var(--bg-sunk)',
        borderRadius: 6,
        fontSize: 13,
      }}>
      {filas.map((c) => (
        <li
          key={c.nombre}
          style={{display: 'flex', justifyContent: 'space-between', gap: 12, padding: '3px 0', minWidth: 0}}>
          <span style={{overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'}}>{c.nombre}</span>
          <span style={{flexShrink: 0, fontFamily: 'var(--font-mono)'}}>
            <s style={{color: 'var(--text-tertiary)'}}>{c.antes}</s>
            {c.nueva !== undefined && ` → ${c.nueva}`}
          </span>
        </li>
      ))}
    </ul>
  );
}

/** El punto ámbar de «al subir, se pisaría», como en la cuadrícula (`docs/02` §3). */
function Punto() {
  return (
    <span
      aria-hidden
      style={{
        display: 'inline-block',
        width: 7,
        height: 7,
        borderRadius: '50%',
        background: 'var(--warn)',
        marginRight: 7,
        verticalAlign: 'middle',
      }}
    />
  );
}

/** Hecho: en verde (el evento), y en rojo las que no entraron. */
function Hecho({previa, subido}: {previa: PreviaSubir; subido: Subido}) {
  const todas = subido.noEntraron.length === 0;
  return (
    <>
      <p
        role="status"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          margin: '10px 0 0',
          fontSize: 13,
          color: todas ? 'var(--success)' : 'var(--text-primary)',
        }}>
        {todas && <Icon d={ICONOS.comprobado} size={14} style={{flexShrink: 0}} />}
        {!todas
          ? t('subir.hecho_parte', {n: subido.entraron, total: previa.conNota, item: previa.item})
          : subido.entraron === 1
            ? t('subir.hecho_una', {item: previa.item})
            : t('subir.hecho', {n: subido.entraron, item: previa.item})}
      </p>
      {!todas && (
        <Texto color="var(--danger)">
          {/* Todos, sin «y 2 más»: son los que hay que ir a mirar. */}
          {subido.noEntraron.length === 1
            ? t('subir.no_entro', {alumno: alumnosEnPalabras(subido.noEntraron)})
            : t('subir.no_entraron', {alumnos: subido.noEntraron.map((n) => `«${n}»`).join(', ')})}
        </Texto>
      )}
      {subido.borradas.length > 0 && (
        <Texto color="var(--danger)">
          {subido.borradas.length === 1
            ? t('subir.borrada', {alumno: `«${subido.borradas[0]}»`})
            : t('subir.borradas', {alumnos: subido.borradas.map((n) => `«${n}»`).join(', ')})}
        </Texto>
      )}
      {subido.avisos.map((a) => (
        <Texto key={a} color="var(--text-tertiary)">
          {t('subir.aviso', {aviso: a})}
        </Texto>
      ))}
    </>
  );
}

/** El fallo, en rojo, y el plan B: el Excel y la página de importar. */
function Fallo({
  fallo,
  previa,
  fichero,
  destino,
}: {
  fallo: FalloSubir | string;
  previa?: PreviaSubir;
  fichero: string;
  destino: string;
}) {
  const [noAbre, setNoAbre] = useState<string>();
  // Tras subir, el fichero ya está en Clickedu sin importar.
  const subido =
    typeof fallo === 'object' && 'paso' in fallo && (fallo.paso === 'importar' || fallo.paso === 'comprobar');
  return (
    <>
      <Texto color="var(--danger)">{mensajeSubir(fallo)}</Texto>
      {subido && <Texto>{t('subir.quedo_subido')}</Texto>}
      {previa && <Texto>{t('subir.plan_b', {excel: previa.excel.split(/[\\/]/).pop() ?? previa.excel})}</Texto>}
      <div style={{display: 'flex', flexWrap: 'wrap', gap: 8, marginTop: 12}}>
        {previa && (
          <Button variant="ghost" onClick={() => invoke('abrir_carpeta_notas').catch((e) => setNoAbre(String(e)))}>
            <Icon d={ICONOS.carpeta} size={14} />
            {t('exportar.abrir_carpeta')}
          </Button>
        )}
        {destino && (
          <Button
            variant="ghost"
            onClick={() => invoke('abrir_importar_en_clickedu', {fichero, destino}).catch((e) => setNoAbre(String(e)))}>
            <Icon d={ICONOS.clickedu} size={14} />
            {t('subir.abrir_importar')}
          </Button>
        )}
      </div>
      {noAbre && <Texto color="var(--danger)">{noAbre}</Texto>}
    </>
  );
}

function mensajeSubir(fallo: FalloSubir | string): string {
  if (typeof fallo === 'string') return fallo;
  switch (fallo.tipo) {
    case 'conexion':
      return mensaje(fallo.fallo);
    case 'sin_entrar':
      return t('subir.fallo_sin_entrar');
    case 'sin_item':
      return t('subir.fallo_sin_item');
    case 'cambiado':
      return t('subir.fallo_cambiado', {motivo: fallo.motivo});
    case 'otro':
      return fallo.detalle;
  }
}
