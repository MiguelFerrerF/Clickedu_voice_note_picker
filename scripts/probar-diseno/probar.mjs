/**
 * La prueba de diseño: las pantallas a varios tamaños de ventana, en un
 * navegador, sin Tauri.
 *
 *     npm run probar:diseno
 *
 * **Qué hace.** La de library-app, recortada a lo que hay: arranca su propio
 * Vite (puerto 1430, caché aparte en `node_modules/.vite-diseno`, para no
 * pisarse con un `tauri dev` abierto en el 1420), abre la interfaz en el
 * **Edge del sistema** —el mismo motor que el WebView2 de la app, sin
 * descargar navegadores— y simula el backend de Tauri. En cada tamaño recorre
 * las pantallas y mide si algo se sale de la ventana o va cortado.
 *
 * **Qué deja.** Las capturas en `capturas/` (no se versionan), para mirarlas,
 * y la lista de fallos por consola. Sale con error si hay alguno.
 *
 * **Se pasa tras todo cambio visible y las capturas se miran.** Una pantalla
 * nueva se prueba añadiendo una entrada a `PRUEBAS`. El scroll horizontal a
 * propósito se marca con `data-scroll-horizontal`.
 */

import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

import {chromium} from 'playwright';
import {createServer} from 'vite';

const AQUI = path.dirname(fileURLToPath(import.meta.url));
const RAIZ = path.resolve(AQUI, '..', '..');
const CAPTURAS = path.join(AQUI, 'capturas');
const PUERTO = 1430;

/** El mínimo de la ventana (`tauri.conf.json`), dos intermedios y uno grande. */
const TAMANOS = [
  [960, 640],
  [1100, 700],
  [1280, 800],
  [1920, 1080],
];

/**
 * El backend falso: lo justo para que las pantallas pinten. `invoke` y
 * `listen` de `@tauri-apps/api` pasan por `window.__TAURI_INTERNALS__`. Se
 * inyecta en la página con `addInitScript`, así que **no puede usar nada de
 * fuera de la función**: solo su argumento.
 *
 * Los datos son **inventados** (nunca alumnos reales en el repo) y buscan los
 * casos difíciles: nombres largos, letras de fuera (Ș, Ł), una plantilla que
 * no se lee, notas que no son nota, un alumno sin nombre. `carpeta` elige la
 * carpeta de plantillas: `llena`, `vacia` o `no_existe`; `modelo`, si el
 * modelo de voz ya está descargado; `auriculares`, si el micrófono elegido
 * en Ajustes son unos auriculares que no están conectados; `actualizacion`,
 * si hay una versión nueva al arrancar (H7); `v1`, si la v1.3.4 está
 * instalada: `traida` (con una renombrada y la carpeta cambiada) o `fallo`;
 * `clickedu`, si ya hay una cuenta de Clickedu guardada (H4); `items`, si
 * las clases traen sus ítems de Clickedu (H6): sin ellos, es quien no se
 * conecta.
 */
