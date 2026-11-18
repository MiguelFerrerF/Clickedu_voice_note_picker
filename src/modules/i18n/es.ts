/**
 * Diccionario castellano — **la única fuente de textos de la interfaz**.
 *
 * La forma es la de library-app: solo hay castellano, así que este fichero es
 * a la vez el diccionario y el contrato de claves (`TranslationKey` sale de
 * él). Ningún texto visible se escribe en un componente.
 *
 * Convención de clave: `ambito.concepto`, en minúsculas y con `_` dentro del
 * concepto. Las claves no se traducen nunca, los valores sí.
 */

const es = {
  // ── Chrome de la app ──
  'app.nombre': 'Gestor de Notas',
  'app.subtitulo': 'Dictado de notas para Clickedu',

  // ── Secciones de la barra lateral ──
  'nav.aula': 'Aula',
  'nav.clickedu_seccion': 'Clickedu',
  'nav.aplicacion': 'Aplicación',

  // ── Entradas de navegación ──
  'nav.calificar': 'Calificar',
  'nav.clases': 'Clases',
  'nav.clickedu': 'Conexión',
  'nav.estadisticas': 'Estadísticas',
  'nav.ajustes': 'Ajustes',
  'nav.muestrario': 'Muestrario',

  // ── Común ──
  'comun.version': 'Versión {version}',
  'comun.llega_en': 'Llega en {hito}',
  'comun.si': 'Sí',
  'comun.no': 'No',

  // ── Pantallas todavía vacías (H0) ──
  'calificar.titulo': 'Calificar',
  'calificar.vacia': 'Aquí pondrás las notas de una clase, con el teclado o dictándolas: mantén ESPACIO, di el nombre y la nota, y suelta.',
  'clases.titulo': 'Clases',
  'clases.vacia': 'Las plantillas de Clickedu que tienes en tu carpeta, una por clase, con cuántos alumnos llevas calificados en cada una.',
  'clickedu.titulo': 'Conexión con Clickedu',
  'clickedu.vacia': 'Entra con tu usuario, trae las plantillas de tus asignaturas y sube las notas a un ítem sin pasar por el navegador.',
  'estadisticas.titulo': 'Estadísticas',
  'estadisticas.vacia': 'Media, mediana, aprobados y cómo se reparten las notas de la clase, y lo que te ha ahorrado el dictado.',
  'ajustes.titulo': 'Ajustes',
  'ajustes.vacia': 'Dónde están tus plantillas y tus notas, el micrófono y la cuenta de Clickedu.',

  // ── Muestrario (H0) ──
  'muestrario.titulo': 'Muestrario',
  'muestrario.subtitulo': 'Los componentes de la app con la paleta, para ver de un vistazo que un cambio no rompe nada. Solo existe en desarrollo.',
  'muestrario.colores': 'Colores',
  'muestrario.botones': 'Botones',
  'muestrario.campos': 'Campos',
  'muestrario.progreso': 'Progreso',
  'muestrario.cuadricula': 'La cuadrícula de notas',
  'muestrario.cuadricula_sub': 'Cómo se verá cada estado de una nota, con la regla de color: el color marca eventos y excepciones, nunca lo normal.',
  'muestrario.boton_primario': 'Subir a Clickedu',
  'muestrario.boton_secundario': 'Exportar',
  'muestrario.boton_peligro': 'Borrar notas',
  'muestrario.boton_desactivado': 'Desactivado',
  'muestrario.abrir_dialogo': 'Abrir un diálogo',
  'muestrario.dialogo_titulo': 'Subir las notas a Clickedu',
  'muestrario.dialogo_texto': 'Se subirán 24 notas al ítem «Examen tema 3» de Matemáticas 3.º A. Las que ya haya en Clickedu se sustituirán.',
  'muestrario.cancelar': 'Cancelar',
  'muestrario.campo_texto': 'Buscar alumno…',
  'muestrario.campo_clase': 'Clase',
  'muestrario.progreso_subiendo': 'Subiendo notas a Clickedu',
  'muestrario.progreso_conectando': 'Conectando con Clickedu',
  'muestrario.col_alumno': 'Alumno',
  'muestrario.col_nota': 'Nota',
  'muestrario.col_estado': 'Qué se ve',
  'muestrario.estado_sin_nota': 'Sin nota: gris, aunque sea la mayoría',
  'muestrario.estado_nota': 'Nota puesta: texto normal',
  'muestrario.estado_suspenso': 'Suspenso: también normal, no es un error',
  'muestrario.estado_dictada': 'Recién dictada: verde, se apaga sola',
  'muestrario.estado_dudosa': 'Dictado dudoso: discontinuo, espera que confirmes',
  'muestrario.estado_invalida': 'Valor que no es nota: rojo',
  'muestrario.dudosa_confirmar': '¿Pérez Gómez, Irene?',
} as const;

export type TranslationKey = keyof typeof es;

export default es;
