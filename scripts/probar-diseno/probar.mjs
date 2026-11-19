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
 * modelo de voz ya está descargado.
 */
function simular({carpeta, modelo}) {
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
  ];
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

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {unregisterListener() {}};
  window.__TAURI_INTERNALS__ = {
    transformCallback: () => n++,
    invoke: async (cmd, args) => {
      switch (cmd) {
        case 'plugin:event|listen':
          return n++;
        case 'listar_clases':
          return lista();
        case 'leer_clase':
          // En la de los avisos, una nota ya puesta en la sesión y otra quitada.
          return args.fichero === 'matematicas_3_a_eso.xlsx'
            ? {plantilla: PLANTILLA, puestas: {20233: 6.5, 20232: null}}
            : {plantilla: OTRA, puestas: {}};
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
        case 'abrir_microfono':
          return 'Micrófono (Realtek(R) Audio)';
        case 'terminar_dictado':
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

const vacia = (nombre, rotulo) => ({
  nombre,
  async probar(page, paso) {
    await ir(page, rotulo, nombre);
    await paso('reposo');
  },
});

const PRUEBAS = [
  {
    nombre: 'calificar',
    async probar(page, paso) {
      // Sin clase elegida enseña la primera; desde el desplegable, otra.
      const main = page.locator('[data-pantalla="calificar"]');
      await main.getByRole('row').first().waitFor();
      await paso('primera');
      await main.getByRole('button', {name: 'Biología 1.º AB Bachillerato'}).click();
      await page.waitForTimeout(150);
      await paso('desplegable');
      await page.getByRole('button', {name: 'Matemáticas 3.º A ESO', exact: true}).last().click();
      await main.getByText('Al leer la plantilla').waitFor();
      await paso('avisos');
    },
  },
  {
    // H2: escribir notas, una que no es nota, y exportar bien y mal.
    nombre: 'notas',
    async probar(page, paso) {
      const main = page.locator('[data-pantalla="calificar"]');
      const campo = (nombre) => main.getByRole('textbox', {name: `Nota de ${nombre}`});
      await campo('Castro Molina, Lucía').fill('7,5');
      await page.keyboard.press('Enter');
      await page.keyboard.type('4');
      await page.keyboard.press('Enter');
      await page.keyboard.type('12');
      await page.keyboard.press('Enter');
      await main.getByText('Ha de ir de 0 a 10').waitFor();
      await paso('escritas');
      await page.keyboard.press('Escape');
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
      const main = page.locator('[data-pantalla="calificar"]');
      await main.getByText('Se escucha por', {exact: false}).waitFor();
      await paso('reposo');
      await page.keyboard.down('Space');
      await main.getByText('Escuchando').waitFor();
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
    nombre: 'sin-plantillas',
    carpeta: 'vacia',
    async probar(page, paso) {
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
    },
  },
  vacia('estadisticas', 'Estadísticas'),
  vacia('clickedu', 'Conexión'),
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
      await page.addInitScript(simular, {carpeta: prueba.carpeta ?? 'llena', modelo: prueba.modelo ?? true});
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
