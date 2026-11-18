/**
 * La pantalla principal: la clase elegida, sus alumnos y sus notas.
 *
 * H1 la llena **en lectura**: elegir la clase y ver la plantilla tal como
 * está, con lo que no cuadra dicho encima. H2 deja escribir y exportar, H3
 * trae el dictado.
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
import {Dropdown} from '@components/ui/Dropdown';
import type {Plantilla} from '@modules/clases/tipos';
import {useClases} from '@modules/clases/useClases';
import {t} from '@modules/i18n';
import {useIdioma} from '@modules/i18n/useIdioma';

export interface CalificarProps {
  clase: string | null;
  setClase: (fichero: string) => void;
  irAAjustes: () => void;
}

export function Calificar({clase, setClase, irAAjustes}: CalificarProps) {
  useIdioma();
  const {lista, error} = useClases();
  const clases = lista?.clases ?? [];
  const actual = clases.find((c) => c.fichero === clase) ?? clases[0];

  // La plantilla, leída de nuevo si cambia la clase o la carpeta avisa.
  const [leida, setLeida] = useState<{fichero: string; plantilla?: Plantilla; error?: string}>();
  useEffect(() => {
    if (!actual) return;
    let vivo = true;
    invoke<Plantilla>('leer_clase', {fichero: actual.fichero}).then(
      (plantilla) => vivo && setLeida({fichero: actual.fichero, plantilla}),
      (e) => vivo && setLeida({fichero: actual.fichero, error: String(e)}),
    );
    return () => {
      vivo = false;
    };
  }, [actual?.fichero, lista]);

  const contenedor = (hijos: React.ReactNode) => <div style={{padding: '28px 32px', maxWidth: 1040}}>{hijos}</div>;

  if (!lista) {
    return contenedor(
      <>
        <Cabecera titulo={t('calificar.titulo')} />
        {error && <Fallo texto={error} />}
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

  const plantilla = leida?.fichero === actual.fichero ? leida.plantilla : undefined;
  const errorClase = leida?.fichero === actual.fichero ? leida.error : undefined;
  const calificados = plantilla?.alumnos.filter((a) => a.nota.tipo === 'valor').length ?? 0;

  return contenedor(
    <>
      <Cabecera
        titulo={actual.nombre}
        intro={
          <>
            <span style={{fontFamily: 'var(--font-mono)', fontSize: 12}}>{actual.fichero}</span>
            {plantilla && (
              <>
                {' · '}
                {t('calificar.calificados', {n: calificados, total: plantilla.alumnos.length})}
              </>
            )}
          </>
        }
        derecha={
          <div style={{width: 300}}>
            <Dropdown
              value={actual.fichero}
              onChange={setClase}
              title={t('calificar.elegir_clase')}
              options={clases.map((c) => ({value: c.fichero, label: c.nombre}))}
            />
          </div>
        }
      />
      {errorClase && <Fallo texto={errorClase} />}
      {plantilla && (
        <>
          <Avisos avisos={plantilla.avisos} />
          <Cuadricula alumnos={plantilla.alumnos} />
        </>
      )}
    </>,
  );
}

/** Un error de verdad: en rojo (D6). */
function Fallo({texto}: {texto: string}) {
  return (
    <p role="alert" style={{margin: 0, fontSize: 13, color: 'var(--danger)'}}>
      {t('calificar.no_se_lee', {error: texto})}
    </p>
  );
}
