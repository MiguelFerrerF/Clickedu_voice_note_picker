/**
 * La versión de la app, sacada de `package.json`.
 *
 * Es uno de los cuatro sitios donde vive el número (con `Cargo.toml`,
 * `tauri.conf.json` y `Cargo.lock`); en H7 los sincroniza `npm run release`,
 * como en library-app. Mientras, se cambian a mano y a la vez.
 */

import paquete from '../../../package.json';

export const VERSION_ACTUAL: string = paquete.version;
