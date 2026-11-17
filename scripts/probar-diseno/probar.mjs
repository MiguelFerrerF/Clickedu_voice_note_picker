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
 * El backend falso: lo justo para que las pantallas pinten. `invoke` de
 * `@tauri-apps/api` pasa por `window.__TAURI_INTERNALS__`. En H0 no hay
 * comandos; cada hito añade aquí los suyos con datos inventados.
 */
const SIMULACION = `
  (() => {
    let n = 1;
    window.__TAURI_INTERNALS__ = {
      transformCallback: () => n++,
      invoke: async (cmd) => {
        switch (cmd) {
          case 'plugin:event|listen': return n++;
          default: return null;
        }
      },
    };
  })();
`;

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
  vacia('calificar', 'Calificar'),
  vacia('clases', 'Clases'),
  vacia('estadisticas', 'Estadísticas'),
  vacia('clickedu', 'Conexión'),
  vacia('ajustes', 'Ajustes'),
  {
    nombre: 'muestrario',
    async probar(page, paso) {
      await ir(page, 'Muestrario', 'muestrario');
      const main = page.locator('[data-pantalla="muestrario"]');
      for (const [paleta, rotulo] of [
        ['agua', 'Agua — neutros del propio turquesa'],
        ['papel', 'Papel — neutros cálidos'],
        ['niebla', 'Niebla — neutros fríos'],
      ]) {
        // El desplegable de la paleta; sus opciones son botones en un portal
        // al final de <body>, así que la última coincidencia es la opción.
        await main.getByRole('button', {name: /neutros/}).first().click();
        await page.getByRole('button', {name: rotulo, exact: true}).last().click();
        await page.waitForTimeout(150);
        await paso(paleta);
      }
      // Vuelta a «agua», el desplegable de clase abierto y el diálogo.
      await main.getByRole('button', {name: /neutros/}).first().click();
      await page.getByRole('button', {name: 'Agua — neutros del propio turquesa', exact: true}).last().click();
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
      await page.addInitScript(SIMULACION);
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
