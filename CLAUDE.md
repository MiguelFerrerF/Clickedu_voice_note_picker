# Claude — instrucciones para el Gestor de Notas

> **Este fichero se queda en 200 líneas como mucho**: se carga entero en cada
> sesión. Al cerrar algo, cambia su línea de la tabla y el «Dónde se quedó»;
> el detalle va a `docs/`.

## Estado del proyecto

Se está rehaciendo la app (**v2**, Tauri) sobre la base de
`C:\dev\library-app`. La app vieja (**v1**, Python + customtkinter) **ya no
está en esta rama**: vive en `main` y en el tag `v1.3.4`, que es la última
publicada, hasta que la v2 la sustituya en H7. Plan, decisiones e hitos en
[`docs/01-plan-de-trabajo.md`](docs/01-plan-de-trabajo.md).

> **Dónde se quedó (2026-11-20): toca H4** (conectar con Clickedu, `docs/01`).
> En la rama `v2`, la única de la v2 (un commit por hito y otro al cerrarlo).
> **H7, casi cerrado**: la `v2.0.0` está **publicada** (actualizador de
> Tauri con firma, `release.yml`, el paso desde la v1.3.4, la app
> asegurada); en este ordenador ya está instalada y la v1, quitada. **En
> espera** (lista «Lo que falta, uno a uno» de H7): el paso 9, la `v2.0.1`
> (ya en `v2`, notas revisadas) para probar el actualizador de verdad,
> **cuando la única persona que tenía la v1 tenga la `v2.0.0`**; y lo de
> D: su procesador con AVX2 e importar un exportado en Clickedu (lo
> pendiente de H2, como muy tarde en H6). **H4, la primera parte hecha y
> probada con Clickedu de verdad** (2026-11-20: la cuenta, entrar y
> comprobar, `clickedu/`; la pantalla Conexión, solo en desarrollo). Sigue
> la segunda parte (`docs/01`): páginas reales anonimizadas para los
> tests, volver a entrar solo y la capa resistente. La cuenta del usuario
> ya está guardada en la app. Clickedu no distingue «caducado» de
> «incorrecto». La contraseña filtrada la cambia el administrador del
> colegio (con el archivo nuevo, ella sola ya no basta: hay que cambiarla).
> **En espera** (`docs/01`, HUI): con lo que traigan H4–H8, volver a mirar
> la distribución de cada pantalla, que ya ocupa la ventana entera.
> Estadísticas y Conexión están escondidas en `Sidebar` (`HAY_ESTADISTICAS`,
> `HAY_CLICKEDU`): se encienden al hacerlas. El audio **no se guarda** (D8);
> lo dictado se mide en local con `cargo run --example medir_registro` (solo
> cuenta, sin nombres). El campo «Probar el dictado» se queda, solo en
> desarrollo. Para ver plantillas reales sin sacar datos de alumnos:
> `cargo run --example revisar_plantillas -- <carpeta>`; para probar el
> dictado sin alumnos reales: `cargo run --example escribir_clase_inventada`
> (a la carpeta por defecto). Copia de los datos de antes de la v2:
> `%USERPROFILE%\Copia Gestor de Notas antes de la v2 (2026-11-20)`.