function simular({carpeta, modelo, auriculares, actualizacion, v1, clickedu, items}) {
  let n = 1;
  const ruta = 'C:\\Users\\profesora\\Documents\\Gestor de Notas\\plantillas';
  const bien = (alumnos, calificados, avisos = 0) => ({tipo: 'bien', alumnos, calificados, avisos});
  const largo = "Ciències Aplicades a l'Activitat Professional - 4t ESO B i C";
  const CLASES = [
    {fichero: 'biologia_1_ab_bach.xlsx', nombre: 'Biología 1.º AB Bachillerato', lectura: bien(18, 0)},
    {fichero: `${largo}.xlsx`, nombre: largo, lectura: bien(31, 12)},
    {fichero: 'fisica_2_bach.xlsx', nombre: 'Física 2.º Bachillerato', lectura: {tipo: 'mal', error: 'no es un Excel (.xlsx o .xls)'}},
    {fichero: 'matematicas_3_a_eso.xlsx', nombre: 'Matemáticas 3.º A ESO', lectura: bien(26, 8, 3)},
    {fichero: 'matematicas_3_b_eso.xlsx', nombre: 'Matemáticas 3.º B ESO', lectura: bien(25, 25)},
    {fichero: 'Pendiente - Física y Química - 2º ESO.xlsx', nombre: 'Pendiente - Física y Química - 2º ESO', lectura: bien(3, 1)},
  ].map((c) => ({...c, pendiente: c.fichero.startsWith('Pendiente - ')}));
  // H6: los ítems, por evaluación, con los nombres de Clickedu («1ª
  // evaluación»; los números, distintos por etapa). En Matemáticas 3.º A, la
  // 1.ª ya subida, la 2.ª en curso (con un nombre largo y uno sin
  // ponderación) y la 3.ª sin ítems; Biología, de Bachillerato, con «JUNIO»
  // y sin ítems aún en la 2.ª; Matemáticas 3.º B, sin traer.
  // Sin decir cuántos solo en Clickedu, los que no se han calificado en la app.
  const item = (id, nombre, ponderacion, enClickedu = 0, calificados = 0, publicadas = Math.max(0, enClickedu - calificados), porSubir = calificados, dentroDe = null) =>
    ({id, nombre, ponderacion, dentroDe, enClickedu, calificados, publicadas, porSubir});
  const ev = (id, nombre, enCurso, lista) => ({id, nombre, enCurso, items: lista});
  const ITEMS = {
    'matematicas_3_a_eso.xlsx': [
      ev('331', '1ª evaluación', false, [
        item('41810', 'Examen tema 1', 40, 26, 26),
        item('41811', 'Examen tema 2', 40, 26, 26),
        item('41812', 'Cuaderno', 20, 25),
      ]),
      ev('332', '2ª evaluación', true, [
        item('42690', 'Examen tema 3', 40, 8, 8, 4),
        item('42691', 'Proyecto: la estadística del patio, con encuestas a todos los cursos del instituto', 20),
        item('42692', 'Cuaderno', 10, 4),
        item('42694', 'Cuaderno: problemas', 3.33, 0, 0, 0, 0, {nombre: 'Cuaderno', ponderacion: 33.33}),
        item('42693', 'Actitud en clase', null),
      ]),
      ev('333', '3ª evaluación', false, []),
    ],
    [`${largo}.xlsx`]: [
      ev('331', '1ª evaluación', false, [item('41900', 'Pràctica de laboratori 1', 50, 31)]),
      ev('332', '2ª evaluación', true, [item('42710', 'Pràctica de laboratori 2', 50, 0, 12), item('42711', 'Examen', 50)]),
      ev('333', '3ª evaluación', false, []),
    ],
    'biologia_1_ab_bach.xlsx': [
      ev('318', '1ª evaluación', false, [item('41700', 'Examen', 70, 18), item('41701', 'Informe de prácticas', 30, 17)]),
      ev('319', '2ª evaluación', true, []),
      ev('320', '3ª evaluación', false, []),
      ev('321', 'JUNIO', false, []),
    ],
    'Pendiente - Física y Química - 2º ESO.xlsx': [
      ev('334', '1ª evaluación', false, []),
      ev('335', '2ª evaluación', true, [item('42800', 'Cuadernillo de recuperación', 100, 0, 1)]),
      ev('336', '3ª evaluación', false, []),
    ],
  };
  if (items)
    for (const c of CLASES) if (ITEMS[c.fichero]) c.items = {traidos: 1795392000, evaluaciones: ITEMS[c.fichero]};
  const NOMBRES = [
    'Castro Molina, Lucía', 'Martínez Ruiz, Hugo', 'Ramos Ortega, Martina', 'Navarro Gil, Mateo',
    'Pérez Gómez, Irene', 'Ionescu, Ștefan', 'Nowak, Łukasz', 'Fernández de la Fuente Villalobos, María Inmaculada',
    'Sánchez Díaz, Leo', 'Romero Serrano, Daniela', 'Torres Vidal, Álex', 'Domínguez Blanco, Sara',
    'Vázquez Ramos, Pablo', 'Gil Cortés, Carmen', 'Ortiz Rubio, Marco', 'Moreno Prieto, Julia',
    'Herrera Santos, Adrián', 'Iglesias Peña, Noa', 'Marín Lozano, Gael', 'Garrido León, Vega',
    '', 'Rey Cabrera, Olivia', 'Flores Pastor, Bruno', 'Vega Calvo, Lola',
    'Santana Ruiz, Thiago', 'Ferrer Soler, Abril',
  ];
  const NOTAS = {1: 8.7, 2: 3.4, 4: 6.8, 5: 'NP', 7: 10, 9: 5, 11: '12', 12: 7.25, 14: 4.5, 17: 9.1};
  const nota = (i) => {
    const v = NOTAS[i];
    if (v === undefined) return {tipo: 'vacia'};
    return typeof v === 'number' ? {tipo: 'valor', valor: v} : {tipo: 'invalida', valor: v};
  };
  const PLANTILLA = {
    alumnos: NOMBRES.map((nombre, i) => ({fila: i + 2, id: String(20231 + i), nombre, nota: nota(i), clave: String(20231 + i)})),
    columnas: {
      hoja: 'Full1',
      filaCabecera: 1,
      id: {indice: 0, cabecera: 'Codi'},
      nombre: {indice: 1, cabecera: 'Alumne'},
      nota: {indice: 2, cabecera: 'Nota'},
    },
    avisos: [
      {tipo: 'notas_con_coma', filas: [14]},
      {tipo: 'notas_invalidas', filas: [7, 13]},
      {tipo: 'sin_nombre', filas: [22]},
    ],
  };
  /** Las otras clases: las 18 primeras, sin nada que avisar ni notas. */
  const OTRA = {
    ...PLANTILLA,
    avisos: [],
    alumnos: PLANTILLA.alumnos.slice(0, 18).map((a) => ({...a, nota: {tipo: 'vacia'}})),
  };
  const perdida = carpeta === 'no_existe';
  const lista = () => ({
    carpeta: {ruta: perdida ? 'E:\\Clase\\plantillas' : ruta, porDefecto: !perdida, existe: !perdida},
    clases: carpeta === 'llena' ? CLASES : [],
  });

  // H5: lo traído de Clickedu, inventado. Las dudas respondidas pasan a
  // apartadas, como en `planear`.
  let respuestasTraer = {};
  const materia = (id, nombre, grupo, curso, fichero, estado, extra = {}) => ({
    materia: {id, nombre, grupo, curso},
    fichero,
    pendiente: false,
    retoques: [],
    avisos: [],
    estado,
    estado_evaluaciones: estado.alumnos !== undefined ? {estado: 'bien'} : null,
    ...extra,
  });
  const BIO = 'Biología y Geología - 3º ESO A.xlsx';
  const CAAP = `${largo.replace(' - 4t ESO B i C', '')} - 4º ESO B.xlsx`;
  const MATERIAS_TRAER = [
    materia('4102', 'Biología y Geología', '3º E.S.O.-A', 'Tercero de ESO', BIO, {tipo: 'nueva', alumnos: 26}),
    materia('4107', 'Física y Química', 'FÍSICA Y QUÍMICA 1', 'Primero de Bachillerato', 'Física y Química - 1º Bach (FÍSICA Y QUÍMICA 1).xlsx', {tipo: 'igual', alumnos: 31}),
    materia('4111', "Ciències Aplicades a l'Activitat Professional", '4º E.S.O.-B', 'Cuarto de ESO', CAAP, {tipo: 'cambia', alumnos: 24}, {
      estado_evaluaciones: {estado: 'degradado', motivo: 'el desplegable de evaluaciones no está: se han leído de los enlaces'},
    }),
    materia('4120', 'Matemáticas', '2º Bach', 'Segundo de Bachillerato', 'Matemáticas - 2º Bach.xlsx', {
      tipo: 'fallo',
      fallo: {tipo: 'bajar', fallo: {tipo: 'no_es_excel', estado: 500, contenido: 'text/html'}},
    }),
    materia('4131', 'Física y Química 2º', 'PENDIENTE FÍSICA Y QUÍMICA 2ºESO', 'Tercero de ESO', 'Pendiente - Física y Química - 2º ESO.xlsx', {tipo: 'nueva', alumnos: 3}, {pendiente: true}),
    materia('4132', 'Física y Química 1º', 'PENDIENTE FÍSICA Y QUÍMICA 1º', 'Segundo de Bachillerato', 'Pendiente - Física y Química - 1º Bach.xlsx', {tipo: 'vacia'}, {pendiente: true}),
  ];
  const planTraer = () => {
    const apartadas = [
      {fichero: 'biologia_3_a_eso.xlsx', notas: {a: BIO, por: 'alumnos', pasan: 12, sin_alumno: 1, sin_id: 0}, descartadas: 0},
      {fichero: 'matematicas_3_b_eso.xlsx', notas: null, descartadas: 0},
    ];
    const dudas = [];
    for (const [fichero, notas] of [['prueba_voz_inventada.xlsx', 15], ['biologia_1_ab_bach.xlsx', 7]]) {
      const r = respuestasTraer[fichero];
      if (!r) dudas.push({fichero, notas, candidatas: [BIO, CAAP]});
      else if (r.tipo === 'ninguna') apartadas.push({fichero, notas: null, descartadas: notas});
      else apartadas.push({fichero, notas: {a: r.a, por: 'respuesta', pasan: notas, sin_alumno: 0, sin_id: 0}, descartadas: 0});
    }
    return {materias: MATERIAS_TRAER, apartadas, dudas};
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {unregisterListener() {}};
  // Los callbacks, por id: los `Channel` reciben por aquí (el nivel del micrófono).
  const callbacks = {};
  let midiendo;
  let subirSinItem = false;
  let microfono = auriculares ? {id: 'wasapi:{0.0.1.00000000}.{sony}', nombre: 'Auriculares (WH-1000XM4)'} : null;
  window.__TAURI_INTERNALS__ = {
    transformCallback: (callback) => {
      callbacks[n] = callback;
      return n++;
    },
    invoke: async (cmd, args) => {
      switch (cmd) {
        case 'plugin:event|listen':
          return n++;
        case 'listar_clases':
          return lista();
        case 'leer_clase':
          // En la de los avisos, una nota ya puesta en la sesión y otra quitada.
          // En su «Examen tema 3» (H6), las de Clickedu: solo allí (también
          // un texto, «NP», y un alumno sin nombre), iguales y dos que
          // cambiarían al subir; una de otro grupo, que no cuenta.
          return args.fichero === 'matematicas_3_a_eso.xlsx'
            ? {
                plantilla: PLANTILLA,
                puestas: {20233: 6.5, 20232: null},
                enClickedu:
                  args.item === '42690'
                    ? {20231: '7.50', 20232: '4', 20233: '6.5', 20235: '6', 20238: '9.5', 20240: '5.00', 20251: 'NP', 20253: '8', 99999: '3'}
                    : args.item
                      ? {}
                      : null,
              }
            : {plantilla: OTRA, puestas: {}, enClickedu: args.item ? {} : null};
        case 'poner_nota':
          return args.nota;
        case 'devolver_nota':
          return null;
        case 'entender_frase': {
          // H3b: lo seguro, lo dudoso (con el nombre más largo y dos notas,
          // cuatro opciones) y lo que no se entiende.
          const oido = args.texto;
          const id = String(n++);
          if (/thiago/i.test(oido)) return {id, oido, tipo: 'seguro', alumno: '20255', nota: 9.5};
          if (/luc[ií]a/i.test(oido)) return {id, oido, tipo: 'seguro', alumno: '20231', nota: 8};
          if (/hugo/i.test(oido))
            return {id, oido, tipo: 'duda', alumnos: ['20232'], notas: [6, 6.5], motivos: ['varias_notas']};
          if (/mar[ií]a/i.test(oido))
            return {
              id,
              oido,
              tipo: 'duda',
              alumnos: ['20238', '20233'],
              notas: [7.5, 7.3],
              motivos: ['nombre_parecido', 'nota_reinterpretada'],
            };
          return {id, oido, tipo: 'no_entendido', motivo: 'sin_nota'};
        }
        case 'hay_modelo':
          return modelo;
        case 'descargar_modelo':
          // Se queda descargando: la barra sin porcentaje.
          return new Promise(() => {});
        case 'listar_microfonos':
          return {
            disponibles: [
              {id: 'wasapi:{0.0.1.00000000}.{realtek}', nombre: 'Micrófono (Realtek(R) Audio)'},
              {id: 'wasapi:{0.0.1.00000000}.{camara}', nombre: 'Micrófono (Cámara HD integrada)'},
              {
                id: 'wasapi:{0.0.1.00000000}.{jabra}',
                nombre: 'Micrófono de los auriculares (Jabra Evolve2 30 Hands-Free AG Audio)',
              },
            ],
            deWindows: 'wasapi:{0.0.1.00000000}.{realtek}',
            elegido: microfono,
          };
        case 'elegir_microfono':
          microfono = args.microfono;
          return null;
        case 'abrir_microfono':
          return {nombre: 'Micrófono (Realtek(R) Audio)', falta: microfono?.nombre ?? null};
        case 'empezar_dictado': {
          // Una voz que sube y baja, y se queda a media altura para la captura.
          const avisar = callbacks[args.nivel.id];
          let index = 0;
          midiendo = setInterval(() => {
            avisar({index, message: index < 6 ? [0.3, 0.7, 0.85, 0.5, 0.75, 0.6][index] : 0.62});
            index++;
          }, 50);
          return null;
        }
        case 'cancelar_dictado':
          clearInterval(midiendo);
          return null;
        case 'terminar_dictado':
          clearInterval(midiendo);
          // H3a: lo que tarda Whisper, y lo oído, repetido en bucle (se
          // enseña una vez).
          await new Promise((r) => setTimeout(r, 600));
          return {
            id: String(n++),
            oido: 'Lucía Castro, nueve y medio Lucía Castro, nueve y medio Lucía Cas',
            repetida: 'Lucía Castro, nueve y medio',
            tipo: 'seguro',
            alumno: '20231',
            nota: 9.5,
          };
        case 'resumen_registro':
          // D8: con casos en la carpeta llena; vacío en las demás.
          return carpeta === 'llena'
            ? {dictados: 142, seguros: 109, preguntados: 27, noEntendidos: 6, corregidos: 3, fallos: 2, desde: 1795046400}
            : {dictados: 0, seguros: 0, preguntados: 0, noEntendidos: 0, corregidos: 0, fallos: 0, desde: null};
        case 'preparar_informe':
          return `${ruta.replace(/plantillas$/, 'informes')}\\informe-2026-11-19.json`;
        case 'buscar_actualizacion':
          // H7: con titular largo y los cuatro tipos de cambio.
          return actualizacion
            ? {
                version: '2.1.0',
                actual: '2.0.0',
                publicada: '2026-12-03',
                titular:
                  'El dictado entiende mejor los nombres compuestos y te pregunta menos; y Clickedu, por fin desde la app.',
                entradas: [
                  {tipo: 'feat', texto: 'Conectar con Clickedu y traer las plantillas sin pasar por el navegador.'},
                  {tipo: 'fix', texto: 'Los nombres con dos apellidos compuestos ya no se confunden.'},
                  {tipo: 'perf', texto: 'La primera frase dictada tarda la mitad.'},
                  {tipo: 'chore', texto: 'El registro guarda seis meses, como antes.'},
                ],
              }
            : null;
        case 'traer_de_la_v1':
          if (v1 === 'fallo')
            throw 'no se puede leer C:\\Users\\profesora\\AppData\\Local\\GestorNotasClickedu\\notas: Acceso denegado. (os error 5)';
          return v1 === 'traida'
            ? {
                traido: {clases: 7, notas: 12, ya_estaban: 0, renombrados: ['matematicas_3_a_eso (de la v1).xlsx']},
                plantillas: ruta,
                notas: ruta.replace(/plantillas$/, 'notas'),
                carpeta_cambiada: true,
                pide_permiso: true,
              }
            : null;
        case 'quitar_la_v1':
          return null;
        case 'descargar_actualizacion':
          // Se queda descargando: la barra sin porcentaje.
          return new Promise(() => {});
        // H4: la cuenta de Clickedu. La contraseña «mala» no entra, y
        // cambiar el archivo de una cuenta guardada da el de «no lo acepta».
        case 'cuenta_clickedu':
          return clickedu
            ? {usuario: 'profesora.fernandez.villalobos', archivo: 'archivo_seguridad_3062.txt', elegido: 1795132800}
            : null;
        case 'elegir_archivo_seguridad':
          return 'C:\\Users\\profesora\\Downloads\\archivo_seguridad_de_la_profesora_curso_2026_2027_3062.txt';
        case 'entrar_clickedu':
          await new Promise((r) => setTimeout(r, 400));
          if (args.contrasena === 'mala') throw {tipo: 'usuario'};
          if (args.archivo && !args.usuario) throw {tipo: 'archivo'};
          if (args.usuario === 'sinred')
            throw {tipo: 'sin_conexion', detalle: 'error sending request: client error (Connect): tcp connect error: Se ha agotado el tiempo de espera (os error 10060)'};
          return {
            usuario: args.usuario ?? 'profesora.fernandez.villalobos',
            archivo: 'archivo_seguridad_de_la_profesora_curso_2026_2027_3062.txt',
            elegido: 1795219200,
          };
        case 'olvidar_cuenta_clickedu':
          return null;
        // H5: traer las plantillas. La primera vez, con una duda; una que
        // se lee de otra manera, una que falla, una pendiente vacía y, al
        // hacerlo, una abierta en Excel que no se puede apartar.
        case 'traer_plantillas': {
          if (!clickedu) throw {tipo: 'sin_entrar'};
          respuestasTraer = {};
          const avisar = callbacks[args.avance.id];
          for (let i = 1; i <= 3; i++) {
            await new Promise((r) => setTimeout(r, 60));
            avisar({index: i - 1, message: {hechas: i, total: 6}});
          }
          await new Promise((r) => setTimeout(r, 1500));
          return {
            plan: planTraer(),
            hecho: null,
            primeraVez: true,
            estadoMaterias: {estado: 'bien'},
            cambios: ['materias: el formulario «buscador» ya no está'],
          };
        }
        case 'replanear_traer':
          respuestasTraer = args.respuestas;
          return planTraer();
        case 'confirmar_traer': {
          respuestasTraer = args.respuestas;
          await new Promise((r) => setTimeout(r, 300));
          const plan = planTraer();
          const abierta = 'matematicas_3_b_eso.xlsx';
          return {
            plan,
            hecho: {
              escritas: plan.materias.filter((m) => ['nueva', 'cambia'].includes(m.estado.tipo)).map((m) => m.fichero),
              movidas: plan.apartadas
                .filter((a) => a.fichero !== abierta)
                .map((a) => ({fichero: a.fichero, a: `antiguas\\2026-11-22\\${a.fichero}`})),
              pasan: plan.apartadas.reduce((n, a) => n + (a.notas?.pasan ?? 0), 0),
              fallos: [{tipo: 'mover', fichero: abierta, error: 'El proceso no tiene acceso al archivo (os error 32)'}],
            },
            primeraVez: true,
            estadoMaterias: null,
            cambios: [],
          };
        }
        case 'cancelar_traer':
          return null;
        // H6: subir. En «Examen tema 3», dos que cambiarían y sale bien; sin
        // ítem (se elige al subir), sin nada que pisar y falla al importar.
        case 'preparar_subida': {
          await new Promise((r) => setTimeout(r, 400));
          subirSinItem = !args.item;
          return {
            item: subirSinItem ? 'Proyecto de investigación sobre la fotosíntesis' : 'Examen tema 3',
            evaluacion: '2ª evaluación',
            alumnos: 26,
            conNota: 9,
            nuevas: subirSinItem ? 9 : 5,
            iguales: subirSinItem ? 0 : 2,
            cambiarian: subirSinItem
              ? []
              : [
                  {nombre: 'Alonso Ferrández de la Fuente, María del Carmen', antes: '7.50', nueva: 8},
                  {nombre: 'Castro Molina, Lucía', antes: 'NP', nueva: 5.25},
                ],
            // Las que solo están en Clickedu van en el Excel; un «NP», no puede.
            seQuedan: subirSinItem ? 0 : 4,
            seBorrarian: subirSinItem ? [] : [{nombre: 'Sin nombre (ID 20251)', antes: 'NP'}],
            excel: `${ruta.replace(/plantillas$/, 'notas')}\\notas_matematicas_3_a_eso.xlsx`,
          };
        }
        case 'confirmar_subida': {
          const avisar = callbacks[args.avance.id];
          const pasos = ['subir', 'importar', 'comprobar'];
          for (let i = 0; i < (subirSinItem ? 2 : 3); i++) {
            avisar({index: i, message: pasos[i]});
            await new Promise((r) => setTimeout(r, 700));
          }
          if (subirSinItem)
            throw {tipo: 'cambiado', paso: 'importar', motivo: 'el fichero subido no tiene el enlace para importarlo'};
          return {entraron: 8, noEntraron: ['Castro Molina, Lucía'], borradas: ['Nowak, Łukasz'], avisos: []};
        }
        case 'cancelar_subida':
          return null;
        case 'exportar_clase':
          return {
            ruta: `${ruta.replace(/plantillas$/, 'notas')}\\notas_${args.fichero.replace(/\.xlsx?$/, '')}.xlsx`,
            conNota: 2,
            alumnos: 18,
            viejo: 'notas_viejas',
          };
        default:
          return null;
      }
    },
  };
}

// ─────────────────────────────────────────────────────────────────────────────
// Lo que se mide
// ─────────────────────────────────────────────────────────────────────────────

/** ¿Se sale algo de la ventana, o va cortado? Devuelve los fallos en palabras. */
function medir(page, donde) {
  return page.evaluate((donde) => {
    const W = innerWidth;
    const H = innerHeight;
    const fallos = [];
    const px = (n) => Math.round(n);

    // Los menús van por portal: hijos de <body> con `position: fixed`.
    for (const menu of document.body.children) {
      if (getComputedStyle(menu).position !== 'fixed') continue;
      const r = menu.getBoundingClientRect();
      if (r.left < 0 || r.right > W) fallos.push(`${donde}: un menú se sale en horizontal (${px(r.left)}–${px(r.right)} de ${W})`);
      if (r.bottom > H) fallos.push(`${donde}: un menú se sale por abajo (${px(r.bottom)} de ${H})`);
      if (menu.scrollWidth > menu.clientWidth + 1) fallos.push(`${donde}: un menú tiene scroll horizontal`);
    }

    if (document.documentElement.scrollWidth > W) fallos.push(`${donde}: la página tiene scroll horizontal`);

    /**
     * El scroll horizontal solo vale donde la app lo marca como a propósito
     * (`data-scroll-horizontal`). En cualquier otro sitio es algo que no cabe.
     */
    const aProposito = (el) => el.closest('[data-scroll-horizontal]') !== null;
    const visible = (el) => el.offsetParent !== null || getComputedStyle(el).position === 'fixed';
    for (const el of document.querySelectorAll('body *')) {
      if (aProposito(el) || el.clientWidth === 0 || !visible(el)) continue;
      const cs = getComputedStyle(el);
      if ((cs.overflowX === 'auto' || cs.overflowX === 'scroll') && el.scrollWidth > el.clientWidth + 1)
        fallos.push(`${donde}: algo no cabe y sale scroll horizontal en <${el.tagName.toLowerCase()}> (${el.scrollWidth} de ${el.clientWidth})`);

      /**
       * Propio del Gestor de Notas: **texto cortado a pelo**. Un elemento con
       * texto propio que recorta lo que no cabe (`overflow: hidden`) sin
       * puntos suspensivos se lee como otra cosa —«Mantén ESPACIO para
       * dicta», que es lo que hacía la app de Python—. Con `ellipsis` vale:
       * se ve que falta algo.
       */
      const tieneTexto = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim());
      if (tieneTexto && cs.overflowX === 'hidden' && cs.textOverflow !== 'ellipsis' && el.scrollWidth > el.clientWidth + 1)
        fallos.push(`${donde}: texto cortado sin «…»: «${el.textContent.trim().slice(0, 40)}»`);
    }

    for (const b of document.querySelectorAll('button')) {
      const r = b.getBoundingClientRect();
      if (r.width === 0 || getComputedStyle(b).visibility === 'hidden') continue;
      if (aProposito(b)) continue;
      const nombre = b.textContent?.trim() || b.getAttribute('aria-label') || '(sin texto)';
      if (r.right > W + 0.5 || r.left < -0.5) fallos.push(`${donde}: el botón «${nombre}» se sale (${px(r.left)}–${px(r.right)} de ${W})`);
      if (b.scrollWidth > b.clientWidth + 1 && getComputedStyle(b).textOverflow !== 'ellipsis')
        fallos.push(`${donde}: el botón «${nombre}» va cortado`);
    }
    return fallos;
  }, donde);
}

