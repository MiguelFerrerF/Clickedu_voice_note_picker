/**
 * Las pantallas del Gestor de Notas.
 *
 * Navegación **por estado**, sin router, como library-app: un `PantallaId` y
 * un `switch` en `App.tsx`. Para cinco pantallas sin URLs, un router sería
 * una dependencia de más.
 *
 * `muestrario` solo aparece en desarrollo: el catálogo de componentes con la
 * paleta. En H0 sirvió para elegir los neutros sobre pantalla.
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
