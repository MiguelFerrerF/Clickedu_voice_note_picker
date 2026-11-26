/**
 * Raíz del Gestor de Notas: barra lateral a la izquierda, pantalla activa a
 * la derecha.
 *
 * Igual que library-app:
 *
 * - **Navegación por estado**: un `PantallaId` y un `switch`, sin router.
 * - **Una pantalla visitada no se desmonta nunca**: se oculta con
 *   `display: none`. Así lo que estabas haciendo —las notas a medio poner, una
 *   subida a Clickedu en marcha, el scroll— sigue ahí al volver. Se montan la
 *   primera vez que se visitan, no al arrancar. **No guardes en el `useState`
 *   de una pantalla nada que dependa de que se desmonte.**
 * - Cada pantalla lleva su propio scroll.
 * - Lo que comparten dos pantallas vive aquí: la clase que se califica y,
 *   de cada clase, el ítem de Clickedu (H6; los eligen Calificar, Clases y
 *   Estadísticas).
 * - Al arrancar (H7) se traen los datos de la v1.3.4 si está instalada, y se
 *   pregunta si hay una versión nueva. Sin conexión no se dice nada: es lo
 *   normal en un portátil. Primero un diálogo y luego el otro.
 * - Lo de Clickedu se refresca solo (H11, `useRefrescoClickedu`) mientras
 *   la ventana se ve.
 * - La ayuda (H9): lo ya visto, para la bienvenida y el aviso de cada
 *   pantalla, que pone `Cabecera` con `PantallaContext`. La bienvenida sale
 *   la primera vez, tras el diálogo de la v1 y antes del de la actualización.
 */

import {useCallback, useEffect, useMemo, useState} from 'react';

import {invoke} from '@tauri-apps/api/core';

import {DialogoActualizacion} from '@components/dialogs/DialogoActualizacion';
import {DialogoBienvenida} from '@components/dialogs/DialogoBienvenida';
import {DialogoVersionAnterior, type PasoV1, type ResultadoV1} from '@components/dialogs/DialogoVersionAnterior';
import {Sidebar} from '@components/layout/Sidebar';
import {AyudaContext, PantallaContext, type Ayuda as EstadoAyuda} from '@modules/ayuda';
import {buscarActualizacion, saltarVersion, versionSaltada, type VersionNueva} from '@modules/actualizacion';
import {useRefrescoClickedu} from '@modules/clickedu/refresco';
import {Ajustes} from '@screens/Ajustes';
import {Ayuda} from '@screens/Ayuda';
import {Calificar} from '@screens/Calificar';
import {Clases} from '@screens/Clases';
import {Clickedu} from '@screens/Clickedu';
import {Estadisticas} from '@screens/Estadisticas';
import {Inicio} from '@screens/Inicio';
import {Muestrario} from '@screens/Muestrario';
import type {ItemElegido} from '@modules/clases/tipos';
import {PANTALLA_INICIAL, type PantallaId} from '@/types/pantalla';