// ─────────────────────────────────────────────────────────────────────────────
// Las pruebas
// ─────────────────────────────────────────────────────────────────────────────

/** Pulsa una entrada de la barra lateral y espera a que su pantalla se vea. */
async function ir(page, rotulo, id) {
  await page.locator('aside').getByRole('button', {name: rotulo, exact: true}).click();
  await page.locator(`[data-pantalla="${id}"]`).waitFor({state: 'visible'});
}

const PRUEBAS = [
  {
    nombre: 'calificar',
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      // Sin clase elegida enseña la primera; desde el desplegable, otra.
      const main = page.locator('[data-pantalla="calificar"]');
      await main.getByRole('row').first().waitFor();
      // HUI: sin ninguna nota, «Exportar» en gris y sin pulsarse.
      if (!(await main.getByRole('button', {name: 'Exportar'}).isDisabled()))
        throw new Error('«Exportar» se puede pulsar sin ninguna nota');
      await paso('primera');
      // HUI: las teclas, con el ⓘ de encima de la tabla.
      await main.getByRole('button', {name: 'Las teclas'}).click();
      await main.getByRole('region', {name: 'Las teclas'}).waitFor();
      await paso('teclas');
      await main.getByRole('button', {name: 'Cerrar las teclas'}).click();
      await main.getByRole('button', {name: 'Biología 1.º AB Bachillerato'}).click();
      await page.waitForTimeout(150);
      await paso('desplegable');
      await page.getByRole('button', {name: 'Matemáticas 3.º A ESO', exact: true}).last().click();
      // HUI: los avisos, plegados en una línea que se abre al pulsarla.
      const avisos = main.getByRole('button', {name: '3 avisos al leer la plantilla'});
      await avisos.waitFor();
      // El galón que gira y «Exportar», que se enciende: ya quietos.
      await page.waitForTimeout(250);
      await paso('avisos');
      await avisos.click();
      await main.getByText('Hay alumnos sin nombre').waitFor();
      await page.waitForTimeout(250);
      await paso('avisos-abiertos');
    },
  },
  {
    // H2: escribir notas, una que no es nota, y exportar bien y mal.
    nombre: 'notas',
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      const main = page.locator('[data-pantalla="calificar"]');
      const campo = (nombre) => main.getByRole('textbox', {name: `Nota de ${nombre}`});
      await campo('Castro Molina, Lucía').fill('7,5');
      await page.keyboard.press('Enter');
      if (await main.getByRole('button', {name: 'Exportar'}).isDisabled())
        throw new Error('«Exportar» no se puede pulsar con una nota');
      await page.keyboard.type('4');
      await page.keyboard.press('Enter');
      await page.keyboard.type('12');
      await page.keyboard.press('Enter');
      await main.getByText('Ha de ir de 0 a 10').waitFor();
      await paso('escritas');
      // Supr vacía la celda: lo escrito a medias, también.
      await page.keyboard.press('Delete');
      // HUI: al siguiente sin nota, desde el último tocado.
      await main.getByRole('button', {name: 'Siguiente sin nota'}).click();
      await page.waitForTimeout(400);
      await paso('siguiente-sin-nota');
      await main.getByRole('button', {name: 'Exportar'}).click();
      await main.getByText('Exportadas 2 notas').waitFor();
      await paso('exportado');
      await main.getByRole('button', {name: 'Biología 1.º AB Bachillerato'}).click();
      await page.getByRole('button', {name: 'Matemáticas 3.º A ESO', exact: true}).last().click();
      await campo('Castro Molina, Lucía').waitFor();
      await main.getByRole('button', {name: 'Exportar'}).click();
      await main.getByText('No se exporta').waitFor();
      await paso('no-se-exporta');
    },
  },
  {
    // H3b: lo dictado, escrito en el campo de desarrollo. Seguro (en verde,
    // abajo del todo, con la fila), deshacerlo, una duda con cuatro opciones
    // y dos apuntadas debajo, elegir con el teclado (las últimas tres), una
    // duda de un solo alumno, descartarla con Esc y lo que no se entiende,
    // encima de las apuntadas.
    nombre: 'dictado',
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      const main = page.locator('[data-pantalla="calificar"]');
      await main.getByRole('button', {name: 'Biología 1.º AB Bachillerato'}).click();
      await page.getByRole('button', {name: 'Matemáticas 3.º A ESO', exact: true}).last().click();
      await main.getByRole('textbox', {name: 'Nota de Castro Molina, Lucía'}).waitFor();
      const dictar = async (frase) => {
        await main.getByPlaceholder('Escribe lo que dirías y pulsa Enter').fill(frase);
        await page.keyboard.press('Enter');
      };
      await dictar('Thiago nueve y medio');
      await main.getByRole('button', {name: 'Deshacer el 9.5'}).waitFor();
      await page.waitForTimeout(700);
      await paso('apuntada');
      await main.getByRole('button', {name: 'Deshacer el 9.5'}).click();
      await page.waitForTimeout(700);
      await paso('deshecha');
      await dictar('Thiago nueve y medio');
      await dictar('Lucía ocho');
      await main.getByRole('button', {name: 'Deshacer el 8'}).waitFor();
      await dictar('María siete con treinta');
      await main.getByRole('alertdialog').waitFor();
      await page.waitForTimeout(700);
      await paso('duda');
      // Fuera del campo de probar: escribiendo, las cifras son de la nota.
      await page.evaluate(() => document.activeElement.blur());
      await page.keyboard.press('1');
      await main.getByRole('button', {name: 'Deshacer el 7.5'}).waitFor();
      await page.waitForTimeout(700);
      await paso('elegida');
      await dictar('Hugo seis, seis y medio');
      await main.getByRole('alertdialog').waitFor();
      await paso('duda-un-alumno');
      await page.evaluate(() => document.activeElement.blur());
      await page.keyboard.press('Escape');
      await main.getByRole('alertdialog').waitFor({state: 'detached'});
      await dictar('Way this');
      await main.getByText('No se ha apuntado nada').waitFor();
      await paso('no-entendida');
    },
  },
  {
    // H3a: el push-to-talk con ESPACIO. El reposo, escuchando, entendiendo,
    // lo apuntado y una pulsación corta.
    nombre: 'voz',
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      const main = page.locator('[data-pantalla="calificar"]');
      await main.getByText('di el alumno y la nota', {exact: false}).waitFor();
      await paso('reposo');
      await page.keyboard.down('Space');
      await main.getByText('Escuchando').waitFor();
      await main.getByRole('meter').waitFor();
      await page.waitForTimeout(400);
      await paso('escuchando');
      await page.keyboard.up('Space');
      await main.getByText('Entendiendo').waitFor();
      await paso('entendiendo');
      await main.getByRole('button', {name: 'Deshacer el 9.5'}).waitFor();
      await page.waitForTimeout(700);
      await paso('apuntada');
      await page.keyboard.down('Space');
      await page.keyboard.up('Space');
      await main.getByText('mientras hablas').waitFor();
      await paso('corto');
    },
  },
  {
    // H3a: sin el modelo de voz, la oferta de descargarlo y su avance.
    nombre: 'sin-modelo',
    modelo: false,
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      const main = page.locator('[data-pantalla="calificar"]');
      await main.getByRole('button', {name: 'Descargar el modelo de voz'}).waitFor();
      await paso('falta');
      await main.getByRole('button', {name: 'Descargar el modelo de voz'}).click();
      await main.getByRole('progressbar').waitFor();
      await paso('descargando');
    },
  },
  {
    nombre: 'clases',
    async probar(page, paso) {
      await ir(page, 'Clases', 'clases');
      await page.locator('.fila-clase').first().waitFor();
      await paso('lista');
      // Pulsar una clase la abre en Calificar.
      await page.locator('.fila-clase').filter({hasText: 'Matemáticas 3.º B ESO'}).click();
      await page.locator('[data-pantalla="calificar"] h1', {hasText: 'Matemáticas 3.º B ESO'}).waitFor();
      await paso('abierta');
    },
  },
  {
    // H6: las clases con sus ítems de Clickedu. Desplegar una (la
    // evaluación en curso, abierta), abrir otra evaluación, pulsar un ítem
    // y, en Calificar, el desplegable del ítem; volver a «Sin ítem».
    nombre: 'clases-items',
    items: true,
    async probar(page, paso) {
      await ir(page, 'Clases', 'clases');
      const main = page.locator('[data-pantalla="clases"]');
      // La de hoy, elegida sola, con las clases plegadas; se despliegan dos.
      await main.getByRole('radio', {name: /2ª evaluación/, checked: true}).waitFor();
      await paso('hoy');
      await main.locator('.fila-clase').filter({hasText: 'Matemáticas 3.º A ESO'}).click();
      await main.locator('.fila-clase').filter({hasText: 'Ciències Aplicades'}).click();
      await main.getByRole('button', {name: /Examen tema 3/}).waitFor();
      await page.waitForTimeout(200);
      await paso('desplegadas');
      await main.getByRole('radio', {name: /1ª evaluación/}).click();
      await main.getByRole('button', {name: /Examen tema 1/}).waitFor();
      await paso('otra-evaluacion');
      await main.getByRole('radio', {name: /JUNIO/}).click();
      await main.getByText('Sin esta evaluación').first().waitFor();
      await paso('junio');
      await main.getByRole('radio', {name: /2ª evaluación/}).click();
      await main.getByRole('button', {name: /Examen tema 3/}).click();
      const calificar = page.locator('[data-pantalla="calificar"]');
      await calificar.locator('h1', {hasText: 'Examen tema 3'}).waitFor();
      await calificar.getByText('2ª evaluación · 40 %').waitFor();
      await calificar.getByText('8 en Clickedu', {exact: false}).waitFor();
      await paso('en-un-item');
      // Las que cambiarían al subir: lleva a la siguiente.
      await calificar.getByRole('button', {name: '2 cambiarían'}).click();
      await page.waitForTimeout(300);
      await paso('cambiarian');
      await calificar.getByRole('button', {name: 'Examen tema 3'}).click();
      await page.waitForTimeout(150);
      await paso('desplegable-item');
      await page.getByRole('button', {name: /^Proyecto/}).last().click();
      await calificar.getByText('2ª evaluación · 20 %').waitFor();
      // El galón, que gira al cerrar la lista: ya quieto.
      await page.waitForTimeout(250);
      await paso('item-largo');
      await calificar.getByRole('button', {name: /^Proyecto/}).click();
      await page.getByRole('button', {name: 'Sin ítem', exact: true}).last().click();
      await calificar.getByText('Sin ítem: el ítem se elige al subir').waitFor();
      await page.waitForTimeout(250);
      await paso('sin-item');
    },
  },
  {
    // H6: subir a un ítem, con dos que cambiarían, y sin ítem, eligiéndolo
    // al subir, con el fallo y el plan B.
    nombre: 'subir',
    items: true,
    async probar(page, paso) {
      await ir(page, 'Clases', 'clases');
      const main = page.locator('[data-pantalla="clases"]');
      await main.locator('.fila-clase').filter({hasText: 'Matemáticas 3.º A ESO'}).click();
      await main.getByRole('button', {name: /Examen tema 3/}).click();
      const calificar = page.locator('[data-pantalla="calificar"]');
      await calificar.locator('h1', {hasText: 'Examen tema 3'}).waitFor();
      await calificar.getByRole('button', {name: 'Subir a Clickedu'}).click();
      const dialogo = page.getByRole('dialog');
      await dialogo.getByText('Mirando lo que ya hay').waitFor();
      await paso('leyendo');
      await dialogo.getByText('y se cambiaría').waitFor();
      await paso('previa');
      await dialogo.getByRole('button', {name: 'Subir 9 y cambiarlas'}).click();
      await dialogo.getByText('Importando').waitFor();
      await paso('subiendo');
      await dialogo.getByText('Tras importar').waitFor();
      await paso('hecho');
      await dialogo.getByRole('button', {name: 'Cerrar'}).click();
      // Sin ítem: se elige al subir.
      await calificar.getByRole('button', {name: 'Examen tema 3'}).click();
      await page.getByRole('button', {name: 'Sin ítem', exact: true}).last().click();
      await calificar.getByText('Sin ítem: el ítem se elige al subir').waitFor();
      await calificar.getByRole('button', {name: 'Subir a Clickedu'}).click();
      await dialogo.getByText('¿A qué ítem').waitFor();
      await paso('elegir');
      await dialogo.getByRole('button', {name: 'Elige el ítem'}).click();
      await page.getByRole('button', {name: /^Proyecto/}).last().click();
      await page.waitForTimeout(250);
      await paso('elegido');
      await dialogo.getByRole('button', {name: 'Seguir'}).click();
      await dialogo.getByRole('button', {name: 'Subir 9 notas'}).click();
      await dialogo.getByText('Abrir la página de importar').waitFor();
      await paso('fallo');
    },
  },
  {
    nombre: 'sin-plantillas',
    carpeta: 'vacia',
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      await page.locator('[data-pantalla="calificar"]').getByText('Todavía no hay').waitFor();
      await paso('calificar');
      await ir(page, 'Clases', 'clases');
      await page.locator('[data-pantalla="clases"]').getByText('Todavía no hay').waitFor();
      await paso('clases');
    },
  },
  {
    nombre: 'carpeta-perdida',
    carpeta: 'no_existe',
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      await page.locator('[data-pantalla="calificar"]').getByText('ya no existe').first().waitFor();
      await paso('calificar');
      await ir(page, 'Ajustes', 'ajustes');
      await page.locator('[data-pantalla="ajustes"]').getByText('ya no existe').waitFor();
      await paso('ajustes');
    },
  },
  {
    nombre: 'ajustes',
    async probar(page, paso) {
      await ir(page, 'Ajustes', 'ajustes');
      const main = page.locator('[data-pantalla="ajustes"]');
      await main.getByText('Frases dictadas').waitFor();
      await paso('reposo');
      // D8: el informe preparado y el diálogo de borrar.
      await main.getByRole('button', {name: 'Preparar un informe'}).click();
      await main.getByText('Informe listo').waitFor();
      await paso('informe');
      await main.getByRole('button', {name: 'Borrar lo guardado'}).click();
      await page.waitForTimeout(250);
      await paso('borrar');
      await page.keyboard.press('Escape');
      // HUI: el micrófono, con la lista abierta y elegidos los auriculares.
      await main.getByRole('button', {name: 'El de Windows', exact: false}).click();
      await page.waitForTimeout(150);
      await paso('microfonos');
      await page.getByRole('button', {name: 'Jabra', exact: false}).last().click();
      await main.getByRole('button', {name: 'Jabra', exact: false}).waitFor();
      await paso('microfono-elegido');
    },
  },
  {
    // HUI: el micrófono elegido no está conectado. Se dice en Ajustes y al
    // dictar, que va por el de Windows.
    nombre: 'microfono-falta',
    auriculares: true,
    async probar(page, paso) {
      await ir(page, 'Calificar', 'calificar');
      const main = page.locator('[data-pantalla="calificar"]');
      await main.getByText('No está conectado', {exact: false}).waitFor();
      await paso('calificar');
      await ir(page, 'Ajustes', 'ajustes');
      await page.locator('[data-pantalla="ajustes"]').getByText('mientras, se escucha', {exact: false}).waitFor();
      await paso('ajustes');
    },
  },
  {
    // H7: una versión nueva al arrancar, y descargándose.
    nombre: 'actualizacion',
    actualizacion: true,
    async probar(page, paso) {
      const dialogo = page.getByRole('dialog', {name: 'Nueva versión'});
      await dialogo.waitFor();
      await page.waitForTimeout(250);
      await paso('aviso');
      await dialogo.getByRole('button', {name: 'Descargar'}).click();
      await dialogo.getByRole('progressbar').waitFor();
      await paso('descargando');
    },
  },
  {
    // H7: los datos de la v1.3.4 traídos, y quitarla.
    nombre: 'version-anterior',
    v1: 'traida',
    actualizacion: true,
    async probar(page, paso) {
      const dialogo = page.getByRole('dialog', {name: 'Tus datos de la versión anterior'});
      await dialogo.waitFor();
      await page.waitForTimeout(250);
      await paso('traida');
      await dialogo.getByRole('button', {name: 'Quitar la versión anterior'}).click();
      await dialogo.getByText('Hecho', {exact: false}).waitFor();
      await paso('quitada');
      // Cerrado, sale el de la versión nueva.
      await dialogo.getByRole('button', {name: 'Entendido'}).click();
      await page.getByRole('dialog', {name: 'Nueva versión'}).waitFor();
    },
  },
  {
    nombre: 'version-anterior-fallo',
    v1: 'fallo',
    async probar(page, paso) {
      await page.getByRole('dialog', {name: 'Tus datos de la versión anterior'}).waitFor();
      await page.waitForTimeout(250);
      await paso('fallo');
    },
  },
  {
    // H7: a mano, desde el pie de la barra, y al día.
    nombre: 'al-dia',
    async probar(page, paso) {
      await page.locator('aside').getByRole('button', {name: 'Buscar actualizaciones'}).click();
      await page.locator('aside').getByText('al día', {exact: false}).waitFor();
      await paso('barra');
    },
  },
  // Estadísticas, escondida hasta H8 (HUI): sin prueba.
  {
    // H4: la cuenta, sin guardar. Vacía, la contraseña con el ojo, un
    // usuario que no entra, sin red y entrar bien (en verde).
    nombre: 'clickedu',
    async probar(page, paso) {
      await ir(page, 'Conexión', 'clickedu');
      const main = page.locator('[data-pantalla="clickedu"]');
      const entrar = main.getByRole('button', {name: 'Entrar', exact: true});
      await entrar.waitFor();
      if (!(await entrar.isDisabled())) throw new Error('«Entrar» se puede pulsar sin nada escrito');
      // H5: sin cuenta no se puede traer.
      if (!(await main.getByRole('button', {name: 'Traer las plantillas'}).isDisabled()))
        throw new Error('«Traer las plantillas» se puede pulsar sin cuenta');
      await paso('vacia');
      await main.getByLabel('Usuario').fill('profesora.fernandez.villalobos');
      await main.getByLabel('Contraseña', {exact: true}).fill('mala');
      await main.getByRole('button', {name: 'Enseñar la contraseña'}).click();
      await main.getByRole('button', {name: 'Elegir archivo…'}).click();
      await main.getByText('archivo_seguridad_de_la_profesora', {exact: false}).waitFor();
      await paso('rellena');
      await entrar.click();
      await main.getByText('Entrando en Clickedu').waitFor();
      await paso('entrando');
      await main.getByText('no son correctos').waitFor();
      await paso('usuario-mal');
      await main.getByLabel('Usuario').fill('sinred');
      await main.getByLabel('Contraseña', {exact: true}).fill('buena');
      await entrar.click();
      await main.getByText('No se puede llegar').waitFor();
      await paso('sin-red');
      await main.getByLabel('Usuario').fill('profesora.fernandez.villalobos');
      await entrar.click();
      await main.getByText('Has entrado en Clickedu').waitFor();
      await paso('entrado');
    },
  },
  {
    // H4: con la cuenta guardada. Cambiar el archivo y que no lo acepte,
    // cambiar la contraseña y olvidar la cuenta.
    nombre: 'clickedu-guardada',
    clickedu: true,
    async probar(page, paso) {
      await ir(page, 'Conexión', 'clickedu');
      const main = page.locator('[data-pantalla="clickedu"]');
      await main.getByText('profesora.fernandez.villalobos').waitFor();
      await paso('guardada');
      await main.getByRole('button', {name: 'Cambiar archivo…'}).click();
      await main.getByText('puede haber caducado', {exact: false}).waitFor();
      await paso('archivo-mal');
      await main.getByRole('button', {name: 'Cambiar contraseña'}).click();
      await main.getByLabel('Contraseña nueva').waitFor();
      await paso('cambiar-contrasena');
      await main.getByRole('button', {name: 'Cancelar'}).click();
      await main.getByRole('button', {name: 'Olvidar la cuenta'}).click();
      await page.getByRole('dialog').waitFor();
      // Que acabe de entrar el diálogo.
      await page.waitForTimeout(250);
      await paso('olvidar');
      await page.getByRole('dialog').getByRole('button', {name: 'Olvidar', exact: true}).click();
      await main.getByRole('button', {name: 'Entrar', exact: true}).waitFor();
      await paso('olvidada');
    },
  },
  {
    // H5: traer las plantillas. El avance, la primera vez con dos dudas y
    // los avisos, una duda respondida «a ninguna» y lo hecho, con la que
    // está abierta en Excel.
    nombre: 'clickedu-traer',
    clickedu: true,
    async probar(page, paso) {
      await ir(page, 'Conexión', 'clickedu');
      const main = page.locator('[data-pantalla="clickedu"]');
      await main.getByRole('button', {name: 'Traer las plantillas'}).click();
      await main.getByText('Bajando las plantillas').waitFor();
      await paso('trayendo');
      await main.getByText('Es la primera vez', {exact: false}).waitFor();
      await paso('plan');
      await main.getByRole('button', {name: 'avisos al leer Clickedu'}).click();
      await paso('avisos');
      await main.getByText('te lo vuelvo a preguntar la próxima vez', {exact: false}).first().waitFor();
      await main.getByRole('button', {name: 'Dejarla como está y decidirlo otro día'}).first().click();
      await page.waitForTimeout(150);
      await paso('desplegable');
      await page.getByRole('button', {name: 'No pasarlas a ninguna: borrarlas'}).last().click();
      await main.getByText('se borran de la app', {exact: false}).waitFor();
      await paso('respondida');
      await main.getByRole('button', {name: 'Hacerlo'}).click();
      await main.getByText('puestas en tu carpeta', {exact: false}).waitFor();
      await paso('hecho');
    },
  },
  {
    nombre: 'muestrario',
    async probar(page, paso) {
      await ir(page, 'Muestrario', 'muestrario');
      const main = page.locator('[data-pantalla="muestrario"]');
      await paso('reposo');
      // El desplegable de clase abierto y el diálogo.
      await main.getByRole('button', {name: 'Matemáticas 3.º A ESO'}).click();
      await page.waitForTimeout(150);
      await paso('desplegable');
      await page.keyboard.press('Escape');
      await main.getByRole('button', {name: 'Abrir un diálogo'}).click();
      await page.waitForTimeout(250);
      await paso('dialogo');
      await page.keyboard.press('Escape');
    },
  },
];

