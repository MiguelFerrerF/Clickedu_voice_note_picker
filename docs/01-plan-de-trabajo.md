# 01 · Plan de trabajo

Escrito y aprobado el **2026-11-17**. Rehacer el Gestor de Notas sobre la base
de `C:\dev\library-app` (MiBiblioteca): mismo stack, mismo método y misma
mecánica visual, con **color propio** sacado del logo y **solo tema claro**.

---

## Punto de partida

La app actual (`GradingApp/` en `main`, Python + customtkinter, `v1.4.0` sin publicar;
la última publicada es la `v1.3.4`) funciona, pero:

- **El diseño no gusta** y no tiene sistema: 25 colores escritos a mano en 73
  sitios, 39 fuentes en línea, un color distinto por botón, textos cortados.
- **Dos dependencias frágiles**: la voz usa la API web no oficial de Google
  (`recognize_google`), y Clickedu se lee del HTML de la web. La API oficial de
  Clickedu existe (`api.clickedu.eu`), pero **no expone notas ni ítems** y
  **no nos dan credenciales**: no es una vía.
- **Fallos conocidos**: la mediana mal calculada con un número par de notas;
  `requirements.txt` incompleto (`beautifulsoup4`, `Pillow`) y con una
  dependencia que no se usa (`webrtcvad`); las plantillas no están donde dice
  el README; las credenciales en texto plano en `config.json` (ya salieron en
  un repo público y se purgaron el 2026-11-17).

### Qué se conserva (es el producto)

| Función | Hoy |
|---|---|
| Clases = plantillas Excel de Clickedu en una carpeta | `plantillas_clickedu_aulas/` |
| Cuadrícula de alumnos con su nota | Enter/↑/↓ saltan de alumno, coma → punto, hasta dos decimales |
| **Dictado push-to-talk** | Mantener ESPACIO: «Elena Gómez siete con cinco» |
| Entender la frase | Números en palabras, «con», «coma», «y medio»; palabras de relleno fuera |
| Encontrar al alumno | Normalización fonética (b/v, c/z, ll/y, h muda) + `token_set_ratio`, umbral 45 |
| Exportar | Copia del Excel con las notas, en `notas/`; las viejas a `notas/notas_viejas/` |
| Clickedu | Login (usuario + contraseña + archivo de seguridad), traer plantillas, subir notas a un ítem |
| Estadísticas | Media, mediana, aprobados, distribución, eficiencia del dictado |
| Actualizaciones | Busca en GitHub Releases el asset `Instalador_GestorNotas.exe` |

### Qué se toma de library-app

- **Stack**: Tauri 2 + Vite + React 18 + TypeScript, lógica en Rust.
- **Método**: decisiones por escrito antes de programar, hitos pequeños que
  van de lo necesario a lo superfluo, cada uno **probado a mano** antes de
  cerrarlo; `CLAUDE.md` corto con el estado y «Dónde se quedó».
- **Sistema visual**: `tokens.css` en `oklch` con los mismos nombres de token,
  primitivas de `components/ui/` traídas **una a una**, IBM Plex empaquetada,
  botones planos, campos «papel», **solo tema claro**.
- **La regla de color**: el color marca eventos y excepciones, nunca estados
  normales.
- **Reglas de la app**: no se congela nunca (todo `async`, lo largo con
  `Progreso`), no pierde lo que estabas haciendo, ninguna normalización en
  silencio.
- **La prueba de diseño**: capturas a 960×640, 1100, 1280 y 1920 con backend
  simulado, tras cada cambio visible.
- **Versiones**: `npm run release X.Y.Z`, notas en `docs/releases/`, el
  workflow deja un borrador que publica el usuario.

---

## Decisiones

**Cerradas el 2026-11-17**: el usuario aprobó todas las recomendaciones. No se
reabren sin que lo pida. Lo que queda abierto dentro de alguna está marcado
como tal.

### D1 · Stack: Tauri, como library-app

Lo que hoy hace Python tiene equivalente directo en Rust: Excel con `calamine`
(leer) y `rust_xlsxwriter` o el escritor propio de library-app; Clickedu con
`reqwest` (cookies, multipart) + `scraper`; el emparejamiento difuso con
`strsim` o un puerto de `token_set_ratio`. La v2 se hace en **una sola rama, `v2`**,
con un commit por hito y otro al cerrarlo (hasta H2 fue una rama por hito,
`v2/h0-andamiaje`…, en línea recta; se juntaron el 2026-11-18), **sin la app
Python** (se quitó el 2026-11-17):
la v1 sigue en `main` y en el tag `v1.3.4`, y su código se lee con
`git show main:GradingApp/…` cuando hay que portar algo.

