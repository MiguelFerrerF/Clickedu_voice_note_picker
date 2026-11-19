/**
 * La pantalla principal: la clase elegida, sus alumnos y sus notas.
 *
 * H1 la llenó en lectura; **H2 deja escribir las notas y exportarlas**. H3
 * trae el dictado.
 *
 * - Cada nota se guarda **al momento** en la sesión del backend
 *   (`poner_nota`): cerrar la app no pierde nada. Aquí se pinta ya, sin
 *   esperar; si el backend no puede guardarla, se dice en rojo y se vuelve a
 *   leer la clase para enseñar lo que de verdad hay.
 * - **Exportar** deja el Excel para Clickedu en `Documentos\Gestor de
 *   Notas\notas`. Si hay algo que no es nota, no se exporta y se dice dónde.
 *   Lo que sale bien se dice en verde, que se apaga solo (D6).
 *
 * - **El dictado** (H3): se mantiene ESPACIO y se dice el alumno y la
 *   nota (`useDictado`, H3a); lo oído se entiende en el backend. Si está
 *   seguro, se apunta, en verde y con «Deshacer», y el panel enseña los
 *   últimos tres (HUI); si duda, se pregunta en el panel y en las filas, y
 *   no se apunta nada hasta que eliges (H3b). En
 *   desarrollo, además, se puede escribir la frase (`entender_frase`).
 * - **El registro** (D8): lo que haces con cada dictado (elegir, descartar,
 *   deshacer, corregir la nota a mano) se apunta en local con
 *   `responder_dictado`: es la respuesta buena para mejorar el dictado.
 *
 * La clase elegida vive en `App` (la eligen también desde Clases). Si no hay
 * ninguna elegida, o la elegida ya no está en la carpeta, se enseña la
 * primera de la lista.
 */

import {invoke} from '@tauri-apps/api/core';
import {useEffect, useRef, useState} from 'react';

import {Avisos} from '@components/clases/Avisos';
import {Cuadricula} from '@components/clases/Cuadricula';
import {SinPlantillas} from '@components/clases/SinPlantillas';
import {PanelDictado} from '@components/dictado/PanelDictado';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Dropdown} from '@components/ui/Dropdown';
import {Icon, ICONOS} from '@components/ui/Icon';
import {alumnosEnPalabras} from '@modules/clases/avisos';
import type {ClaseAbierta, Exportado} from '@modules/clases/tipos';
import {deshacible, type Apuntado, type Dictado, type Pendiente, type Respuesta} from '@modules/dictado/tipos';
import {useClases} from '@modules/clases/useClases';
import {useDictado} from '@modules/dictado/useDictado';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';
import {notaDe} from '@modules/notas/nota';

export interface CalificarProps {
  /** La pantalla está a la vista: solo entonces se dicta. */
  visible: boolean;
  clase: string | null;
  setClase: (fichero: string) => void;
  irAAjustes: () => void;
}

/** Lo último que ha pasado al exportar o guardar, para decirlo. */
type Resultado = {tipo: 'exportado'; exportado: Exportado} | {tipo: 'fallo'; texto: string};

/** Lo que tarda en apagarse el verde de «exportado» y de lo dictado (D6). */
const APAGAR_MS = 6000;

/**
 * Los últimos apuntados por voz que enseña el panel, cada uno con su
 * «Deshacer»: el nuevo grande y dos más. Dictando seguido, así se comprueba
 * el de antes. Más, y el panel, que va pegado arriba, tapa la lista.
 */
const ULTIMOS = 3;