// ─────────────────────────────────────────────────────────────────────────────

const vite = await createServer({
  configFile: path.join(RAIZ, 'vite.config.ts'),
  root: RAIZ,
  cacheDir: path.join(RAIZ, 'node_modules', '.vite-diseno'),
  logLevel: 'warn',
  clearScreen: false,
  server: {port: PUERTO, strictPort: true, hmr: false},
});
await vite.listen();

fs.rmSync(CAPTURAS, {recursive: true, force: true});
fs.mkdirSync(CAPTURAS, {recursive: true});

const fallos = [];
const navegador = await chromium.launch({channel: 'msedge'});
try {
  for (const prueba of PRUEBAS) {
    for (const [w, h] of TAMANOS) {
      const tam = `${w}x${h}`;
      const page = await navegador.newPage({viewport: {width: w, height: h}});
      await page.addInitScript(simular, {
        carpeta: prueba.carpeta ?? 'llena',
        modelo: prueba.modelo ?? true,
        auriculares: prueba.auriculares ?? false,
        actualizacion: prueba.actualizacion ?? false,
        v1: prueba.v1 ?? null,
        clickedu: prueba.clickedu ?? false,
        items: prueba.items ?? false,
      });
      page.on('pageerror', (e) => fallos.push(`${prueba.nombre} ${tam}: error en la página: ${e.message}`));
      await page.goto(`http://localhost:${PUERTO}/`);
      await page.evaluate(() => document.fonts.ready);
      let i = 0;
      const paso = async (n) => {
        const donde = `${prueba.nombre} ${tam} ${n}`;
        fallos.push(...(await medir(page, donde)));
        await page.screenshot({path: path.join(CAPTURAS, `${prueba.nombre}-${tam}-${i++}-${n}.png`)});
      };
      try {
        await prueba.probar(page, paso);
      } catch (e) {
        fallos.push(`${prueba.nombre} ${tam}: la prueba no pudo seguir: ${e.message.split('\n')[0]}`);
        await page.screenshot({path: path.join(CAPTURAS, `${prueba.nombre}-${tam}-fallo.png`)});
      }
      await page.close();
    }
  }
} finally {
  await navegador.close();
  await vite.close();
}

const n = fs.readdirSync(CAPTURAS).length;
console.log(`\n${n} capturas en ${path.relative(RAIZ, CAPTURAS)}`);
if (fallos.length) {
  console.log(`\n${fallos.length} fallos:\n  ${fallos.join('\n  ')}`);
  process.exitCode = 1;
} else {
  console.log('Sin fallos.');
}
