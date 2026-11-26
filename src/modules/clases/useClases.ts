/**
 * La lista de clases de la carpeta de plantillas, siempre al día.
 *
 * Se pide al montar y otra vez cada vez que el backend avisa de que la
 * carpeta ha cambiado (`plantillas-cambiadas`: un Excel nuevo, uno quitado,
 * otra carpeta elegida en Ajustes). Lo usan Calificar, Clases y Ajustes, cada
 * uno con su copia: es una lista corta y leerla es barato.
 *
 * Con `contarNotas`, también cada vez que se guarda una nota
 * (`notas-cambiadas`), para que la columna «Con nota» de Clases vaya al día.
 *
 * Y siempre que se refresca lo de Clickedu solo (H11, `clickedu-refrescado`):
 * cambian los ítems y lo que ya está en Clickedu.
 */

import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import {useEffect, useState} from 'react';

import {EVENTO_REFRESCADO} from '@modules/clickedu/refresco';

import {EVENTO_NOTAS, EVENTO_PLANTILLAS, type ListaClases} from './tipos';

export interface EstadoClases {
  /** `undefined` mientras llega la primera vez. */
  lista?: ListaClases;
  error?: string;
  /**
   * Cuántas veces ha avisado la carpeta. Quien lea una plantilla la vuelve a
   * leer cuando cambia. Sube **con la lista nueva**, no con el aviso: si no,
   * se releería con la lista de antes una clase que ya no está (al traer de
   * Clickedu, la abierta se aparta a `antiguas`).
   */
  cambios: number;
}

export function useClases({contarNotas = false} = {}): EstadoClases {
  const [estado, setEstado] = useState<EstadoClases>({cambios: 0});
  const [avisos, setAvisos] = useState(0);
  const [notas, setNotas] = useState(0);

  useEffect(() => {
    let vivo = true;
    invoke<ListaClases>('listar_clases').then(
      (lista) => vivo && setEstado({lista, cambios: avisos}),
      (e) => vivo && setEstado((antes) => ({lista: antes.lista, error: String(e), cambios: avisos})),
    );
    return () => {
      vivo = false;
    };
  }, [avisos, notas]);

  useEffect(() => {
    const quitar = [
      listen(EVENTO_PLANTILLAS, () => setAvisos((v) => v + 1)),
      listen(EVENTO_REFRESCADO, () => setNotas((v) => v + 1)),
    ];
    if (contarNotas) quitar.push(listen(EVENTO_NOTAS, () => setNotas((v) => v + 1)));
    return () => {
      quitar.forEach((q) => q.then((f) => f()));
    };
  }, [contarNotas]);

  return estado;
}
