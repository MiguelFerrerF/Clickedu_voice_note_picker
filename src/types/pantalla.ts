/**
 * Las pantallas del Gestor de Notas.
 *
 * Navegación **por estado**, sin router, como library-app: un `PantallaId` y
 * un `switch` en `App.tsx`. Para cinco pantallas sin URLs, un router sería
 * una dependencia de más.
 *
 * `inicio` (HUI, 2026-11-25): donde abre la app.
 *
 * `ayuda` (H9): cómo funciona cada cosa, en un sitio.
 *
 * `muestrario` solo aparece en desarrollo: el catálogo de componentes con la
 * paleta. En H0 sirvió para elegir los neutros sobre pantalla.
 */

export type PantallaId =
  | 'inicio'
  | 'calificar'
  | 'clases'
  | 'clickedu'
  | 'estadisticas'
  | 'ajustes'
  | 'ayuda'
  | 'muestrario';

/**
 * La pantalla con la que abre la app: el Inicio (HUI, 2026-11-25), con lo
 * que se dejó a medias, lo pendiente con Clickedu y cómo van las clases.
 * Antes abría en Clases, que hacía de resumen a medias.
 */
export const PANTALLA_INICIAL: PantallaId = 'inicio';
