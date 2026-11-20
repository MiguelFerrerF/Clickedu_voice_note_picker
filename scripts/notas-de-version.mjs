#!/usr/bin/env node
/**
 * notas-de-version.mjs — lee y valida `docs/releases/vX.Y.Z.md`.
 *
 * Lo usan dos sitios, y por eso vive aparte:
 *
 *   - `scripts/release.mjs`, que se para **antes de tocar git** si las notas
 *     de la versión no están o no traen titular y entradas.
 *   - El workflow de CI (`.github/workflows/release.yml`, H7), que compone con
 *     ellas el cuerpo del release:
 *
 *       node scripts/notas-de-version.mjs 0.2.0                  (comprueba)
 *       node scripts/notas-de-version.mjs 0.2.0 --cuerpo         (lo imprime)
 *       node scripts/notas-de-version.mjs v0.2.0 --github-output (a $GITHUB_OUTPUT)
 *
 * Es el de library-app tal cual. El cuerpo es el Markdown sin su frontmatter
 * y, detrás, el bloque que lee la app sin una segunda petición
 * (`src-tauri/src/actualizacion.rs`; `tauri-action` lo copia en las `notes`
 * de `latest.json`):
 *
 *   <!--RELEASE_DATA
 *   {"highlights":"…","entries":[{"type":"feat","text":"…"}]}
 *   -->
 */

import {appendFileSync, existsSync, readFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const raiz = resolve(dirname(fileURLToPath(import.meta.url)), '..');

export const TIPOS = ['feat', 'fix', 'perf', 'chore'];

/**
 * Las notas de `version` (sin la `v`), o un error que dice qué falta.
 * Devuelve `{ruta, titular, entradas, markdown}`.
 */
export function leerNotas(version) {
  const ruta = `docs/releases/v${version}.md`;
  if (!existsSync(resolve(raiz, ruta))) {
    throw new Error(
      `falta ${ruta}.\n  Cópialo de docs/releases/PLANTILLA.md y cuenta qué trae la versión.`,
    );
  }
  const texto = readFileSync(resolve(raiz, ruta), 'utf8');
  const fm = texto.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?/);
  if (!fm) throw new Error(`${ruta} no tiene frontmatter entre --- y ---.`);

  const titular = fm[1].match(/^highlights:\s*(.+)$/m)?.[1]?.trim();
  const entradas = [...fm[1].matchAll(/^\s*-\s+(\w+):\s*(.+)$/gm)]
    .map(([, tipo, texto]) => ({type: tipo, text: texto.trim()}))
    .filter((e) => TIPOS.includes(e.type));

  if (!titular) throw new Error(`${ruta} no tiene "highlights:" en el frontmatter.`);
  if (entradas.length === 0) {
    throw new Error(
      `${ruta} no tiene ninguna entrada válida bajo "entries:".\n` +
        `  Los tipos admitidos son: ${TIPOS.join(', ')}.`,
    );
  }
  return {ruta, titular, entradas, markdown: texto.slice(fm[0].length).trim()};
}

/** El cuerpo del release: Markdown legible + el bloque para la app. */
export function cuerpoDelRelease({titular, entradas, markdown}) {
  const datos = JSON.stringify({highlights: titular, entries: entradas});
  return `${markdown}\n\n<!--RELEASE_DATA\n${datos}\n-->\n`;
}

// ── Por consola ─────────────────────────────────────────────────────────────
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const version = args.find((a) => /^v?\d+\.\d+\.\d+$/.test(a))?.replace(/^v/, '');
  if (!version) {
    console.error('Uso: node scripts/notas-de-version.mjs <X.Y.Z> [--cuerpo | --github-output]');
    process.exit(1);
  }
  let notas;
  try {
    notas = leerNotas(version);
  } catch (e) {
    console.error(`ERROR: ${e.message}`);
    process.exit(1);
  }
  const cuerpo = cuerpoDelRelease(notas);
  if (args.includes('--cuerpo')) {
    process.stdout.write(cuerpo);
  } else if (args.includes('--github-output')) {
    const salida = process.env.GITHUB_OUTPUT;
    if (!salida) {
      console.error('ERROR: --github-output solo vale dentro de GitHub Actions.');
      process.exit(1);
    }
    // Un delimitador que no puede salir en las notas.
    const fin = `FIN_${Date.now()}_${Math.random().toString(36).slice(2)}`;
    appendFileSync(salida, `cuerpo<<${fin}\n${cuerpo}\n${fin}\n`);
    console.log(`✓ ${notas.ruta} — ${notas.entradas.length} entrada(s), cuerpo en $GITHUB_OUTPUT`);
  } else {
    console.log(`✓ ${notas.ruta} — ${notas.entradas.length} entrada(s)`);
  }
}
