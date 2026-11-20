#!/usr/bin/env node
/**
 * release.mjs — sube la versión en los CUATRO ficheros, commit y tag.
 *
 *   npm run release 2.0.1
 *   npm run release 2.0.1 -- --dry-run     (enseña lo que haría, sin tocar git;
 *                                           vale también desde PowerShell)
 *
 * Es el de library-app con dos cambios:
 *
 *   1. Los cuatro ficheros son `package.json` (de ahí la lee la interfaz,
 *      `src/modules/app/version.ts`), `Cargo.toml` (la que manda: la lee el
 *      actualizador), `tauri.conf.json` y `Cargo.lock`.
 *   2. **No hace push**: el push lo hace el usuario (CLAUDE.md). Al acabar
 *      dice qué escribir.
 *
 * Como allí, valida antes que nada `docs/releases/vX.Y.Z.md` y **relee los
 * cuatro al final y falla si no coinciden**.
 *
 * El push del tag dispara `.github/workflows/release.yml` (H7), que compila
 * en Windows, firma y deja un release **borrador**: hasta que alguien pulsa
 * «Publish release» en GitHub, ninguna app se entera — ni esta ni la v1.3.4.
 */

import {execSync} from 'node:child_process';
import {readFileSync, writeFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

import {leerNotas} from './notas-de-version.mjs';

const raiz = resolve(dirname(fileURLToPath(import.meta.url)), '..');

const args = process.argv.slice(2);
const version = args.find((a) => /^\d+\.\d+\.\d+$/.test(a));
// En PowerShell 5.1 el `--` lo quita el propio PowerShell: `--dry-run` se lo
// queda npm y aquí solo llega `npm_config_dry_run` (library-app, v0.3.0).
const seco = args.includes('--dry-run') || process.env.npm_config_dry_run === 'true';

if (!version) {
  console.error('Uso: npm run release <X.Y.Z> [-- --dry-run]');
  process.exit(1);
}

const leer = (rel) => readFileSync(resolve(raiz, rel), 'utf8');
const escribir = (rel, txt) => {
  if (seco) return console.log(`  [seco] escribiría ${rel}`);
  writeFileSync(resolve(raiz, rel), txt);
};
const correr = (cmd) => {
  if (seco) return console.log(`  [seco] ${cmd}`);
  execSync(cmd, {cwd: raiz, stdio: 'inherit'});
};

// ── 0. Las notas de versión, antes que nada ──────────────────────────────────
let notas;
try {
  notas = leerNotas(version);
} catch (e) {
  console.error(`ERROR: ${e.message}`);
  process.exit(1);
}
console.log(`\n  ✓ ${notas.ruta} — ${notas.entradas.length} entrada(s)`);

// ── 1. Los cuatro ficheros ───────────────────────────────────────────────────
const V = String.raw`\d+\.\d+\.\d+`;
const FICHEROS = [
  {
    ruta: 'package.json',
    sube: (s) => s.replace(new RegExp(String.raw`"version"\s*:\s*"${V}"`), `"version": "${version}"`),
    lee: (s) => s.match(new RegExp(String.raw`"version"\s*:\s*"(${V})"`))?.[1],
  },
  {
    ruta: 'src-tauri/Cargo.toml',
    // El primer `version = "x"` del fichero es el del paquete.
    sube: (s) => s.replace(new RegExp(String.raw`^version\s*=\s*"${V}"`, 'm'), `version = "${version}"`),
    lee: (s) => s.match(new RegExp(String.raw`^version\s*=\s*"(${V})"`, 'm'))?.[1],
  },
  {
    ruta: 'src-tauri/tauri.conf.json',
    sube: (s) => s.replace(new RegExp(String.raw`"version"\s*:\s*"${V}"`), `"version": "${version}"`),
    lee: (s) => s.match(new RegExp(String.raw`"version"\s*:\s*"(${V})"`))?.[1],
  },
  {
    ruta: 'src-tauri/Cargo.lock',
    // La entrada de la propia app, no la de una dependencia.
    sube: (s) =>
      s.replace(
        new RegExp(String.raw`(name = "gestor-de-notas"\r?\nversion = )"${V}"`),
        `$1"${version}"`,
      ),
    lee: (s) => s.match(new RegExp(String.raw`name = "gestor-de-notas"\r?\nversion = "(${V})"`))?.[1],
  },
];

console.log(`\nVersión ${version}${seco ? ' (en seco)' : ''}\n`);

const resultado = FICHEROS.map(({ruta, sube, lee}) => {
  const antes = leer(ruta);
  const despues = sube(antes);
  if (antes === despues) {
    // Ya estaba en esa versión, o la expresión no casa: no es lo mismo.
    console.log(
      lee(antes) === version
        ? `  · ${ruta} — ya estaba en ${version}`
        : `  ✗ ${ruta} — no encuentro la versión dentro`,
    );
  } else {
    console.log(`  ↑ ${ruta}`);
    escribir(ruta, despues);
  }
  return {ruta, quedaEn: lee(despues)};
});

// ── 2. Los cuatro, iguales ───────────────────────────────────────────────────
const malas = resultado.filter(({quedaEn}) => quedaEn !== version);
if (malas.length > 0) {
  console.error('\nERROR: los cuatro ficheros no quedan iguales.');
  for (const {ruta, quedaEn} of resultado) {
    console.error(`  ${quedaEn === version ? '✓' : '✗'} ${ruta} → ${quedaEn ?? '(no la encuentro)'}`);
  }
  console.error('\nNo se ha tocado git. Arregla el fichero y repite.');
  process.exit(1);
}
console.log(`\n  ✓ los cuatro ficheros en ${version}`);

// La clave pública del actualizador tiene que estar puesta: con la de mentira,
// la versión publicada no podría actualizarse nunca.
if (leer('src-tauri/tauri.conf.json').includes('PON_AQUI_LA_CLAVE_PUBLICA')) {
  console.error('\nERROR: falta la clave pública del actualizador en src-tauri/tauri.conf.json.');
  process.exit(1);
}

// ── 3. Git, sin push ─────────────────────────────────────────────────────────
try {
  execSync(`git rev-parse v${version}`, {cwd: raiz, stdio: 'ignore'});
  console.error(`\nERROR: el tag v${version} ya existe. Si es un intento fallido, bórralo a mano.`);
  process.exit(1);
} catch {
  /* no existía, que es lo normal */
}

correr(`git add ${FICHEROS.map((f) => f.ruta).join(' ')} ${notas.ruta}`);
correr(`git commit -m "chore: release v${version}"`);
correr(`git tag v${version}`);

const rama = execSync('git rev-parse --abbrev-ref HEAD', {cwd: raiz}).toString().trim();
if (seco) {
  console.log(`\n(en seco: no se ha escrito nada ni se ha tocado git)\n`);
  process.exit(0);
}
console.log(`\n✓ v${version} lista. Para publicarla:\n`);
console.log(`    git push origin ${rama} v${version}\n`);
console.log('  CI compila y firma el instalador y deja un release BORRADOR.');
console.log('  Revísalo en GitHub y pulsa «Publish release»: hasta entonces nadie lo ve.\n');
