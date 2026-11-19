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
  'lista.y': '{primeros} y {ultimo}',
  'lista.y_mas': '{primeros}, {ultimo} y {mas} más',

  // ── Calificar ──
  'calificar.titulo': 'Calificar',
  'calificar.calificados': '{n} / {total} con nota',
  'calificar.elegir_clase': 'Elegir la clase',
  'calificar.no_se_lee': 'No se puede leer: {error}',
  'calificar.no_se_guarda': 'No se ha podido guardar la nota: {error}',

  // ── Exportar (H2) ──
  'exportar.boton': 'Exportar',
  'exportar.ayuda': 'Deja las notas en un Excel para subirlo a Clickedu, en Documentos\\Gestor de Notas\\notas',
  'exportar.hecho': 'Exportadas {n} notas de {total} a {fichero}.',
  'exportar.hecho_una': 'Exportada 1 nota de {total} a {fichero}.',
  'exportar.viejo': 'El exportado anterior está en notas_viejas.',
  'exportar.abrir_carpeta': 'Abrir la carpeta',
  'exportar.hay_invalidas': 'No se exporta: {alumnos} no tienen una nota de 0 a 10. Escribe encima la nota buena o bórrala.',
  'exportar.hay_invalida': 'No se exporta: {alumno} no tiene una nota de 0 a 10. Escribe encima la nota buena o bórrala.',
  'exportar.fallo': 'No se ha podido exportar: {error}',
  'exportar.no_abre': 'No se puede abrir la carpeta: {error}',

  // ── La cuadrícula ──
  'cuadricula.n': 'Nº',
  'cuadricula.alumno': 'Alumno',
  'cuadricula.nota': 'Nota',
  'cuadricula.sin_nombre': 'Sin nombre (ID {id})',
  'cuadricula.no_es_nota': 'No es una nota de 0 a 10',
  'cuadricula.nota_de': 'Nota de {nombre}',
  'cuadricula.mal_no_es_numero': 'No es un número',
  'cuadricula.mal_fuera_de_rango': 'Ha de ir de 0 a 10',
  'cuadricula.mal_decimales': 'Dos decimales como mucho',
  'cuadricula.propuesta': '¿{nota}?',
  'cuadricula.apuntar_propuesta': 'Apuntar un {nota} a {nombre}',
  'cuadricula.ayuda': 'Con coma o con punto. Enter o ↓ guarda y pasa al siguiente, ↑ al anterior, Esc deshace. Cada nota se guarda al momento.',

  // ── El dictado (H3b) ──
  'dictado.oido': 'Se ha oído «{oido}».',
  'dictado.apuntada': 'Apuntado un {nota} a {alumno}.',
  'dictado.deshacer': 'Deshacer',
  'dictado.descartar': 'Descartar',
  'dictado.cerrar': 'Cerrar',
  'dictado.duda_quien': '¿Para quién es el {nota}?',
  'dictado.duda_nota': '¿Qué nota le pongo a {alumno}?',
  'dictado.duda_quien_y_nota': '¿Para quién es, y qué nota?',
  'dictado.duda_si': '¿Le pongo un {nota} a {alumno}?',
  'dictado.motivo_nombre_poco_claro': 'El nombre no se ha oído claro.',
  'dictado.motivo_nombre_parecido': 'Hay dos alumnos que se llaman casi igual: con el segundo apellido no te lo pregunta.',
  'dictado.motivo_nota_reinterpretada': 'La nota se ha oído de una forma que se puede leer de dos maneras.',
  'dictado.motivo_varias_notas': 'Se ha oído más de una nota.',
  'dictado.motivo_numero_de_mas': 'Además de la nota se ha oído otro número.',
  'dictado.no_entendido_sin_nota': 'No se ha apuntado nada: no se ha oído ninguna nota de 0 a 10.',
  'dictado.no_entendido_sin_nombre': 'No se ha apuntado nada: se ha oído la nota, pero no para quién.',
  'dictado.no_entendido_nombre_desconocido': 'No se ha apuntado nada: el nombre no se parece a nadie de esta clase.',
  'dictado.fallo': 'No se ha podido entender lo dictado: {error}',
  'dictado.probar': 'Probar el dictado (solo en desarrollo)',
  'dictado.probar_ejemplo': 'Escribe lo que dirías y pulsa Enter',

  // ── Avisos de una plantilla ──
  'avisos.titulo': 'Al leer la plantilla',
  'aviso.la_fila': 'la fila {fila}',
  'aviso.las_filas': 'las filas {filas} y {ultima}',
  'aviso.las_filas_y_mas': 'las filas {filas}, {ultima} y {mas} más',
  'aviso.columnas_por_posicion': 'No se han reconocido las cabeceras: se ha tomado la primera columna como ID, la segunda como nombre y la tercera como nota.',
  'aviso.notas_con_coma': 'Hay notas escritas con coma, en {filas}: se leen con punto, que es como las acepta Clickedu.',
  'aviso.notas_invalidas': 'Hay valores que no son una nota de 0 a 10, en {filas}. Se enseñan en rojo, tal cual: escribe encima la nota buena o bórralos.',
  'aviso.sin_nombre': 'Hay alumnos sin nombre, en {filas}.',
  'aviso.id_repetido': 'El ID {id} está repetido, en {filas}.',

  // ── Clases ──
  'clases.titulo': 'Clases',
  'clases.intro': 'Las plantillas de Clickedu de tu carpeta, una por clase. Si añades o quitas un Excel, la lista se pone al día sola.',
  'clases.col_clase': 'Clase',
  'clases.col_alumnos': 'Alumnos',
  'clases.col_calificados': 'Con nota',
  'clases.col_estado': 'Avisos',
  'clases.un_aviso': '1 aviso',
  'clases.n_avisos': '{n} avisos',
  'clases.no_se_lee': 'No se puede leer: {error}',
  'clases.abrir_carpeta': 'Abrir la carpeta',
  'clases.cambiar_carpeta': 'Usar otra carpeta',
  'clases.sin_plantillas': 'Todavía no hay ninguna plantilla en tu carpeta',
  'clases.sin_plantillas_ayuda': 'Guarda aquí las plantillas de Clickedu, un Excel por clase. En cuanto añadas una, aparece.',
  'clases.carpeta_no_existe': 'La carpeta de plantillas ya no existe',
  'clases.carpeta_no_existe_ayuda': 'Puede que se haya movido o borrado, o que sea de un disco que no está conectado. Elige otra en Ajustes.',

  // ── Pantallas todavía vacías ──
  'clickedu.titulo': 'Conexión con Clickedu',
  'clickedu.vacia': 'Entra con tu usuario, trae las plantillas de tus asignaturas y sube las notas a un ítem sin pasar por el navegador.',
  'estadisticas.titulo': 'Estadísticas',
  'estadisticas.vacia': 'Media, mediana, aprobados y cómo se reparten las notas de la clase, y lo que te ha ahorrado el dictado.',

  // ── Ajustes ──
  'ajustes.titulo': 'Ajustes',
  'ajustes.plantillas': 'Plantillas',
  'ajustes.plantillas_sub': 'La carpeta donde guardas las plantillas de Clickedu, un Excel por clase. La app las lee y no las cambia nunca.',
  'ajustes.por_defecto': 'la de por defecto',
  'ajustes.cambiar': 'Cambiar de carpeta…',
  'ajustes.volver_por_defecto': 'Volver a la de por defecto',
  'ajustes.llegan': 'El micrófono llega con el dictado y la cuenta de Clickedu con la conexión.',

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
  'muestrario.estado_dudosa': 'Dictado dudoso: discontinuo, espera que elijas',
  'muestrario.estado_invalida': 'Valor que no es nota: rojo',
} as const;

export type TranslationKey = keyof typeof es;

export default es;