### D2 · El motor de voz

La voz es lo que distingue a la app y hoy depende de un endpoint de Google no
oficial que puede cerrarse cualquier día. Además **manda la voz con nombres de
alumnos menores a un tercero**.

| Opción | A favor | En contra |
|---|---|---|
| **Whisper local** (`whisper-rs`) — *recomendado* | Sin red, sin terceros, buen castellano, se le puede dar la lista de alumnos como contexto | El modelo pesa 60–190 MB; hay que medir la latencia en el ordenador del profesor |
| Reconocimiento de Windows (WinRT) | Ya está en el sistema, latencia baja | Necesita el paquete de voz en castellano instalado; calidad desigual con nombres propios |
| Seguir con Google | Ya funciona | Lo frágil y lo de los datos siguen igual |

**Se va a Whisper local.** Queda abierto **solo** si la prueba de H3a lo
tumba: las mismas 50 frases grabadas, pasadas por cada motor, con aciertos y
latencia medidos. Si Whisper no llega, se vuelve aquí con los números.

### D3 · Las credenciales de Clickedu, fuera de los ficheros

En el **Administrador de credenciales de Windows** (crate `keyring`), nunca en
un JSON. El archivo de seguridad se copia a la carpeta de datos de la app.

### D4 · Dónde viven los datos

- **Lo del usuario, a la vista**: `Documentos\Gestor de Notas\plantillas` y
  `…\notas`. La plantilla de Clickedu es el origen y **no se escribe nunca**.
- **Lo de la app**, en `%APPDATA%\<identificador>\`: ajustes y el estado de la
  sesión (las notas que llevas puestas), guardado al momento, para que cerrar
  la app no pierda nada.
- **Nombre**: se queda «Gestor de Notas». `productName: "Gestor de Notas"`,
  `identifier: "es.gestornotas.app"` — no el DNS inverso de inBiot: es una
  herramienta para profesores, no un producto de la empresa.

### D5 · La paleta: turquesa del logo, en claro

El logo es `#0F9096` = `oklch(0.592 0.100 195)`. **Tal cual no sirve de
fondo de botón**: con texto blanco da 3,9:1 y el mínimo es 4,5:1. Punto de
partida, con el detalle en [`02-sistema-visual.md`](02-sistema-visual.md).
Los neutros son **«niebla»**, gris azulado frío, elegidos sobre pantalla en
el Muestrario el 2026-11-18:

| Token | Valor | Contraste |
|---|---|---|
| `--bg` | `oklch(0.982 0.006 240)` · `#F6FAFD` | — |
| `--accent` | `oklch(0.540 0.095 197)` · `#007F82` | blanco encima 4,8:1 |
| `--accent-ink` | `oklch(0.455 0.085 200)` · `#006469` | sobre fondo 6,6:1 |
| `--text-primary` | `oklch(0.250 0.018 250)` · `#1B222A` | 15:1 |
| `--danger` | `oklch(0.520 0.180 28)` · `#BA2C25` | 5,7:1 |
| `--success` | `oklch(0.540 0.130 150)` · `#258343` | 4,5:1 |

### D6 · Qué se pinta y qué no

Aplicando la regla de library-app:

- **Un suspenso no es rojo**: es una nota, un estado normal. Rojo solo para lo
  que está mal de verdad: un valor que no es nota, un fallo al subir.
- **Verde = evento**: la nota que acaba de entrar por voz, la subida que
  terminó bien. Se apaga sola.
- **Borde discontinuo = provisional**: el dictado entendió a un alumno con
  poca seguridad y espera que lo confirmes.
- **Sin poner** va en gris, aunque sea la mayoría al principio.

### D7 · Publicar y pasar a los usuarios de la v1.3.4

La nueva sale como **`v2.0.0`** en este mismo repo, **público** (el
actualizador de la v1.3.4 consulta la API de Releases sin token). Si el
instalador NSIS se sube con el nombre **`Instalador_GestorNotas.exe`**, la
v1.3.4 lo encuentra, lo descarga y lo instala sola: no hay que avisar a nadie.
Desde ahí, el actualizador de Tauri.

---

## Los hitos

De lo necesario a lo superfluo. Hasta H2 la app ya es útil **sin Clickedu y sin
voz**: abre plantillas, se ponen notas a mano y se exporta.

### H0 · Andamiaje ✓ (2026-11-18)

**La app abre, tiene el aspecto definitivo y no hace nada.**

