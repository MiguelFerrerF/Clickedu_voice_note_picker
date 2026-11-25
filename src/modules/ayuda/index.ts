/**
 * La ayuda (H9): lo que ya se ha visto y cómo llegar a ella.
 *
 * Lo visto vive en `ajustes.json` (`ayuda_vista`, `marcar_ayuda`), no en el
 * `localStorage`: sobrevive a reinstalar y a actualizar. El estado lo tiene
 * `App` y lo reparte con `AyudaContext`; la pantalla en la que está cada
 * cosa, con `PantallaContext`. Así `Cabecera` pone el aviso de la primera
 * vez sin que cada pantalla lo tenga que pedir.
 */

import {createContext, useContext} from 'react';

import type {PantallaId} from '@/types/pantalla';

/** Las pantallas con aviso la primera vez. Cada una tiene `ayuda.aviso_<id>` en `es.ts`. */
export const CON_AVISO = ['inicio', 'calificar', 'clases', 'estadisticas', 'clickedu', 'ajustes'] as const;
export type PantallaConAviso = (typeof CON_AVISO)[number];

export function tieneAviso(p: PantallaId): p is PantallaConAviso {
  return (CON_AVISO as readonly string[]).includes(p);
}

export interface Ayuda {
  /** Lo ya visto; `null` mientras se lee (y entonces no se enseña nada). */
  vista: ReadonlySet<string> | null;
  /** Da por visto `clave`: `bienvenida` o una pantalla. */
  marcar: (clave: string) => void;
  /** Lleva a la pantalla de Ayuda. */
  abrir: () => void;
}

export const AyudaContext = createContext<Ayuda>({vista: null, marcar: () => {}, abrir: () => {}});
export const useAyuda = () => useContext(AyudaContext);

/** La pantalla en la que se pinta un componente: la pone `App` en cada una. */
export const PantallaContext = createContext<PantallaId | null>(null);
export const usePantalla = () => useContext(PantallaContext);
