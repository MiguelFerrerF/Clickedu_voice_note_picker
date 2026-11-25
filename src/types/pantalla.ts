/**
 * Las pantallas del Gestor de Notas.
 *
 * Navegación **por estado**, sin router, como library-app: un `PantallaId` y
 * un `switch` en `App.tsx`. Para cinco pantallas sin URLs, un router sería
 * una dependencia de más.
 *
 * `ayuda` (H9): cómo funciona cada cosa, en un sitio.
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
  | 'ayuda'
  | 'muestrario';

/**
 * La pantalla con la que abre la app: Clases, el resumen de lo que hay, y de
 * ahí se elige qué calificar (abrir sin más la primera clase no tenía
 * sentido). Una pantalla de inicio propia, más adelante (`docs/01`).
 */
export const PANTALLA_INICIAL: PantallaId = 'clases';
