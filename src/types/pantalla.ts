/**
 * Las pantallas del Gestor de Notas.
 *
 * Navegación **por estado**, sin router, como library-app: un `PantallaId` y
 * un `switch` en `App.tsx`. Para cinco pantallas sin URLs, un router sería
 * una dependencia de más.
 *
 * `muestrario` es provisional (H0): sirve para elegir la paleta sobre
 * pantalla y solo aparece en desarrollo. Se queda como catálogo de
 * componentes, sin selector, cuando la paleta esté elegida.
 */

export type PantallaId =
  | 'calificar'
  | 'clases'
  | 'clickedu'
  | 'estadisticas'
  | 'ajustes'
  | 'muestrario';

/** La pantalla con la que abre la app: a lo que se viene. */
export const PANTALLA_INICIAL: PantallaId = 'calificar';