- [x] Tauri 2 + Vite + React + TS en la raíz del repo, con las versiones de
      library-app; ventana 1280×800 (mín. 960×640); NSIS `currentUser`, solo
      NSIS (sin MSI).
- [x] `tokens.css` con la paleta de D5, todo en `:root`; `globals.css`,
      botones y campos copiados de library-app.
- [x] **Muestrario** (solo en desarrollo) con las tres familias de neutros,
      los componentes y una maqueta de la cuadrícula con los seis estados de
      una nota.
- [x] **Elegir la familia de neutros**: «niebla» (2026-11-18); fuera «agua»,
      «papel» y el selector.
- [x] IBM Plex empaquetada, con `latin-ext` para los nombres de alumnos; icono
      y logo sacados del `icon.png` de la v1 (`src-tauri/icons/origen.png` y
      `src/assets/logo.png`).
- [x] Primitivas una a una: `Button`, `Card`, `Select`, `TextInput`, `Icon`,
      `Dropdown`, `Progreso`, `Modal`; y `Cabecera`, `PantallaVacia`.
- [x] `Sidebar` con las pantallas vacías: **Calificar**, **Clases**,
      **Estadísticas** (Aula), **Conexión** (Clickedu), **Ajustes**
      (Aplicación).
- [x] i18n en castellano; alias de rutas; la prueba de diseño, que además
      busca **texto cortado sin «…»**.
- [x] `CLAUDE.md` del repo y `docs/` al estilo de library-app.
- [x] **Probado a mano** con `npm run tauri:dev` (2026-11-18).

### H1 · Las clases y sus alumnos ✓ (2026-11-18)

- [x] Leer las plantillas de Clickedu en Rust (`plantillas.rs`, `calamine`),
      con tests sobre plantillas inventadas que se escriben al vuelo (nunca
      alumnos reales en el repo).
      Las columnas se buscan **por su cabecera**, en castellano o catalán
      (`Alumno`/`Alumne`/`Nombre`, `Nota`/`Qualificació`, `ID`/`Codi`), en
      las diez primeras filas; si no se reconocen, ID · nombre · nota por
      posición, como la v1, **diciéndolo**. Lo que se corrige o no cuadra
      (nota con coma, valor que no es nota, alumno sin nombre, ID repetido)
      vuelve como aviso y se enseña encima de la cuadrícula. El nombre del
      alumno se deja **tal cual** (las reales traen algún doble espacio de
      Clickedu): no se corrige, así que no se avisa.
      **Comprobado sobre las siete plantillas reales** (2026-11-18, con
      `cargo run --example revisar_plantillas -- <carpeta>`, que no enseña
      datos de alumnos): hoja `Worksheet`, cabecera `Id | Alumnos | Nota` en
      la fila 1; todas se leen sin avisos.
- [x] La lista de clases con nombre legible (`biologia_1_ab_bach` →
      «Biología 1.º AB Bachillerato», `fisicaQuimica_…` → «Física y
      Química…»; el `Materia - Curso` de la
      sincronización se deja tal cual); la cuadrícula de alumnos **en
      lectura**, en Calificar.
- [x] Carpeta de plantillas configurable en Ajustes (por defecto
      `Documentos\Gestor de Notas\plantillas`, que se crea sola; una elegida
      que desaparece se dice, no se crea) y vigilada con `notify`: si se
      añade, cambia o quita un Excel, la lista se pone al día sola.
- [x] **Probado a mano** con `npm run tauri:dev` (2026-11-18), apuntando Ajustes a la
      carpeta de la v1.

La v1 instalada guarda las plantillas en
`%LOCALAPPDATA%\GestorNotasClickedu\plantillas_clickedu_aulas`: llevarlas
a la carpeta nueva es parte del paso desde la v1.3.4 (H7).

### H2 · Poner notas a mano y exportar ✓ (2026-11-18)

- [x] Escribir notas: 0–10, hasta dos decimales, coma → punto, Enter/↓
      guardan y pasan al siguiente, ↑ al anterior, Esc deshace. Lo que no es
      nota (`12`, `siete`, `7.555`) **no se guarda ni se corrige**: se queda
      en rojo con el motivo al lado del nombre.
- [x] El estado de la sesión se guarda al momento (`sesion.rs`:
      `sesion.json` en la carpeta de la app, escrito de una vez con cada
      nota). Cada nota va con la **clave** del alumno —su ID, o la fila si no
      tiene o está repetido—, no con su posición: si Clickedu reordena la
      lista, cada nota sigue con su alumno. Una nota de la plantilla se puede
      quitar (`null`), que no es lo mismo que no tocarla.