export function Calificar({visible, clase, setClase, irAAjustes}: CalificarProps) {
  useIdioma();
  const {lista, error, cambios} = useClases();
  const clases = lista?.clases ?? [];
  const actual = clases.find((c) => c.fichero === clase) ?? clases[0];

  // La clase abierta, leída de nuevo si cambia la clase, la carpeta avisa o
  // una nota no se ha podido guardar.
  const [abierta, setAbierta] = useState<{fichero: string; clase?: ClaseAbierta; error?: string}>();
  const [releer, setReleer] = useState(0);
  useEffect(() => {
    if (!actual) return;
    let vivo = true;
    invoke<ClaseAbierta>('leer_clase', {fichero: actual.fichero}).then(
      (c) => vivo && setAbierta({fichero: actual.fichero, clase: c}),
      (e) => vivo && setAbierta({fichero: actual.fichero, error: String(e)}),
    );
    return () => {
      vivo = false;
    };
  }, [actual?.fichero, cambios, releer]);

  const [resultado, setResultado] = useState<Resultado>();
  const [encendido, setEncendido] = useState(false);
  const [exportando, setExportando] = useState(false);
  // Otra clase: lo dicho era de la de antes.
  useEffect(() => setResultado(undefined), [actual?.fichero]);
  useEffect(() => {
    if (resultado?.tipo !== 'exportado') return;
    const apagar = setTimeout(() => setEncendido(false), APAGAR_MS);
    return () => clearTimeout(apagar);
  }, [resultado]);

  // El dictado: lo que espera respuesta, lo último apuntado (lo más nuevo
  // primero, de todas las clases), las notas dictadas que siguen en verde
  // (`fichero|clave` → la vez que se encendieron) y la fila que hay que
  // traer a la vista.
  const [pendiente, setPendiente] = useState<Pendiente>();
  const [apuntados, setApuntados] = useState<Apuntado[]>([]);
  const [encendidas, setEncendidas] = useState<Record<string, number>>({});
  const [ver, setVer] = useState<{clave: string; vez: number}>();
  const veces = useRef(0);

  // El push-to-talk (H3a). Lo entendido llega a `alDictar`, si sigue
  // abierta la clase con la que se entendió: cambiar de clase en el segundo
  // que tarda Whisper es raro, y la nota sería de la otra.
  const recibir = useRef<(fichero: string, d: Dictado) => void>(() => {});
  const dictando = useDictado({
    activo: visible && !!actual && abierta?.fichero === actual.fichero && !!abierta.clase,
    fichero: actual?.fichero,
    alDictar: (f, d) => recibir.current(f, d),
  });

  const contenedor = (hijos: React.ReactNode) => <div style={{padding: '28px 32px', maxWidth: 1040}}>{hijos}</div>;

  if (!lista) {
    return contenedor(
      <>
        <Cabecera titulo={t('calificar.titulo')} />
        {error && <Fallo texto={t('calificar.no_se_lee', {error})} />}
      </>,
    );
  }
  if (!actual) {
    return contenedor(
      <>
        <Cabecera titulo={t('calificar.titulo')} />
        <SinPlantillas carpeta={lista.carpeta} irAAjustes={irAAjustes} />
      </>,
    );
  }

  const fichero = actual.fichero;
  const abiertaAhora = abierta?.fichero === fichero ? abierta.clase : undefined;
  const errorClase = abierta?.fichero === fichero ? abierta.error : undefined;
  const alumnos = abiertaAhora?.plantilla.alumnos ?? [];
  const puestas = abiertaAhora?.puestas ?? {};
  const notas = alumnos.map((a) => notaDe(a, puestas));
  // «Exportar» es el botón principal solo con algo que exportar (HUI). Sin
  // ninguna nota, gris y sin pulsarse: el Excel saldría vacío y mandaría el
  // bueno de antes a `notas_viejas`. Una que no es nota sí cuenta: al
  // pulsar, se dice cuál.
  const hayQueExportar = !!abiertaAhora && notas.some((n) => n.tipo !== 'vacia');

  const pendienteAqui = pendiente?.fichero === fichero ? pendiente.dictado : undefined;
  const apuntadosAqui = apuntados.filter((a) => a.fichero === fichero);
  const propuestas =
    pendienteAqui?.tipo === 'duda'
      ? Object.fromEntries(pendienteAqui.alumnos.map((c) => [c, pendienteAqui.notas]))
      : undefined;
  const prefijo = `${fichero}|`; // `|` no puede ir en un nombre de fichero
  const dictadas = new Set(
    Object.keys(encendidas)
      .filter((k) => k.startsWith(prefijo))
      .map((k) => k.slice(prefijo.length)),
  );

  function poner(clave: string, nota: number | null) {
    setResultado(undefined);
    setAbierta((a) =>
      a?.fichero === fichero && a.clase ? {...a, clase: {...a.clase, puestas: {...a.clase.puestas, [clave]: nota}}} : a,
    );
    invoke('poner_nota', {fichero, clave, nota}).catch((e) => {
      setResultado({tipo: 'fallo', texto: t('calificar.no_se_guarda', {error: String(e)})});
      setReleer((r) => r + 1);
    });
  }

  /**
   * La nota a mano, desde la cuadrícula: si era de las últimas dictadas, es
   * que se ha corregido, y ya no se deshace ni va en verde.
   */
  function ponerAMano(clave: string, nota: number | null) {
    const dictada = apuntadosAqui.find((a) => a.clave === clave);
    if (dictada) {
      responder(dictada.dictado, {tipo: 'corregida', nota});
      setApuntados((l) => l.filter((a) => a.fichero !== fichero || a.clave !== clave));
    }
    apagar(clave);
    poner(clave, nota);
  }

  /** Vuelve a la nota de la plantilla: olvida lo puesto en la sesión. */
  function devolver(clave: string) {
    setAbierta((a) => {
      if (a?.fichero !== fichero || !a.clase) return a;
      const {[clave]: _, ...puestas} = a.clase.puestas;
      return {...a, clase: {...a.clase, puestas}};
    });
    invoke('devolver_nota', {fichero, clave}).catch((e) => {
      setResultado({tipo: 'fallo', texto: t('calificar.no_se_guarda', {error: String(e)})});
      setReleer((r) => r + 1);
    });
  }

  function encender(clave: string) {
    const k = `${fichero}|${clave}`;
    const vez = ++veces.current;
    setEncendidas((e) => ({...e, [k]: vez}));
    setTimeout(() => setEncendidas((e) => (e[k] === vez ? sin(e, k) : e)), APAGAR_MS);
  }

  function apagar(clave: string) {
    const k = `${fichero}|${clave}`;
    setEncendidas((e) => (k in e ? sin(e, k) : e));
  }

  function traerAVista(clave: string) {
    setVer({clave, vez: ++veces.current});
  }

  /**
   * Apunta lo dictado, guardando la nota de antes para poder deshacerlo, y
   * lo pone el primero de los últimos de esta clase.
   */
  function apuntar(clave: string, nota: number, dictado: Dictado) {
    const antes = clave in puestas ? puestas[clave] : undefined;
    poner(clave, nota);
    const nuevo: Apuntado = {fichero, dictado, clave, nota, antes};
    setApuntados((l) => [
      ...[nuevo, ...l.filter((a) => a.fichero === fichero)].slice(0, ULTIMOS),
      ...l.filter((a) => a.fichero !== fichero),
    ]);
    setPendiente(undefined);
    encender(clave);
    traerAVista(clave);
  }

  /** Lo nuevo sustituye a lo que esperaba: una duda sin contestar se queda sin respuesta. */
  function alDictar(d: Dictado) {
    if (d.tipo === 'seguro') return apuntar(d.alumno, d.nota, d);
    setPendiente({fichero, dictado: d});
    if (d.tipo === 'duda') traerAVista(d.alumnos[0]);
  }
  recibir.current = (f, d) => {
    if (f === fichero) alDictar(d);
  };

  /**
   * Apunta en el registro lo que se ha hecho con un dictado. Si no se puede,
   * no se dice: el backend ya apunta el fallo, y no es asunto del profesor.
   */
  function responder(dictado: Dictado, respuesta: Respuesta) {
    invoke('responder_dictado', {id: dictado.id, respuesta}).catch(() => {});
  }

  function elegir(clave: string, nota: number) {
    if (pendienteAqui?.tipo !== 'duda') return;
    responder(pendienteAqui, {tipo: 'elegida', clave, nota});
    apuntar(clave, nota, pendienteAqui);
  }

  function descartar() {
    if (pendienteAqui?.tipo === 'duda') responder(pendienteAqui, {tipo: 'descartada'});
    setPendiente(undefined);
  }

  /** Deshace el apuntado del dictado `id`, si es lo último dictado a su alumno. */
  function deshacer(id: string) {
    const i = apuntadosAqui.findIndex((a) => a.dictado.id === id);
    if (i < 0 || !deshacible(apuntadosAqui, i)) return;
    const a = apuntadosAqui[i];
    responder(a.dictado, {tipo: 'deshecha'});
    if (a.antes === undefined) devolver(a.clave);
    else poner(a.clave, a.antes);
    apagar(a.clave);
    setApuntados((l) => l.filter((x) => x !== a));
    traerAVista(a.clave);
  }

  function probar(texto: string) {
    invoke<Dictado>('entender_frase', {fichero, texto}).then(alDictar, (e) =>
      setResultado({tipo: 'fallo', texto: t('dictado.fallo', {error: String(e)})}),
    );
  }

  const nombreDe = (clave: string) => {
    const a = alumnos.find((x) => x.clave === clave);
    if (!a) return clave;
    return a.nombre.trim() ? a.nombre : t('cuadricula.sin_nombre', {id: a.id});
  };

  function exportar() {
    const invalidas = alumnos
      .filter((_, i) => notas[i].tipo === 'invalida')
      .map((a) => (a.nombre.trim() ? a.nombre : t('cuadricula.sin_nombre', {id: a.id})));
    if (invalidas.length > 0) {
      const alumnosMal = alumnosEnPalabras(invalidas);
      setResultado({
        tipo: 'fallo',
        texto:
          invalidas.length === 1
            ? t('exportar.hay_invalida', {alumno: alumnosMal})
            : t('exportar.hay_invalidas', {alumnos: alumnosMal}),
      });
      return;
    }
    setExportando(true);
    invoke<Exportado>('exportar_clase', {fichero})
      .then(
        (exportado) => {
          setResultado({tipo: 'exportado', exportado});
          setEncendido(true);
        },
        (e) => setResultado({tipo: 'fallo', texto: t('exportar.fallo', {error: String(e)})}),
      )
      .finally(() => setExportando(false));
  }

  return contenedor(
    <>
      <Cabecera
        titulo={
          <Dropdown
            size="titulo"
            value={fichero}
            onChange={setClase}
            title={t('calificar.elegir_clase')}
            options={clases.map((c) => ({value: c.fichero, label: c.nombre}))}
          />
        }
        intro={<span style={{fontFamily: 'var(--font-mono)', fontSize: 12}}>{fichero}</span>}
        derecha={
          <Button
            variant={hayQueExportar ? 'primary' : 'ghost'}
            onClick={exportar}
            disabled={!hayQueExportar || exportando}
            title={hayQueExportar ? t('exportar.ayuda') : t('exportar.nada')}>
            <Icon d={ICONOS.exportar} size={14} />
            {t('exportar.boton')}
          </Button>
        }
      />
      {errorClase && <Fallo texto={t('calificar.no_se_lee', {error: errorClase})} />}
      {resultado?.tipo === 'fallo' && <Fallo texto={resultado.texto} />}
      {resultado?.tipo === 'exportado' && (
        <Hecho
          exportado={resultado.exportado}
          encendido={encendido}
          alFallar={(e) => setResultado({tipo: 'fallo', texto: t('exportar.no_abre', {error: e})})}
        />
      )}
      {abiertaAhora && (
        <>
          <Avisos avisos={abiertaAhora.plantilla.avisos} />
          <PanelDictado
            voz={dictando}
            pendiente={pendienteAqui}
            apuntados={apuntadosAqui}
            encendido={apuntadosAqui.length > 0 && dictadas.has(apuntadosAqui[0].clave)}
            nombreDe={nombreDe}
            elegir={elegir}
            descartar={descartar}
            deshacer={deshacer}
            teclado={visible}
            probar={import.meta.env.DEV ? probar : undefined}
          />
          <Cuadricula
            alumnos={alumnos}
            puestas={puestas}
            poner={ponerAMano}
            dictadas={dictadas}
            propuestas={propuestas}
            elegir={elegir}
            ver={ver}
          />
        </>
      )}
    </>,
  );
}