| Hito | Qué dejó | Probado |
|---|---|---|
| **HUI** | La interfaz, antes de H4: la cuadrícula compacta y de 1 a 4 columnas; el dictado (lo apuntado grande y los dos de antes, dudas con 1–4/Esc, medidor de volumen, micrófono en Ajustes); el avance, «Siguiente sin nota» y las teclas tras un ⓘ; el título que es el desplegable; `BarraAvance`; las pantallas a todo el ancho | sí, a mano (A–D y a todo el ancho) y con la prueba de diseño |
| **H3b** | `voz::interprete` (la nota; lo no visto en grabaciones, `NOTA_RARA`, no se interpreta), `voz::emparejar` (fonética, los dos órdenes de «Apellidos, Nombre»), `voz::entender` (cuándo preguntar); `entender_frase`, el panel del dictado y las propuestas en la cuadrícula; `registro.rs` e informe sin nombres (D8), en Ajustes | sí, a mano (clase inventada), con tests de Rust, la prueba de diseño y `probar_entender` sobre las 100 grabaciones: **77 apuntadas, 20 preguntadas, 2 sin entender, 0 mal en silencio** |
| **H3a** | D2 cerrada con números: Whisper `small` `q5_1` recortado (90–94 % en 1,1 s); micrófono, remuestreo y motor; el banco de pruebas de voz; **el push-to-talk** (`useDictado`, `commands::voz`), la descarga comprobada del modelo (`voz::modelo`) y medir en local (`registro::medir`, sin audio); los bucles de Whisper se enseñan una vez y «[Música]» es silencio | el motor, con voz de verdad (100 grabaciones); el push-to-talk, a mano (clase inventada y descarga), con tests de Rust y la prueba de diseño |
| **H2** | `sesion.rs` (notas por clave de alumno, al momento), `exportar.rs` (el `.xlsx` de la v1, a `Documentos\Gestor de Notas\notas`); Calificar con campos de nota, Exportar y contador | sí, a mano, con tests de Rust (ida y vuelta) y la prueba de diseño; **falta importarlo en Clickedu** |
| **H1** | `plantillas.rs` (lee por cabecera, con avisos), `ajustes.json`, vigilante de la carpeta; Calificar en lectura, Clases, Ajustes | sí, a mano (plantillas reales), con tests de Rust y la prueba de diseño |
| **H0** | Andamiaje Tauri 2 + Vite + React + TS, paleta turquesa en claro con neutros «niebla», primitivas de `components/ui/`, `Sidebar` y cinco pantallas vacías, Muestrario, i18n, IBM Plex con `latin-ext`, prueba de diseño | sí, a mano y con la prueba de diseño |

## Cómo se mira lo que hay

```powershell
npm run tauri:dev        # la app
npm run typecheck        # los tipos
npm run probar:diseno    # pantallas a 960×640, 1100, 1280 y 1920, sin Tauri
cd src-tauri; cargo test # los tests de Rust
npm run release X.Y.Z    # versión, commit y tag (docs/releases/README.md)
```

- **La prueba de diseño se pasa tras todo cambio visible y las capturas se
  miran** (`scripts/probar-diseno/capturas/`). Una pantalla nueva se añade a
  `PRUEBAS` en `scripts/probar-diseno/probar.mjs`. Arranca su Vite en el
  **1430**: no lances otro en el 1420 con `tauri dev` abierto. El scroll
  horizontal a propósito se marca con `data-scroll-horizontal`. Además de lo
  de library-app, falla con **texto cortado sin «…»**.
- **En Windows** (Git Bash + PowerShell), `cmd.exe /c` se cuelga: usa
  PowerShell. PowerShell 5.1 escribe UTF-8 **con BOM** con `Set-Content
  -Encoding utf8`, y eso rompe un JSON para Python y otros: escribe con
  `[IO.File]::WriteAllText(..., (New-Object Text.UTF8Encoding $false))`.
- **El código de la v1 es referencia para portar** (el intérprete de frases y
  el emparejamiento fonético en H3, el cliente de Clickedu en H4–H6). Se lee
  sin cambiar de rama: `git show main:GradingApp/core/voice_processor.py`,
  `git show main:GradingApp/core/clickedu_client.py`. **Se porta, no se
  copia**: con tests, y arreglando lo que estaba mal (la mediana, D6).

## Antes de proponer o escribir nada, lee

1. [`docs/01-plan-de-trabajo.md`](docs/01-plan-de-trabajo.md) — **las
   decisiones D1–D8 y los hitos**
2. [`docs/02-sistema-visual.md`](docs/02-sistema-visual.md) — la paleta y la
   regla de color

## Decisiones que no hay que reabrir

Cerradas el 2026-11-17 (D8, el 2026-11-19). No se reabren sin que el usuario lo pida.

- **D1 · Stack**: Tauri 2 + React + TS, lógica en Rust, como library-app.
- **D2 · Voz**: Whisper local, `small` `q5_1` con la ventana recortada
  (cerrada con la prueba de H3a, 2026-11-18); se descarga al usar la voz.
  **La voz con nombres de menores no sale del ordenador.**
- **D3 · Credenciales de Clickedu**: en el Administrador de credenciales de
  Windows (`keyring`), **nunca en un fichero**. Ya se filtraron una vez en
  `config.json` (purgado del historial el 2026-11-17).