export default function App() {
  const [pantalla, setPantalla] = useState<PantallaId>(PANTALLA_INICIAL);
  const [visitadas, setVisitadas] = useState<ReadonlySet<PantallaId>>(() => new Set([PANTALLA_INICIAL]));
  useRefrescoClickedu();

  // La clase que se está calificando: la eligen Calificar y Clases.
  const [clase, setClase] = useState<string | null>(null);
  // El ítem de cada clase (H6), por fichero; `null`, sin ítem.
  const [items, setItems] = useState<Record<string, ItemElegido | null>>({});
  const elegirItem = useCallback(
    (fichero: string, item: ItemElegido | null) => setItems((m) => ({...m, [fichero]: item})),
    [],
  );

  useEffect(() => {
    setVisitadas((v) => (v.has(pantalla) ? v : new Set(v).add(pantalla)));
  }, [pantalla]);

  // La última clase abierta en Calificar (HUI): Calificar abre en ella. Si
  // ya se ha elegido otra (desde Inicio o Clases), no se pisa.
  useEffect(() => {
    invoke<{fichero: string; item: ItemElegido | null} | null>('ultima_clase').then(
      (u) => {
        if (!u) return;
        setClase((c) => c ?? u.fichero);
        setItems((m) => (u.fichero in m ? m : {...m, [u.fichero]: u.item}));
      },
      () => {},
    );
  }, []);

  /** Abre la clase en Calificar; con `item` (también `null`, sin ítem), en ese. */
  const abrir = (fichero: string, item?: ItemElegido | null) => {
    setClase(fichero);
    if (item !== undefined) elegirItem(fichero, item);
    setPantalla('calificar');
  };

  const irAAjustes = () => setPantalla('ajustes');

  const [v1, setV1] = useState<ResultadoV1 | null>(null);
  // Hasta saber si hay algo de la v1, la bienvenida espera: si no, saldría
  // y el diálogo de la v1 se le pondría encima.
  const [v1Mirada, setV1Mirada] = useState(false);
  useEffect(() => {
    invoke<PasoV1 | null>('traer_de_la_v1')
      .then(
        (paso) => setV1(paso ? {paso} : null),
        (e) => setV1({error: String(e)}),
      )
      .finally(() => setV1Mirada(true));
  }, []);

  // La ayuda (H9). Si no se puede leer lo visto (unos ajustes dañados), no
  // se enseña nada: mejor eso que la bienvenida cada vez que se abre.
  const [vista, setVista] = useState<ReadonlySet<string> | null>(null);
  useEffect(() => {
    invoke<string[]>('ayuda_vista').then(
      (claves) => setVista(new Set(claves)),
      () => {},
    );
  }, []);
  const marcar = useCallback((clave: string) => {
    setVista((v) => (v && !v.has(clave) ? new Set(v).add(clave) : v));
    invoke('marcar_ayuda', {clave}).catch(() => {});
  }, []);
  const ayuda = useMemo<EstadoAyuda>(() => ({vista, marcar, abrir: () => setPantalla('ayuda')}), [vista, marcar]);
  const [otraVez, setOtraVez] = useState(false);
  const bienvenida = otraVez || (v1Mirada && !v1 && vista !== null && !vista.has('bienvenida'));
  const cerrarBienvenida = () => {
    setOtraVez(false);
    marcar('bienvenida');
  };

  const [nueva, setNueva] = useState<VersionNueva | null>(null);
  useEffect(() => {
    buscarActualizacion()
      .then((v) => {
        if (v && v.version !== versionSaltada()) setNueva(v);
      })
      .catch(() => {});
  }, []);
  const cerrarActualizacion = useCallback(() => setNueva(null), []);
  const saltarActualizacion = useCallback((version: string) => {
    saltarVersion(version);
    setNueva(null);
  }, []);

  function renderizar(id: PantallaId) {
    switch (id) {
      case 'inicio':
        return <Inicio visible={pantalla === 'inicio'} ir={setPantalla} abrir={abrir} />;
      case 'calificar':
        return (
          <Calificar
            visible={pantalla === 'calificar'}
            clase={clase}
            setClase={setClase}
            items={items}
            elegirItem={elegirItem}
            irAAjustes={irAAjustes}
          />
        );
      case 'clases':
        return (
          <Clases abrir={abrir} irAAjustes={irAAjustes} />
        );
      case 'clickedu':
        return <Clickedu />;
      case 'estadisticas':
        return (
          <Estadisticas
            visible={pantalla === 'estadisticas'}
            clase={clase}
            setClase={setClase}
            items={items}
            elegirItem={elegirItem}
            irAAjustes={irAAjustes}
          />
        );
      case 'ajustes':
        return <Ajustes visible={pantalla === 'ajustes'} />;
      case 'ayuda':
        return <Ayuda ir={setPantalla} verBienvenida={() => setOtraVez(true)} />;
      case 'muestrario':
        return <Muestrario />;
    }
  }

  return (
    <AyudaContext.Provider value={ayuda}>
      <div style={{display: 'flex', height: '100vh', overflow: 'hidden'}}>
        <Sidebar pantalla={pantalla} setPantalla={setPantalla} alEncontrar={setNueva} />
        <main style={{flex: 1, minWidth: 0, overflow: 'hidden'}}>
          {[...visitadas].map((id) => (
            <div
              key={id}
              data-pantalla={id}
              style={{display: id === pantalla ? 'block' : 'none', height: '100%', overflowY: 'auto'}}>
              <PantallaContext.Provider value={id}>{renderizar(id)}</PantallaContext.Provider>
            </div>
          ))}
        </main>
        <DialogoVersionAnterior resultado={v1} cerrar={() => setV1(null)} />
        {bienvenida && (
          <DialogoBienvenida
            cerrar={cerrarBienvenida}
            ir={(p) => {
              cerrarBienvenida();
              setPantalla(p);
            }}
          />
        )}
        <DialogoActualizacion
          nueva={v1 || bienvenida ? null : nueva}
          cerrar={cerrarActualizacion}
          saltar={saltarActualizacion}
        />
      </div>
    </AyudaContext.Provider>
  );
}