/** `e` sin la clave `k`. */
function sin<T>(e: Record<string, T>, k: string): Record<string, T> {
  const {[k]: _, ...resto} = e;
  return resto;
}

/** Un error de verdad: en rojo (D6). */
function Fallo({texto}: {texto: string}) {
  return (
    <p role="alert" style={{margin: '0 0 16px', fontSize: 13, color: 'var(--danger)'}}>
      {texto}
    </p>
  );
}

/**
 * Exportado: en verde mientras es noticia, y luego en el color normal (D6:
 * el verde es un evento y se apaga solo). Se queda para poder abrir la
 * carpeta hasta que se cambie una nota o de clase.
 */
function Hecho({
  exportado,
  encendido,
  alFallar,
}: {
  exportado: Exportado;
  encendido: boolean;
  alFallar: (error: string) => void;
}) {
  const fichero = exportado.ruta.split(/[\\/]/).pop() ?? exportado.ruta;
  const texto =
    exportado.conNota === 1
      ? t('exportar.hecho_una', {total: exportado.alumnos, fichero})
      : t('exportar.hecho', {n: exportado.conNota, total: exportado.alumnos, fichero});
  return (
    <div
      role="status"
      style={{
        display: 'flex',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '6px 12px',
        marginBottom: 16,
        fontSize: 13,
        color: encendido ? 'var(--success)' : 'var(--text-secondary)',
        transition: 'color 600ms',
      }}>
      <span style={{display: 'inline-flex', alignItems: 'center', gap: 6, minWidth: 0}}>
        <Icon d={ICONOS.comprobado} size={14} style={{flexShrink: 0}} />
        <span>
          {texto}
          {exportado.viejo && ` ${t('exportar.viejo')}`}
        </span>
      </span>
      <Button variant="ghost" onClick={() => invoke('abrir_carpeta_notas').catch((e) => alFallar(String(e)))}>
        <Icon d={ICONOS.carpeta} size={14} />
        {t('exportar.abrir_carpeta')}
      </Button>
    </div>
  );
}
