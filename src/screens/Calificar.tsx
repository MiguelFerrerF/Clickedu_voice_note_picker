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
 * La clase elegida vive en `App` (la eligen también desde Clases). Si no hay
 * ninguna elegida, o la elegida ya no está en la carpeta, se enseña la
 * primera de la lista.
 */

import {invoke} from '@tauri-apps/api/core';
import {useEffect, useState} from 'react';

import {Avisos} from '@components/clases/Avisos';
import {Cuadricula} from '@components/clases/Cuadricula';
import {SinPlantillas} from '@components/clases/SinPlantillas';
import {Cabecera} from '@components/layout/Cabecera';
import {Button} from '@components/ui/Button';
import {Dropdown} from '@components/ui/Dropdown';
import {Icon, ICONOS} from '@components/ui/Icon';
import {alumnosEnPalabras} from '@modules/clases/avisos';
import type {ClaseAbierta, Exportado} from '@modules/clases/tipos';
import {useClases} from '@modules/clases/useClases';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';
import {notaDe} from '@modules/notas/nota';

export interface CalificarProps {
  clase: string | null;
  setClase: (fichero: string) => void;
  irAAjustes: () => void;
}

/** Lo último que ha pasado al exportar o guardar, para decirlo. */
type Resultado = {tipo: 'exportado'; exportado: Exportado} | {tipo: 'fallo'; texto: string};

/** Lo que tarda en apagarse el verde de «exportado» (D6). */
const APAGAR_MS = 6000;

export function Calificar({clase, setClase, irAAjustes}: CalificarProps) {
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
  const calificados = notas.filter((n) => n.tipo === 'valor').length;

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
        titulo={actual.nombre}
        intro={
          <>
            <span style={{fontFamily: 'var(--font-mono)', fontSize: 12}}>{fichero}</span>
            {abiertaAhora && (
              <>
                {' · '}
                <span style={{whiteSpace: 'nowrap'}}>
                  {t('calificar.calificados', {n: calificados, total: alumnos.length})}
                </span>
              </>
            )}
          </>
        }
        derecha={
          <div style={{display: 'flex', gap: 8, alignItems: 'center'}}>
            <div style={{width: 260}}>
              <Dropdown
                value={fichero}
                onChange={setClase}
                title={t('calificar.elegir_clase')}
                options={clases.map((c) => ({value: c.fichero, label: c.nombre}))}
              />
            </div>
            <Button
              variant="primary"
              onClick={exportar}
              disabled={!abiertaAhora || exportando}
              title={t('exportar.ayuda')}>
              <Icon d={ICONOS.exportar} size={14} />
              {t('exportar.boton')}
            </Button>
          </div>
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
          <Cuadricula alumnos={alumnos} puestas={puestas} poner={poner} />
          <p style={{margin: '10px 0 0', fontSize: 12, color: 'var(--text-tertiary)'}}>{t('cuadricula.ayuda')}</p>
        </>
      )}
    </>,
  );
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
