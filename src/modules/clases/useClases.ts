/**
 * La lista de clases de la carpeta de plantillas, siempre al día.
 *
 * Se pide al montar y otra vez cada vez que el backend avisa de que la
 * carpeta ha cambiado (`plantillas-cambiadas`: un Excel nuevo, uno quitado,
 * otra carpeta elegida en Ajustes). Lo usan Calificar, Clases y Ajustes, cada
 * uno con su copia: es una lista corta y leerla es barato.
 */

import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {useEffect, useState} from 'react';

import {EVENTO_PLANTILLAS, type ListaClases} from './tipos';

export interface EstadoClases {
  /** `undefined` mientras llega la primera vez. */
  lista?: ListaClases;
  error?: string;
}

export function useClases(): EstadoClases {
  const [estado, setEstado] = useState<EstadoClases>({});
  const [vuelta, setVuelta] = useState(0);

  useEffect(() => {
    let vivo = true;
    invoke<ListaClases>('listar_clases').then(
      (lista) => vivo && setEstado({lista}),
      (e) => vivo && setEstado((antes) => ({lista: antes.lista, error: String(e)})),
    );
    return () => {
      vivo = false;
    };
  }, [vuelta]);

  useEffect(() => {
    const quitar = listen(EVENTO_PLANTILLAS, () => setVuelta((v) => v + 1));
    return () => {
      quitar.then((f) => f());
    };
  }, []);

  return estado;
}
