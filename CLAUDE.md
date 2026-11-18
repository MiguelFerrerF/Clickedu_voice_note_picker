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

> **Dónde se quedó (2026-11-18): H0 cerrado.** Rama `v2/h0-andamiaje`, con
> los neutros «niebla» y probado a mano. Lo siguiente es **H1 · Las clases y
> sus alumnos**: leer las plantillas de Clickedu en Rust (con tests sobre
> plantillas inventadas), la lista de clases y la cuadrícula en lectura, y la
> carpeta de plantillas vigilada.

| Hito | Qué dejó | Probado |
|---|---|---|
| **H0** | Andamiaje Tauri 2 + Vite + React + TS, paleta turquesa en claro con neutros «niebla», primitivas de `components/ui/`, `Sidebar` y cinco pantallas vacías, Muestrario, i18n, IBM Plex con `latin-ext`, prueba de diseño | sí, a mano y con la prueba de diseño |

## Cómo se mira lo que hay

```powershell
npm run tauri:dev        # la app
npm run typecheck        # los tipos
npm run probar:diseno    # pantallas a 960×640, 1100, 1280 y 1920, sin Tauri
cd src-tauri; cargo test # los tests de Rust (llegan con H1)
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
   decisiones D1–D7 y los hitos**
2. [`docs/02-sistema-visual.md`](docs/02-sistema-visual.md) — la paleta y la
   regla de color

## Decisiones que no hay que reabrir

Cerradas el 2026-11-17. No se reabren sin que el usuario lo pida.

- **D1 · Stack**: Tauri 2 + React + TS, lógica en Rust, como library-app.
- **D2 · Voz**: Whisper local. Solo se revisa si la prueba de H3a lo tumba.
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
  encuentra el actualizador de la v1.3.4 y lo instala solo.
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
  (`origin`, rama `main`). **El usuario hace el push.**
- Los commits terminan con la línea `Co-Authored-By` que indique la sesión.
