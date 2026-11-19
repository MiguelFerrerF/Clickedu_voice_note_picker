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
 * - Lo que comparten dos pantallas vive aquí: la clase que se califica
 *   (la eligen Calificar y Clases).
 */

import {useEffect, useState} from 'react';

import {Sidebar} from '@components/layout/Sidebar';
import {Ajustes} from '@screens/Ajustes';
import {Calificar} from '@screens/Calificar';
import {Clases} from '@screens/Clases';
import {Clickedu} from '@screens/Clickedu';
import {Estadisticas} from '@screens/Estadisticas';
import {Muestrario} from '@screens/Muestrario';
import {PANTALLA_INICIAL, type PantallaId} from '@/types/pantalla';

export default function App() {
  const [pantalla, setPantalla] = useState<PantallaId>(PANTALLA_INICIAL);
  const [visitadas, setVisitadas] = useState<ReadonlySet<PantallaId>>(
    () => new Set([PANTALLA_INICIAL]),
  );

  // La clase que se está calificando: la eligen Calificar y Clases.
  const [clase, setClase] = useState<string | null>(null);

  useEffect(() => {
    setVisitadas((v) => (v.has(pantalla) ? v : new Set(v).add(pantalla)));
  }, [pantalla]);

  const irAAjustes = () => setPantalla('ajustes');

  function renderizar(id: PantallaId) {
    switch (id) {
      case 'calificar':
        return (
          <Calificar visible={pantalla === 'calificar'} clase={clase} setClase={setClase} irAAjustes={irAAjustes} />
        );
      case 'clases':
        return (
          <Clases
            abrir={(fichero) => {
              setClase(fichero);
              setPantalla('calificar');
            }}
            irAAjustes={irAAjustes}
          />
        );
      case 'clickedu':
        return <Clickedu />;
      case 'estadisticas':
        return <Estadisticas />;
      case 'ajustes':
        return <Ajustes visible={pantalla === 'ajustes'} />;
      case 'muestrario':
        return <Muestrario />;
    }
  }

  return (
    <div style={{display: 'flex', height: '100vh', overflow: 'hidden'}}>
      <Sidebar pantalla={pantalla} setPantalla={setPantalla} />
      <main style={{flex: 1, minWidth: 0, overflow: 'hidden'}}>
        {[...visitadas].map((id) => (
          <div
            key={id}
            data-pantalla={id}
            style={{display: id === pantalla ? 'block' : 'none', height: '100%', overflowY: 'auto'}}>
            {renderizar(id)}
          </div>
        ))}
      </main>
    </div>
  );
}