- **D4 · Datos**: plantillas y notas en `Documentos\Gestor de Notas\`; lo de la
  app en `%APPDATA%\es.gestornotas.app\`. **La plantilla de Clickedu no se
  escribe nunca.**
- **D5 · Paleta**: turquesa del logo (`#0F9096`), oscurecido a `#007F82` para
  los botones por contraste. Solo tema claro.
- **D6 · Color**: el suspenso **no** es rojo; verde = evento que se apaga;
  discontinuo = provisional; rojo = error de verdad.
- **D7 · Publicar**: `v2.0.0` en este repo, **que sigue público**. El
  instalador se sube como **`Instalador_GestorNotas.exe`**: con ese nombre lo
  encuentra el actualizador de la v1.3.4 y lo instala solo. Desde ahí,
  `tauri-plugin-updater` **con firma**; la clave privada, solo en los
  secrets de Actions. La v2 desinstala la v1 tras copiar sus datos.
- **D8 · Casos reales**: registro en local (`registro.rs`, seis meses) con
  la respuesta buena del profesor, y un **informe sin nombres ni voz** que
  se manda a mano como fichero. Sale la forma del error, nunca su
  contenido. **El audio no se guarda** (2026-11-19); se mide en local con
  `medir_registro`.
- **Clickedu no tiene API para esto**: la oficial no expone notas y no nos dan
  credenciales. Se lee la web, con una capa resistente (H4).

## Reglas del proyecto

- **La referencia es library-app**: copia su estilo, formato, simpleza y
  método (commits, i18n, docs). Antes de inventar un token, un componente o
  una carpeta, mira cómo lo resuelve ella. **Seis tokens propios y ni uno
  más**, los suyos. Los componentes se traen **uno a uno**, no el directorio.
- **El color marca eventos y excepciones, nunca estados normales**
  (`docs/02` §3). El **borde discontinuo** es solo para «provisional».
- **La app no se congela nunca y no pierde lo que estabas haciendo**: ningún
  comando de Tauri síncrono hace trabajo (todos `async`); lo largo lleva
  `Progreso` con avance real; **las pantallas visitadas no se desmontan**
  (`display: none`), así que no guardes en el `useState` de una pantalla nada
  que dependa de que se desmonte.
- **Las pantallas ocupan la ventana entera**, sin `maxWidth` (en esto no
  se sigue a library-app): lo que crece es lo que lo aprovecha (más
  columnas, tablas más anchas). Tope solo para párrafos de texto y controles.
- **Ningún texto visible en un componente**: todo por `t()` y
  `src/modules/i18n/es.ts`.
- **Nunca datos de alumnos reales en el repo**: las plantillas de prueba son
  inventadas. `.gitignore` excluye todo `.xlsx`/`.xls` salvo
  `src-tauri/tests/datos/`.
- **Nunca credenciales en el repo**: ni de Clickedu, ni tokens de GitHub.
- **Ninguna normalización en silencio**: lo que la app corrija de una
  plantilla o de lo que lee de Clickedu, se dice.

## Notas de entorno

- Node 22, npm 11, cargo/rustc 1.95, en Windows. `tauri-build` **exige
  `src-tauri/icons/icon.ico`**; los iconos salen de `src-tauri/icons/origen.png`
  con `npx tauri icon src-tauri\icons\origen.png -o src-tauri\icons` (y luego
  se borran `android/` e `ios/`).
- `node_modules` es de Windows: se instala siempre desde Windows.
- Repo **público**: <https://github.com/MiguelFerrerF/Clickedu_voice_note_picker>
  (`origin`, rama `main`; la v2, en `v2`). **El usuario hace el push.**
- Los commits terminan con la línea `Co-Authored-By` que indique la sesión.
- **Formato de Rust**: no hay `rustfmt.toml` y el repo no está entero
  formateado: **nunca `cargo fmt`** (reescribe todo). Lo más parecido es
  `rustfmt --config max_width=120,use_small_heuristics=Max` sobre los
  ficheros propios, y ojo, que sigue los `mod` (desde `lib.rs` formatea
  todo). En TS, `npx prettier --print-width 120 --single-quote
  --no-bracket-spacing --bracket-same-line`.
- Con `tauri:dev` abierto, `cargo test` no puede reescribir el `.exe`: pasa
  `cargo test --lib`. En los heredocs de Bash con Python, las barras
  invertidas se interpretan (una `\n` escrita acaba en salto de línea):
  para textos con barras invertidas, usa Edit o Write.
