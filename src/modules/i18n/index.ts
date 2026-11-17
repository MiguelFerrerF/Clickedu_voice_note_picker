/**
 * i18n — la fontanería de traducción en tiempo de ejecución.
 *
 * COPIADO de library-app (y esta, de la Quality Test App), con la misma
 * forma a propósito, para que portar una pantalla sea un renombre mecánico:
 *
 *   - `t(clave, vars?)` — traduce, interpolando `{nombre}`.
 *   - `setIdioma(l)` — cambia el idioma, lo persiste y avisa por el bus.
 *   - `useIdioma()` — hook de React que re-renderiza al cambiar.
 *
 * **Solo hay castellano**, y aun así se monta desde el primer día: cuesta
 * poco ahora y mucho después.
 */

import es, {type TranslationKey} from './es';

export type Idioma = 'es';

const CLAVE_ALMACEN = 'gestornotas-idioma';
const IDIOMA_POR_DEFECTO: Idioma = 'es';

const DICCIONARIOS = {es} as const;

/** Bus de eventos para quien no es un componente de React. */
const BUS_I18N: EventTarget = new EventTarget();

let _idioma: Idioma = ((): Idioma => {
  try {
    const v = localStorage.getItem(CLAVE_ALMACEN);
    return v === 'es' ? v : IDIOMA_POR_DEFECTO;
  } catch {
    return IDIOMA_POR_DEFECTO;
  }
})();

export const getIdioma = (): Idioma => _idioma;

export const setIdioma = (l: Idioma): void => {
  if (!DICCIONARIOS[l]) return;
  _idioma = l;
  try {
    localStorage.setItem(CLAVE_ALMACEN, l);
  } catch {
    /* modo privado o almacenamiento lleno: no es motivo para romper nada */
  }
  BUS_I18N.dispatchEvent(new CustomEvent('cambio-idioma', {detail: {idioma: l}}));
};

/**
 * Traduce una clave.
 *
 * Si falta, **devuelve la clave tal cual** en vez de una cadena vacía: así un
 * texto sin traducir se ve en pantalla y se arregla, en lugar de desaparecer.
 */
export const t = (
  clave: TranslationKey | string,
  vars?: Record<string, string | number>,
): string => {
  const dic = DICCIONARIOS[_idioma] as Record<string, string>;
  let s = dic[clave] ?? clave;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      s = s.replace(`{${k}}`, String(v));
    }
  }
  return s;
};

export {BUS_I18N, DICCIONARIOS};
export type {TranslationKey};
