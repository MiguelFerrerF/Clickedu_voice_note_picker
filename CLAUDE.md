# Claude — instrucciones para el Gestor de Notas

> **Este fichero se queda en 200 líneas como mucho**: se carga entero en cada
> sesión. Al cerrar algo, cambia su línea de la tabla y el «Dónde se quedó»;
> el detalle va a `docs/`.

## Estado del proyecto

La app es la **v2** (Tauri), hecha sobre la base de `C:\dev\software\library-app`.
La vieja (**v1**, Python + customtkinter), sustituida en H7, solo queda en
el tag `v1.3.4`. Plan, decisiones e hitos en
[`docs/01-plan-de-trabajo.md`](docs/01-plan-de-trabajo.md).

> **Dónde se quedó (2026-11-25): H9, cerrado** y en `main`; con él,
> **la `v2.3.0`, publicada** (la ayuda). Siguiente,
> por decidir: H10 o volver a mirar las pantallas (HUI). Cada
> cambio, en una rama propia que sale de `main` (un commit por hito y
> otro al cerrarlo); las versiones, de `main`. Ramas, versiones y
> cómo está la carpeta de este ordenador para probar traer, en `docs/01`
> («Cómo se trabaja y cómo está este ordenador»). **Importar reemplaza el
> ítem entero**: la subida completa el Excel con lo de Clickedu
> (`subir::completar`). El mapa de Clickedu, en
> [`docs/03-clickedu.md`](docs/03-clickedu.md). Las páginas de verdad
> anonimizadas (`tests/datos/clickedu/`, `anonimizar_clickedu`) **se miran
> antes de cada commit: el repo es público**; lo crudo, en
> `%APPDATA%\es.gestornotas.app\clickedu\capturas\`, **nunca en el repo**.
> El aviso automático por issue (D8 reabierta) **espera** a hacer el repo
> privado, que rompe los actualizadores. **Lo básico es traer y subir**;
> mirar Clickedu (ítems, rúbricas…) va aparte, en H10, solo leyendo y tras
> H6; qué enseña, se decide más adelante (`docs/01`, la frontera,
> 2026-11-21). Dentro de Clickedu, **nunca `/` ni
> `user.php`: cierran la sesión**. **En espera** (`docs/01`, HUI): con lo que traigan H4–H8, volver a mirar
> la distribución de cada pantalla, que ya ocupa la ventana entera.
> El audio **no se guarda** (D8);
> lo dictado se mide en local con `cargo run --example medir_registro` (solo
> cuenta, sin nombres). El campo «Probar el dictado» se queda, solo en
> desarrollo. Para ver plantillas reales sin sacar datos de alumnos:
> `cargo run --example revisar_plantillas -- <carpeta>`; para probar el
> dictado sin alumnos reales: `cargo run --example escribir_clase_inventada`
> (a la carpeta por defecto). Copia de los datos de antes de la v2:
> `%USERPROFILE%\Copia Gestor de Notas antes de la v2 (2026-11-20)`.

| Hito | Qué dejó | Probado |
|---|---|---|
| **H9** | La ayuda, la ligera (sin focos): pantalla `Ayuda` en la barra (recorrido, cada pantalla, dictado, teclas con `ListaTeclas`, lo privado, si algo no va); `DialogoBienvenida` la primera vez (sin plantillas, a Conexión; con ellas, a Clases); `AvisoPrimeraVez` en cada pantalla, desde `Cabecera` con `PantallaContext`; lo visto, en `ajustes.json` (`ayudaVista`, `ayuda_vista`, `marcar_ayuda`) | sí: el usuario lo ha visto (2026-11-25); tests de Rust (170) y la prueba de diseño |
| **H8** | Estadísticas, con la clase y el ítem de Calificar (`TituloClase`): `leer_estadisticas`; `estadisticas.rs` (media, mediana de verdad, aprobados, una columna por punto; en un ítem, también lo que ya está en Clickedu; al pulsar, quiénes son) y `registro::estadisticas` (por voz, a la primera, segundos por nota, tiempo ahorrado: 15 s/nota a mano, 3 min por importar); `exportado` y `subido` en el registro; `Columnas`, `Proporcion` y `Barras` de library-app; pestañas Evaluación · Ítem · Alumno (`leer_curso`, `estadisticas::curso`): nota de la evaluación calculada (sin ponderación no cuenta) junto a la «Nota final» de Clickedu, guardada al traer; el alumno, con la raya de la media | sí: el usuario, en la app (2026-11-25); tests de Rust (169) y la prueba de diseño |
| **H7** | `v2.0.0` y `v2.0.1` publicadas: `tauri-plugin-updater` con firma, `release.yml`, `npm run release`, el paso desde la v1.3.4 (`Instalador_GestorNotas.exe`, copia y desinstala la v1) | sí: el actualizador de la v1.3.4 y el de Tauri (2.0.0 → 2.0.1), de verdad; lo de D (AVX2), por el usuario (2026-11-24) |
| **H6** | El ítem se elige al calificar (Clases por evaluación, `clickedu::items`, sesión por ítem); lo que ya hay en Clickedu, a la vista (gris = publicada, tachada con punto ámbar = se pisaría); `clickedu::subir` (`completar`, `nombre_subida`, comprobar con `GetGrades`) y `DialogoSubir` con los pasos y el resultado en la misma ventana; plan B, el Excel y la página de importar | sí: el usuario, con Clickedu de verdad (ítem 42700, 2026-11-24); 151 tests de Rust contra el Clickedu falso; la prueba de diseño |
| **H5** | `clickedu::traer` (plan, dudas, `replanear_traer`, hacerlo: lo viejo a `antiguas`, las notas por `Id`, `traidas.json`); `TarjetaTraer` en Conexión, que entra sola con la cuenta guardada; las pendientes aparte en Conexión, Clases y Calificar (`plantillas::es_pendiente`); Conexión, encendida fuera de desarrollo | sí: a mano con Clickedu de verdad (`probar_traer` y la pantalla, 2026-11-22); 63 tests de `clickedu`; la prueba de diseño; en la app instalada, por el usuario (2026-11-25) |
| **H4** | `clickedu::cuenta` (usuario y contraseña en el Administrador de credenciales, el archivo de seguridad en `%APPDATA%`; se rellena una vez), `cliente` (entrar, comprobar, `abrir` en ISO-8859-15 y vuelve a entrar solo), `fichas` (materias, evaluaciones, ítems, importar, `GetGrades`; bien/degradado/roto), `huella`, `tapar`; Clickedu investigado solo leyendo (`explorar_clickedu`, `docs/03`); la pantalla Conexión, solo en desarrollo | sí: entrar, a mano con Clickedu de verdad; 43 tests contra el Clickedu falso y páginas reales anonimizadas; la sesión quieta, 111 min sin caer. **Volver a entrar solo, solo contra el falso** |
| **HUI** | La interfaz, antes de H4: la cuadrícula compacta y de 1 a 4 columnas; el dictado (lo apuntado grande y los dos de antes, dudas con 1–4/Esc, medidor de volumen, micrófono en Ajustes); el avance, «Siguiente sin nota» y las teclas tras un ⓘ; el título que es el desplegable; `BarraAvance`; las pantallas a todo el ancho | sí, a mano (A–D y a todo el ancho) y con la prueba de diseño |
| **H3b** | `voz::interprete` (la nota; lo no visto en grabaciones, `NOTA_RARA`, no se interpreta), `voz::emparejar` (fonética, los dos órdenes de «Apellidos, Nombre»), `voz::entender` (cuándo preguntar); `entender_frase`, el panel del dictado y las propuestas en la cuadrícula; `registro.rs` e informe sin nombres (D8), en Ajustes | sí, a mano (clase inventada), con tests de Rust, la prueba de diseño y `probar_entender` sobre las 100 grabaciones: **77 apuntadas, 20 preguntadas, 2 sin entender, 0 mal en silencio** |
| **H3a** | D2 cerrada con números: Whisper `small` `q5_1` recortado (90–94 % en 1,1 s); micrófono, remuestreo y motor; el banco de pruebas de voz; **el push-to-talk** (`useDictado`, `commands::voz`), la descarga comprobada del modelo (`voz::modelo`) y medir en local (`registro::medir`, sin audio); los bucles de Whisper se enseñan una vez y «[Música]» es silencio | el motor, con voz de verdad (100 grabaciones); el push-to-talk, a mano (clase inventada y descarga), con tests de Rust y la prueba de diseño |
| **H2** | `sesion.rs` (notas por clave de alumno, al momento), `exportar.rs` (el `.xlsx` de la v1, a `Documentos\Gestor de Notas\notas`); Calificar con campos de nota, Exportar y contador | sí, a mano, con tests de Rust (ida y vuelta) y la prueba de diseño; importado en Clickedu (H6) |
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
  sin cambiar de rama: `git show v1.3.4:GradingApp/core/voice_processor.py`,
  `git show v1.3.4:GradingApp/core/clickedu_client.py`. **Se porta, no se
  copia**: con tests, y arreglando lo que estaba mal (la mediana, D6).

## Antes de proponer o escribir nada, lee

1. [`docs/01-plan-de-trabajo.md`](docs/01-plan-de-trabajo.md) — **las
   decisiones D1–D8 y los hitos**
2. [`docs/02-sistema-visual.md`](docs/02-sistema-visual.md) — la paleta y la
   regla de color
3. [`docs/03-clickedu.md`](docs/03-clickedu.md) — cómo es Clickedu por
   dentro (H4–H6)

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
  `medir_registro`. **Reabierta** (2026-11-20): un cambio de Clickedu se
  avisa solo, por issue, cuando el repo sea privado (`docs/01`, D8).
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
  (`origin`, rama `main`, que es la v2; la v1, en el tag `v1.3.4`). **El usuario hace el push.**
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