- [x] Exportar (`exportar.rs`) **replicando el de la v1**, que Clickedu
      aceptaba: un `.xlsx` nuevo, hoja `Sheet1`, las cabeceras de la
      plantilla, el ID como número (como texto si empieza por cero) y la nota
      **como texto con punto** sin ceros de más (`"7.5"`, `"7"`). En
      `Documentos\Gestor de Notas\notas\notas_<plantilla>.xlsx`; el de antes
      de esa clase pasa a `notas_viejas\` (`notas_x (2).xlsx` si ya hay uno).
      Lo que cambia respecto a la v1, que lo hacía en silencio: **si hay algo
      que no es nota no se exporta** y se dice quién; solo se aparta el
      exportado de esa clase, no todos. Test de ida y vuelta: exportar y
      releer con `plantillas::leer`, y celda a celda con `calamine`.
- [x] Contador de progreso (`18 / 24 con nota`), y la columna «Con nota» de
      Clases cuenta también la sesión.
- [x] **Probado a mano** con `npm run tauri:dev` (2026-11-18): poner notas,
      cerrar y abrir sin perderlas, exportar.

**Queda pendiente**, por decisión del usuario: importar un exportado en
Clickedu a mano para confirmar que lo acepta. Se hará más adelante, y como
muy tarde en H6, cuyo plan B es justo ese Excel. Si Clickedu lo rechaza, el
formato vuelve a abrirse aquí (`exportar.rs`).

### H3a · Voz: el motor (cierra D2)

- [ ] Grabar 50 frases de prueba con nombres inventados.
- [ ] Probar Whisper local y el reconocimiento de Windows (y Google como
      referencia): aciertos y latencia.
- [ ] Push-to-talk con ESPACIO, sin congelar nada, con indicador de «escuchando».

### H3b · Voz: entender la frase

- [ ] Puerto del intérprete a Rust con **tests sobre un corpus de frases**:
      «siete con cinco», «nueve y medio», «ocho coma veinticinco», «un diez».
- [ ] Puerto del emparejamiento fonético, con el mismo corpus.
- [ ] Confirmación: verde si está seguro; borde discontinuo y elegir entre los
      dos más probables si duda.

### H4 · Conectar con Clickedu

- [ ] Login con usuario, contraseña y archivo de seguridad; credenciales en el
      Administrador de credenciales (D3).
- [ ] **Capa de Clickedu resistente**: leer por los parámetros de las URLs
      antes que por las clases CSS, varias estrategias por dato, error claro
      («Clickedu ha cambiado: no encuentro los ítems») y el HTML guardado para
      diagnosticar.
- [ ] Tests con páginas de Clickedu guardadas y anonimizadas; script para
      capturarlas.
- [ ] Sesión caducada → volver a entrar sin que el usuario lo note.

### H5 · Traer las plantillas desde Clickedu

- [ ] Asignaturas y evaluaciones; descargar las plantillas que falten, con
      progreso.

### H6 · Subir notas a Clickedu

- [ ] Elegir asignatura, evaluación e ítem; subir; **comprobar** que entraron.
- [ ] Plan B: si falla, el Excel exportado y la página de importación abierta
      en el navegador. **Antes, confirmar que Clickedu acepta el exportado de
      H2**, que no se ha importado todavía.

### H7 · Actualizaciones y la `v2.0.0`

- [ ] `release.yml` y el actualizador de Tauri, como en library-app.
- [ ] El paso desde la v1.3.4 (D7), probado de punta a punta en una máquina
      con la v1.3.4 instalada.

**Con H7 cerrado, la `v2.0.0` sustituye a la app Python**: la rama `v2`
pasa a `main`, y la v1 queda en el tag `v1.3.4`.

### H8 · Estadísticas

- [ ] Media, mediana (bien), aprobados y distribución, con las reglas de
      gráficos de library-app (H8).
- [ ] Lo del dictado: notas por voz, aciertos a la primera, tiempo ahorrado.

### H9 · La ayuda guiada

- [ ] La primera vez, y una ayuda por pantalla, como H12 de library-app.

---

## La cola de lo superfluo

Solo si sobra tiempo y alguien lo pide: tema oscuro, sonidos de
acierto/error, el «modo ráfaga», otros idiomas.

## Lo que no se va a hacer

- **La API oficial de Clickedu**: no nos dan credenciales, y aunque las
  dieran, no tiene notas.
- **La API interna de la app móvil de Clickedu**: no está documentada y es
  más dudosa que leer la web.
- **Portar la interfaz de customtkinter**: se rehace, no se traduce.
