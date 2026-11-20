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

**Cerrada con la prueba de H3a (2026-11-18): Whisper `small` (`q5_1`,
181 MB), con la ventana recortada a la frase (más 2 s) y sin la lista de la
clase; el modelo se descarga la primera vez que se usa la voz.** Con voz de
verdad acierta nota y nombre en el 90–94 % de las frases, en 1,1 s de
mediana, como Google (90–92 %, 0,7 s) y sin que la voz salga del ordenador.
Windows no llega ni de lejos (14–34 %). El detalle, en H3a.

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

**Precisada el 2026-11-19** (el usuario): el actualizador es el **oficial
de Tauri, con firma** (el repo es público, así que no hace falta el propio
de library-app), y la v2 **desinstala la v1** tras copiar sus plantillas y
sus notas. Se adelanta a antes de H4 (ver H7).

### D8 · Aprender de los casos reales, sin que salgan los datos

**Cerrada el 2026-11-19** (el usuario pidió poder registrar y enviar los
errores para mejorar la app, y dejó la manera a criterio). Choca con D2:
la voz y los nombres de menores no salen del ordenador, y el repo es
público (nada de issues con casos). Por eso **sale la forma del error,
nunca su contenido**, como en `revisar_plantillas`:

1. **Registro en local** (`registro.rs`): en
   `%APPDATA%\es.gestornotas.app\registro\`, un `.jsonl` por mes, seis
   meses. Cada dictado (lo oído, lo entendido, los candidatos con su
   parecido y su distancia) y **la respuesta buena, que da el profesor sin
   hacer nada aparte**: elegir otra opción, descartar, deshacer, corregir a
   mano la nota recién dictada. Y cada error que devuelve un comando
   (`en_segundo_plano` lo apunta; uno igual, una vez al día) y los
   pánicos. Se ve y se borra en Ajustes.
2. **Un informe sin nombres ni voz** (`registro::informe`), que el profesor
   prepara en Ajustes, abre si quiere y manda **como fichero, por correo**
   (sin servidor ni terceros: son pocos usuarios). Lo oído va con **lista
   blanca**: solo quedan las palabras de dictar una nota (números, «con»,
   «coma», el relleno), y lo demás pasa a su forma (`▢5`); los alumnos son
   A, B, C; las clases, «clase 1»; en los errores, las rutas pasan a
   `%DOCUMENTOS%` y `~` y se tapan las palabras de los nombres de los
   alumnos. Con los números de los candidatos se ajustan los umbrales de
   `voz::entender` con datos reales.
3. **Medir en local** (`registro::medir`, `examples/medir_registro`,
   2026-11-19): cada versión nueva vuelve a pasar los casos guardados y
   solo cuenta (cuántos mal sin preguntar, cuántas preguntas). Así se sabe
   si un cambio mejora sin mover un nombre.

**El audio no se guarda** (el usuario, 2026-11-19): el registro lleva lo
oído y la respuesta, y con eso se mide de lo oído en adelante. Un cambio de
Whisper o del modelo se mide con las grabaciones del corpus (H3a), y un
nombre concreto mal oído, grabando frases parecidas con la clase
inventada.

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

### H3a · Voz: el motor (cierra D2) ✓ (2026-11-19)

- [x] **El banco de pruebas** (2026-11-18). El corpus, en
      `src-tauri/tests/datos/voz/`: una clase inventada de 28 alumnos
      (`clase.txt`) con las trampas de verdad —Martín/Martínez,
      García/Gracia, b/v, ll/y, h, catalanes, vascos y de fuera— y 50
      frases (`frases.tsv`) con el alumno, lo que se dice de su nombre y la
      nota; servirá también para H3b. Para grabarlas,
      `cargo run --release --example grabar_frases` (ESPACIO para empezar y
      para terminar: la consola no distingue soltar una tecla mantenida;
      a `Documentos\Gestor de Notas\prueba-voz`, fuera del repo). Para
      medirlas, `cargo run --release --example probar_voz`, que pasa cada
      modelo `ggml-*.bin` de `%APPDATA%\es.gestornotas.app\modelos` con y sin
      la lista de la clase, y con la ventana de 30 s o **recortada** a lo
      que dura la frase; `scripts/probar-voz/windows.ps1` (SAPI: dictado
      libre y con el vocabulario de la clase) y `google.py` (referencia)
      dejan sus resultados al lado, y el informe los compara: nota bien,
      nombre reconocible, mediana y p90 de lo que tarda.
- [x] `voz::microfono` (abierto todo el rato con los últimos 0,3 s en
      memoria: abrirlo al pulsar tarda 0,2–0,4 s y se comería el principio
      del nombre), `voz::remuestreo` (a 16 kHz, con tests) y
      `voz::whisper` (sin reintentos de temperatura y con 32 tokens como
      mucho: con ellos, algunas frases tardaban 8 s en bucle).
- [x] whisper.cpp con MSVC salía **sin `/O2` y sin AVX2**:
      `src-tauri/.cargo/config.toml` lo fuerza. AVX2 deja fuera algunos
      Celeron/Pentium baratos: **comprobar el procesador del ordenador del
      profesor** antes de H7.
- [x] **Ensayo con voz sintética** (la de Windows), solo para ver que la
      cadena funciona —no decide nada: es la voz de Microsoft hablándole al
      reconocedor de Microsoft—. En este Ryzen 5 5600H: Whisper `small`
      recortado, 80–86 % de notas en 0,6 s (1,6 s con la lista); `base`, peor
      y en 0,2 s; Windows con vocabulario, 98 % en 0,06 s; Google, 98 % en
      0,6 s. Muchos fallos de Whisper son de **escritura**, no de oído
      («Un10», «825», romanos como «V» o «X10»): los arregla el intérprete
      de H3b o el contexto.
- [x] Grabar las 50 frases con voz de verdad (el usuario, 2026-11-18). El
      grabador pasó a ESPACIO para empezar y ESPACIO para terminar: la
      consola de Windows manda una «soltada» con cada repetición de la tecla
      mantenida, y cortaba todo a 0 s. Salieron **bajas** (picos del
      5–9 %): Whisper y Google no lo notan; Windows tampoco mejora
      normalizándolas.
- [x] **La prueba, con voz de verdad** (2026-11-18, Ryzen 5 5600H, 6
      hilos). «Las dos» = nota y nombre bien en la misma frase:

      | Motor | Nota | Nombre | Las dos | Mediana | p90 |
      |---|---|---|---|---|---|
      | Google (referencia) | 94 % | 96 % | 90 % | 0,66 s | 0,90 s |
      | Whisper `small` | 92 % | 96 % | 90 % | 7,1 s | 7,8 s |
      | Whisper `small` + lista | 92 % | 100 % | 92 % | 7,2 s | 7,5 s |
      | **Whisper `small` + recorte** | 92 % | 92 % | **88 %** | **0,84 s** | 0,96 s |
      | ↳ con 2 s de margen (lo que se queda, abajo) | 94 % | 94 % | 90 % | 1,2 s | 1,5 s |
      | Whisper `small` + lista + recorte | 86 % | 94 % | 84 % | 1,75 s | 1,93 s |
      | Whisper `base` + recorte | 82 % | 78 % | 72 % | 0,34 s | 0,45 s |
      | Windows con vocabulario | 30 % | 46 % | 14 % | 0,14 s | 0,22 s |
      | Windows, dictado | 16 % | 20 % | 4 % | 0,67 s | 0,97 s |

      Lo que se saca:
      - **Recortar la ventana** cuesta 2 puntos y baja de 7 s a 0,84 s.
        Sin recorte no se puede usar.
      - **La lista de la clase como contexto no compensa**: con recorte
        empeora la nota (escribe «Cundiz», «A5», repite la frase) y dobla
        el tiempo. Los nombres se arreglan mejor después, emparejando con
        la clase (H3b).
      - De los 6 fallos de `small` + recorte, **5 los puede arreglar
        H3b**: notas mal escritas pero bien oídas («825», «5x25», la frase
        repetida en bucle hasta cortarse) y nombres que suenan a uno de la
        clase («Noria Villa SK», «Moncerrat será»). El único fallo de oído
        de verdad es «Wei diez», dos sílabas con un nombre de fuera; Google
        tampoco lo saca.
      - `base` se queda corto, y Windows queda descartado.
- [x] **Repetida sobre una mesa** (2026-11-19; la primera fue con el
      portátil sobre una cama). Suben los niveles (pico mediano del 7 % al
      10 %) y sube Windows (vocabulario 34 %, dictado 14 %), que sigue
      lejos. **`small` + recorte da otra vez 88 %** (nota 94 %, nombre 90 %,
      0,82 s); Google, 92 %. Entre variantes de Whisper las diferencias son
      de una o dos frases (`small` + lista + recorte, 90 % en 1,8 s): ruido
      de 50 frases, no razón para cambiar D2. Ya no hay bucles ni «825»;
      los fallos son nombres a medio oír («Sofí el baño», «Muhometino»,
      «Etxeberria siete» oído como «7»), «siete y medio» como «7.30»,
      «Ximena nueve y medio» como «Si menos, no hay medio», y otra vez «Wei
      diez». Todas las grabaciones llevan el clic de ESPACIO a los 0,35 s:
      a Whisper no le molesta.
- [x] **Afinar el motor** (2026-11-19), con las 100 grabaciones (cama y
      mesa), cambiando una cosa cada vez sobre `small` + recorte (88 % y
      88 %):
      - **Más margen a la ventana recortada: 2 s en vez de 1 → 90 % y
        94 %**, por 0,15 s más. Con 3 o 4 s, nada más; añadir 1 s de
        silencio al final da lo mismo que el margen. **Se queda en 2 s.**
      - Buscar con haz (5) en vez de lo más probable: 86 % y 90 %, un 30 %
        más lento. No.
      - **Obligarle a escribir solo el vocabulario de la clase** (nombres,
        números y las palabras de dictar), filtrando lo que puede escribir
        en cada paso: **los nombres caen al 66–70 %**. Whisper escribe a
        trozos; si quería empezar «Xirau» por «Shi» y no puede, salta a otra
        palabra permitida («Montserrat, Sánchez»). Más suave (castigar en vez
        de prohibir), 78–82 %. No: el nombre se empareja con la clase
        después, en H3b. (La gramática de whisper.cpp no se puede usar desde
        `whisper-rs` 0.16: se la pasa mal.)

      Con el margen de 2 s quedan **8 fallos de 100**, y **6 son de H3b**:
      nombres que suenan a uno de la clase («Noria Villa SK», «Moncerrat
      será», «Sofí el baño», «Muhometino») y notas oídas pero mal escritas
      («825», «5x25»). El único fallo de oído es «Wei diez» (las dos
      veces): ahí la app tiene que **preguntar, no adivinar**.
- [x] **El modelo se descarga la primera vez que se usa la voz** (el
      usuario, 2026-11-19): el instalador sigue pequeño. La descarga lleva
      `Progreso` y se comprueba antes de usarla. Hecho con el push-to-talk
      (`voz::modelo`): de Hugging Face, **fijado a un commit** de
      `ggerganov/whisper.cpp`, a `.parcial`, y solo se deja en su sitio si
      llegan los 190 085 487 bytes y su SHA-256 es el del modelo de la
      prueba (comprobado: el de allí y el de aquí coinciden). En Calificar,
      la oferta de descargarlo («181 MB, una vez, y la voz no sale de este
      ordenador») y su avance.
- [ ] **Queda abierto**: la latencia en un ordenador de profesor, que será
      más lento que este.
- [x] **Push-to-talk con ESPACIO** (2026-11-19), sin congelar nada
      (`useDictado` y los comandos de `commands::voz`):
      - **El micrófono se abre mientras se puede dictar**: Calificar a la
        vista, con una clase abierta y la ventana delante. Si no, se cierra,
        y Windows deja de enseñar que se usa. Abierto guarda los últimos
        0,3 s, y al abrirlo se carga Whisper, para que la primera frase no
        espere.
      - **ESPACIO se coge antes que nadie**: ni baja la página ni pulsa el
        botón con el foco, y vale con el foco en una celda de nota. Lo que
        necesita escribir espacios se marca con `data-sin-dictado` (el campo
        de probar el dictado, que se queda solo en desarrollo).
      - Una pulsación de menos de 0,3 s no se manda a Whisper y se dice cómo
        se dicta. **Lo grabado sin voz** (pico por debajo de 0,01; en las
        100 grabaciones el ruido llega a 0,0002 y la frase más baja a 0,045)
        tampoco: Whisper, con silencio, se inventa frases. Se dice que no se
        ha oído nada.
      - Mientras Whisper entiende una frase se puede decir la siguiente.
        Si la ventana deja de estar delante con ESPACIO pulsado, lo grabado
        se tira. La voz no se escribe en disco.
      - En el panel del dictado: cómo se dicta y por qué micrófono se
        escucha, «Escuchando…» en turquesa (está pasando), «Entendiendo…»
        en gris, y los fallos del micrófono en rojo.
- [x] **Probado a mano** con `npm run tauri:dev` (2026-11-19): dictar de
      verdad sobre la clase inventada (apuntadas, dudas, no entendidas,
      silencio, pulsación corta, cambiar de ventana dictando) y descargar el
      modelo apartando antes el que hay. Nada mal en silencio. Salieron dos
      cosas, arregladas:
      - **Un bucle** («Marta Ortera Lucia, 7 Marta Ortera Lucia, 7 … Mart»,
        por «Martín Ortega, Lucía, siete»): el intérprete ya se quedaba con
        la primera vez y apuntó bien, pero el panel enseñaba el texto en
        bruto y parecía dictado tres veces. Ahora enseña la frase una vez y
        dice que venía repetida (`interprete::una_vez`, el `repetida` del
        `Dictado`); el registro guarda lo oído tal cual.
      - **«[Música]»**: lo que escribe Whisper cuando no oye voz, en una
        pulsación sin hablar que pasó del umbral de silencio. Salía como
        frase sin entender; ahora, lo que no tiene letras ni cifras fuera de
        corchetes o paréntesis es silencio (`Oido::sin_palabras`): «No se ha
        oído nada», y no se apunta.
- [x] **Medir en local con el registro** (D8, 2026-11-19):
      `cargo run --example medir_registro`. Por cada dictado del registro,
      lo que había que apuntar sale de lo que hizo el profesor
      (`registro::medir::verdad`: un seguro que nadie tocó estaba bien; lo
      elegido o corregido es lo bueno; lo deshecho o descartado, no). Se
      vuelve a entender lo oído con los alumnos de su clase y se cuenta,
      entonces y ahora con la misma vara, cuántas apuntadas bien, preguntadas
      (con la buena o sin ella), no entendidas y **mal en silencio**; las
      frases que cambian, tapadas como en el informe. Sale con error si
      ahora hay más mal en silencio que entonces. El registro apunta ahora
      si la frase llegó **por voz** o escrita en desarrollo (`voz`; lo de
      antes, escrito): por defecto solo cuentan las dictadas.
      **El audio no se guarda** (el usuario, 2026-11-19): ver D8.

### H3b · Voz: entender la frase ✓ (2026-11-19)

> **Va antes que el push-to-talk de H3a** (el usuario, 2026-11-19): así,
> al probarlo con la voz, Calificar ya enseña alumno y nota. Se mide con
> las 100 grabaciones de H3a (cama y mesa): el éxito es **cero notas mal
> puestas en silencio**; lo que no esté claro, se pregunta.

- [x] **El intérprete** (`voz::interprete`, 2026-11-19), con tests sobre
      las frases del corpus y sobre lo que escribió Whisper al oírlas: la
      nota dicha o escrita («siete con cinco», «9 y medio», «8,25», «cinco
      con 25», «siete coma cero cinco»); «un» y «una» delante de un número
      son artículo (la v1 los cambiaba por 1 y cogía el último número); los
      **bucles** («Pablo García, un 10. Pablo García, un 10. Pablo…») se
      quedan en la primera vez; el relleno («ponle», «para», «tiene») se
      quita solo en los extremos del nombre («Mohamed **El** Amrani»). **Lo
      que se lee de dos maneras no se decide**: «825», «5x25» y «75» se
      entienden pero se preguntan; «7.30» puede ser «siete y media»
      escrito como hora (pasó una vez en H3a), así que ofrece 7,5 y 7,3; un
      «con cinco» sin nada delante (se perdió el «cuatro») no es un 5, y un
      número de más que no es nota («ochenta») hace preguntar.
- [x] **El emparejamiento** (`voz::emparejar`). Fonética para castellano y
      catalán (b/v, ll/y/i, h, c/z/s/x, qu/k, g/j, tx/ch, la «ig» de
      «Puig»), y dos cambios sobre la v1, que comparaba por palabras: **se
      compara todo junto, sin espacios** (Whisper escribe «Hay tan la
      Cervera», «Weicheng», «Eche verría»), y **con cada forma de decir el
      nombre** (cada subsecuencia de sus palabras: «Etxeberria» solo es
      Iker Etxeberria entero). Una partícula sola («el», «de») no es forma:
      si no, «el cinco» era de Mohamed El Amrani.
      **Clickedu da los nombres como «Apellidos, Nombre»** (las siete
      plantillas reales, todos), y comparar en orden no casaba «Lucía
      Martín» con «Martín Ortega, Lucía»: con coma, se prueban los dos
      órdenes (2026-11-19). Con la clase del corpus escrita así
      (`probar_entender -- --como-clickedu`) sale lo mismo: 77 / 20 / 2 / 0.
- [x] **Cuándo se pregunta** (`voz::entender`), medido con las 100
      grabaciones (`cargo run --release --example probar_entender`, que
      guarda lo oído en `resultados\oidos.json` y cuenta apuntadas,
      preguntadas, no entendidas y **mal en silencio**):
      - **La confianza de Whisper por palabra no sirve**: el único nombre
        mal oído que podía pasar en silencio, «Lucía Martín» oído «Lucía
        **Martínez**» (que también está en la clase), tenía un 57 %, y los
        bien oídos van del 10 % al 95 %. Whisper reparte la probabilidad
        entre maneras de escribir lo mismo («10»/«diez», «9,5»/«9.5»), no
        entre cosas oídas. Se guarda (`whisper::Oido`) para volver a
        mirarlo con otro modelo, pero no decide nada.
      - Lo que sí: **si con cambiar dos sonidos de lo oído sería otro
        alumno, se pregunta**, ofreciendo los dos. Pilla Martín/Martínez y
        García/Gracia; diciendo el nombre entero («Lucía Martín Ortega») ya
        no pregunta. También se pregunta si lo oído se parece poco (menos
        del 75 %) al más parecido, y no se apunta nada por debajo del 50 %.

      | `small` + recorte, 100 frases | Apuntadas | Preguntadas | No entendidas | **Mal en silencio** |
      |---|---|---|---|---|
      | Cama y mesa | 77 | 20 (+1 sin el bueno entre los dos) | 2 («Wei diez») | **0** |

      De las 20 preguntas, 14 son de las dos parejas trampa del corpus
      (Martín/Martínez y García/Gracia, dichas sin el segundo apellido) y 2
      de notas escritas «825» y «5x25». Con `--todos` entiende también lo
      que dejaron en H3a los demás motores y variantes: **0 mal en
      silencio** con Google y con cualquier Whisper, salvo el obligado al
      vocabulario de la clase (descartado en H3a), que escribe el apellido
      de otro alumno («y quiroga 5» por «Iker cinco»): un motivo más para
      no usarlo.

      **La regla se queda así** (el usuario, 2026-11-19), aunque pregunte
      por las parejas que se llaman casi igual si no se dice el segundo
      apellido.

      **Lo que no salió en las grabaciones no se interpreta** (el usuario,
      2026-11-19): «siete y cuarto», «seis menos», «notable», «aprobado»
      no se apuntan y se dice por qué (`interprete::NOTA_RARA`). Si salen
      en casos reales (D8), se verá cómo leerlas. Y si solo se duda de la
      nota («825»), no se ofrece otro alumno.
- [x] **La confirmación en Calificar** (2026-11-19). El comando
      `entender_frase` entiende una frase con los alumnos de la clase y
      devuelve cada uno por su **clave** (la de la sesión). Un **panel del
      dictado** encima de la cuadrícula, pegado arriba al bajar por la
      lista, enseña lo último:
      - **Seguro**: se apunta; la celda, en verde con el micrófono, y el
        panel, «Apuntado un 9.5 a …», se apagan solos a los 6 s. La fila se
        trae a la vista. **«Deshacer»** deja la nota de antes, exacta: si
        el alumno no se había tocado, vuelve a la de la plantilla
        (`devolver_nota`, que no es lo mismo que quitarla).
      - **Duda**: no se apunta nada. Borde discontinuo, la pregunta («¿Para
        quién es el 9.5?»), por qué se duda y las opciones (cada alumno con
        cada nota, cuatro como mucho, lo más probable primero) o
        «Descartar». En la fila de cada candidato, la nota propuesta con
        borde discontinuo («¿9.5?»), que también se puede pulsar.
      - **No entendida**: el punto ámbar y por qué, no rojo: la app no ha
        fallado.
      - Siempre se enseña lo oído, en gris, para saber de dónde sale.
      Hasta el push-to-talk, en desarrollo, un campo para escribir la frase
      como si se dictara. Prueba de diseño con los tres casos.
- [x] **El registro (D8)**, con la confirmación (2026-11-19): cada frase
      entendida se apunta con sus candidatos, y lo que haces con ella
      (elegir, descartar, deshacer, corregir a mano) es su respuesta. En
      Ajustes, «Mejorar el dictado»: cuánto hay, preparar el informe y
      borrar. Tests: el informe no lleva nombres, claves, clases ni rutas, y
      ninguna de las 50 frases del corpus deja pasar una palabra de un
      nombre.
- [x] **Probado a mano** con `npm run tauri:dev` (2026-11-19), con la clase
      inventada (`cargo run --example escribir_clase_inventada`): frases que
      se apuntan, que preguntan y que no se entienden, elegir, deshacer, y
      preparar y abrir el informe.

### HUI · Mejorar la interfaz (antes de H4) — cerrado el 2026-11-19

Pedido por el usuario el 2026-11-19, tras probar la voz: mejorar la
visibilidad, la interfaz y la experiencia. **Se hacen todas, un bloque cada
vez**, con la prueba de diseño y sus capturas miradas; un commit por cambio.
Ninguna rompe D5/D6 ni añade tokens. **Cerrado por el usuario el
2026-11-19**, con todo probado a mano (A–D y el 18).

- [ ] **En espera: la distribución de las pantallas.** Ahora que ocupan la
      ventana entera (el 18), según vayan saliendo cosas nuevas (H4–H8) se
      vuelve a mirar cómo se orientan y reparten los elementos de cada
      pantalla. No se hace nada hasta que haga falta.

- [x] **Compactar la cuadrícula** (2026-11-19): dos columnas desde 760 px,
      filas de 32, sin caja en la nota vacía (era la 7). Ver `docs/02` §3.

**A · El dictado**

- [x] 1. **El resultado, más grande**: «Apuntado un 9.5 a …» va a 13 px y
      es lo que hay que leer de un vistazo mientras se dicta. Nombre y nota a
      16–18 px (nota en mono), y **la fila entera** del alumno en verde un
      momento, no solo la celda (la tabla ya baja sola hasta él).
- [x] 2. **Los últimos 3–5 dictados**, cada uno con su «Deshacer»; los
      viejos en gris. Ahora solo se ve el último y, dictando seguido, el de
      antes se va sin comprobarlo. Hechos los dos (2026-11-19): tres, el
      nuevo grande; solo deshace el último dictado a cada alumno.
- [x] 3. **Las instrucciones, legibles**: «Mantén ESPACIO pulsado…» va en
      `--text-tertiary` (3,5:1, solo para rótulos según `docs/02`). A
      `--text-secondary`, y ESPACIO dibujado como una tecla.
- [x] 4. **Medidor de volumen** mientras se mantiene ESPACIO: se ve que el
      micrófono oye (evita casos como «[Música]»). Necesita Rust: mandar el
      nivel por el canal del micrófono. Hecho (2026-11-19): el pico de cada
      50 ms, en decibelios, por un `Channel` de `empezar_dictado`; probado
      con el micrófono de verdad (`cargo test -- --ignored microfono`).
- [x] 5. **Las dudas con el teclado**: 1–4 eligen la opción, Esc descarta.
- [x] 6. **Las dudas, sin repetir el nombre**: una fila por alumno con sus
      notas como botones cortos, en vez de un botón por alumno y nota.
- [x] 17. **El micrófono, en Ajustes** (pedido por el usuario el
      2026-11-19, en lugar del «Se escucha por…» del panel): elegir por
      cuál se dicta, por ejemplo unos auriculares; si no, el de Windows. Se
      guarda su id de `cpal` en `ajustes.json`; si no está conectado, se
      escucha por el de Windows y se dice, en Ajustes y en el panel. La
      lista se mira cada 3 s con Ajustes a la vista.

**B · La tabla de notas**

- [x] 8. **El progreso, visible**: barra fina (turquesa, se permite para
      progreso) en vez de «0 / 18 con nota» en pequeño, y «Ir al siguiente
      sin nota». Hecho (2026-11-19): encima de la tabla, «Siguiente sin
      nota» sigue desde el último campo tocado.
- [x] 9. **La ayuda del teclado** («Enter o ↓ guarda…»), arriba o al
      enfocar la primera celda: debajo de la tabla no se ve con 30 alumnos.
      Hecho (2026-11-19): un ⓘ junto al avance abre las teclas, una por
      fila, las de la tabla y las del dictado (el usuario prefirió eso a una
      línea siempre a la vista). **Supr** vacía la celda y quita la nota;
      Esc sigue devolviendo lo de antes de escribir. Una pantalla «Ayuda»
      en la barra lateral, pedida también, se deja para H9.

**C · La cabecera de Calificar**

- [x] 10. **El título es el desplegable**: ahora el nombre de la clase sale
      dos veces (título y desplegable). Hecho (2026-11-19): `Dropdown`
      con `size="titulo"`, sin caja y con la letra de la `Cabecera`; se
      corta con «…» solo si no cabe.
- [x] 11. **«Exportar» principal solo si hay algo que exportar**; antes, gris.
      Hecho (2026-11-19): sin ninguna nota, fantasma y sin pulsarse (el
      Excel saldría vacío y mandaría el bueno a `notas_viejas`); una que no
      es nota cuenta, para que al pulsar se diga cuál.
- [x] 12. **Los avisos de la plantilla, plegados** en una línea («● 3 avisos
      al leer la plantilla ›») que se abre al pulsarla. Hecho (2026-11-19):
      abiertos, la lista debajo, sin caja; otra clase empieza plegada.

**D · El resto**

- [x] 13. **«Descargar el modelo de voz», botón principal**: es lo que hace
      falta para dictar. Hecho (2026-11-19).
- [x] 14. **Ajustes**: las seis cifras del registro, en una fila o en 3×2
      (a 1280, «Fallos» se queda sola abajo). Hecho (2026-11-19): seis desde
      640 px de tarjeta, si no tres.
- [x] 15. **Clases**: que se vea que la fila se pulsa (fondo al pasar, `›` al
      final) y «Con nota» como mini barra en vez de «12 / 31». Hecho
      (2026-11-19): el fondo ya estaba; la barra es la de Calificar
      (`BarraAvance`), con la cifra en su etiqueta y al pasar el ratón.
- [x] 16. **La barra lateral**: el Muestrario, solo en desarrollo;
      Estadísticas y Conexión, «próximamente» o escondidas hasta H4/H8.
      Hecho (2026-11-19): escondidas (`HAY_ESTADISTICAS`, `HAY_CLICKEDU` en
      `Sidebar`); el Muestrario ya era solo de desarrollo.
- [x] 18. **Las pantallas, a todo el ancho** (pedido por el usuario el
      2026-11-19): tenían un tope de 880 o 1040 px, como library-app, y a
      1920 sobraba un tercio. Ahora ocupan la ventana entera; la cuadrícula
      de Calificar pone una columna más por cada 460 px, hasta cuatro. Se
      quedan con tope solo los párrafos de texto y el desplegable del
      micrófono.

### H7 · Actualizaciones y la `v2.0.0` (adelantado, antes de H4)

**Adelantado por el usuario el 2026-11-19**, para que la v2 empiece a
usarse y a reunir casos reales (D8) cuanto antes. Lo que pesaba en contra
no aplica: la v1.3.4 publicada **no se conecta a Clickedu** (eso solo está
en `main`, sin publicar, como prueba interna) y **la tiene una sola
persona** (8 descargas del instalador, casi todas de pruebas). Lo único
que la v2 pierde frente a la v1.3.4 son las estadísticas (H8). Condición
del usuario: **antes, el actualizador de Tauri y todo bien asegurado.**
Un commit por bloque, como en HUI.

**A · El actualizador de la v2** (D7) — hecho el 2026-11-20, sin publicar

- [x] `tauri-plugin-updater` **oficial, con firma** (no el propio de
      library-app, que existe porque su repo es privado y no comprueba
      firmas), con `requireSignedVersion`: la firma lleva la versión y no
      se puede hacer pasar una vieja por nueva. Lo hace todo Rust
      (`commands/actualizacion.rs`); la interfaz no tiene permisos del
      actualizador. Con el TLS del sistema, como `reqwest`. Probado: una
      compilación firmada con una clave de prueba lleva `version:2.0.0` en
      la firma, y el plugin solo mira la versión, no el nombre del fichero.
- [x] **La clave**: la genera el usuario y no pasa por la conversación. La
      pública ya está en `tauri.conf.json` (minisign `E99FFA4B21E16BB4`,
      2026-11-20; `npm run release` y CI se paran si vuelve el
      `PON_AQUI_LA_CLAVE_PUBLICA`); la privada y su
      contraseña, **solo** en los secrets de Actions
      (`TAURI_SIGNING_PRIVATE_KEY`, `…_PASSWORD`) y en una copia del
      usuario fuera del repo. Sin ella, la app instalada no acepta más
      actualizaciones.
- [x] `release.yml` como el de library-app por su estructura: un tag
      `vX.Y.Z` compila en `windows-latest` y deja un **borrador**;
      «Publish» es a mano. Antes: notas, versión del tag en los cuatro
      ficheros, clave pública puesta, `npm audit` y `cargo audit`.
      Permisos mínimos, acciones fijadas por SHA, tres secrets. El
      instalador sube **directamente como `Instalador_GestorNotas.exe`**
      (`releaseAssetNamePattern`), con su `.sig` y `latest.json`: un solo
      fichero para los dos actualizadores. Después comprueba que
      `latest.json` apunta a esta versión y que el subido es el compilado
      (SHA-256, en el resumen). Sin probar en CI todavía.
- [x] La versión igual en los cuatro ficheros (`npm run release`, que **no
      hace push**) y las notas en `docs/releases/vX.Y.Z.md`, como
      library-app. Borrador de las de la `v2.0.0`, por revisar.
- [x] El diálogo: busca al arrancar y a mano (pie de la barra), nunca
      instala sin pulsar, deja saltarse una versión; avance real al
      descargar. Insignias con color, como library-app (pedido el
      2026-11-20; nunca rojo, D6). Con la prueba de diseño. Los errores,
      solo en castellano: el detalle del plugin (en inglés), al registro.
- [x] **Probado de punta a punta en local** (2026-11-20, con el usuario):
      `0.9.0` y `0.9.1` con `tauri build --debug` (así la migración no mira
      la v1 de verdad), `--config` con una clave de prueba y un servidor en
      `127.0.0.1` (`dangerousInsecureTransportProtocol`). Visto: el aviso
      al arrancar y a mano, la descarga con avance, instalar (se cierra y
      vuelve a abrirse en la `0.9.1`), «al día»; **rechaza** una firmada
      con otra clave y una que anuncia `0.9.5` con la firma de la `0.9.1`;
      sin red, «no se ha podido mirar» y nada al arrancar. `tauri build`
      mete `version:` en la firma, que es lo que hará CI. El desinstalador
      con `/S` no borra `%APPDATA%\es.gestornotas.app`.

**B · El paso desde la v1.3.4** — hecho el 2026-11-20, sin probar con la v1 de verdad

La v1.3.4 está en `%LOCALAPPDATA%\GestorNotasClickedu\` y guarda
**dentro** `plantillas_clickedu_aulas\` y `notas\`. **Su instalador pide
administrador**: aunque la carpeta es del usuario, la apunta en
`HKLM\…\WOW6432Node\…\Uninstall\Gestor de Notas Clickedu_is1` y deja los
accesos directos para todos («Gestor de Notas Clickedu», que no choca con
los de la v2). Quitarla pide permiso a Windows (UAC).

- [x] La v1.3.4 instala lo que encuentre sin comprobar nada (solo HTTPS):
      **verificación en dos pasos en la cuenta de GitHub** (ya la tenía
      activada, 2026-11-20).
      El hash del subido frente al compilado ya lo comprueba CI.
- [x] Al abrir la v2, si la v1 está (`migracion.rs`, por el registro:
      HKCU o HKLM, en las dos vistas): **copiar** sus plantillas y sus
      notas a `Documentos\Gestor de Notas\` (nunca mover), cada copia
      comparada byte a byte, sin pisar nada (uno distinto entra como «…
      (de la v1)»); si Ajustes apuntaba dentro de la v1, pasa a la copia.
      Se dice en un diálogo. **Quitarla es un botón** de ese diálogo,
      avisando de que Windows pedirá permiso (decidido al ver que su
      instalador pide administrador: un UAC que salta solo al arrancar
      asusta): su `unins000.exe` en silencio, solo con la v1 cerrada y
      tras volver a copiar. Con «Más tarde», se ofrece al abrir otra vez.
      Si copiar falla, se dice y la v1 se queda. En desarrollo no se toca
      la v1 de verdad (`GESTORNOTAS_V1`).
- [x] Probado de punta a punta en este ordenador, con la v1.3.4 de verdad
      (2026-11-20, con permiso y con copia de seguridad en
      `%USERPROFILE%\Copia Gestor de Notas antes de la v2 (2026-11-20)`):
      el instalador del borrador de CI; las 7 plantillas, copiadas byte a
      byte (sin el README ni nada más); lo que ya había en Documentos, sin
      tocar; quitarla con el botón y el UAC. **El desinstalador de la v1
      borra sus plantillas**: copiar antes era imprescindible. Deja
      `%LOCALAPPDATA%\GestorNotasClickedu` con tres carpetas vacías.

**C · Asegurar la app** — hecho el 2026-11-20

- [x] CSP estricta: sin `unsafe-eval` ni red salvo el IPC; Tauri añade el
      hash de su script. Probada en la compilada (las tres pantallas sin
      violaciones, con las fuentes y el logo); en `tauri dev` no se aplica.
      En una de las tandas, la página se cayó una vez («Target crashed»)
      y no se ha vuelto a ver en seis más: a vigilar.
- [x] Capacidades: solo `core:default`. Los comandos que reciben un
      fichero ya rechazan rutas (`leer_plantilla`, `mostrar_informe`).
- [x] Sin herramientas de desarrollo en la compilación publicada (Tauri no
      las compila sin la *feature* `devtools`).
- [x] `cargo audit` y `npm audit`, en CI. `cargo audit` encontró
      `calamine` 0.36 (se caía con un `.xls` malformado) y el `zip` 8.6
      vulnerable: subidos `calamine` 0.38 y `rust_xlsxwriter` 0.103, con
      todos los tests y las plantillas reales leídas igual. Quedan tres
      avisos sin arreglo: `chrono` y `proc-macro-error` sin mantener,
      `glib` (solo Linux).
- [x] `gitleaks` sobre todo el historial, todas las ramas y el reflog:
      limpio.
- [x] En GitHub, el escaneo de secretos con bloqueo al hacer push:
      activados (comprobado con `gh api` el 2026-11-20).
- [ ] La contraseña de Clickedu que se filtró en
      `config.json`, **cambiada**: purgar el historial no la quita de las
      copias que GitHub y otros guardan.
- No se firma el `.exe` con certificado de Windows: cuesta dinero y no
  compra nada con una sola persona usándola.

**Lo que falta, uno a uno y guiando al usuario** (pedido el 2026-11-20):

1. ~~**La clave de firma**~~ (hecho el 2026-11-20, `E99FFA4B21E16BB4`): el usuario la genera **en su propia terminal**
   (no con `!`, para que la privada no pase por la conversación):
   `npx tauri signer generate -w "$env:USERPROFILE\.tauri\gestor-notas.key"`,
   con contraseña. El `.key` y la contraseña, a su gestor de contraseñas.
   Claude lee **solo** el `.pub` y lo pone en `tauri.conf.json` en lugar de
   `PON_AQUI_LA_CLAVE_PUBLICA`.
2. ~~**Los secrets**~~ (hecho el 2026-11-20; la clave, puesta con
   `--body` y recortada: por tubería, PowerShell 5.1 le añade un salto de
   línea) en GitHub (Settings → Secrets and variables → Actions):
   `TAURI_SIGNING_PRIVATE_KEY` (el contenido del `.key`) y
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
3. ~~**Verificación en dos pasos**~~ (las dos cosas, ya activadas) en la cuenta de GitHub, y en el repo el
   **escaneo de secretos con bloqueo al hacer push** (Settings → Code
   security).
4. **Preguntar** si la contraseña de Clickedu filtrada en `config.json` se
   cambió. **Pendiente** (2026-11-20): el usuario no está seguro y la
   cambiará cuando tenga un archivo de seguridad nuevo (el suyo está
   caducado). No frena la `v2.0.0`.
5. ~~Revisar con el usuario `docs/releases/v2.0.0.md`~~ (hecho el
   2026-11-20, y además el actualizador probado en local: ver A).
6. ~~`npm run release 2.0.0` y el push del usuario~~ (hecho el
   2026-11-20, tag en `10f9d64`): CI dejó el **borrador**, sin «Publish»,
   con el SHA-256 del instalador `fb6a3760…4460b`. Por el camino: el
   script no etiquetaba si la versión ya estaba puesta; el secret de la
   contraseña tenía mal la contraseña (se volvió a poner con
   `gh secret set` a secas, que la pide sin salto de línea); y
   `tauri-action` 1.0 pone en `latest.json` la URL del API del asset (el
   actualizador la baja con `Accept: application/octet-stream`), así que la
   comprobación exige esa, la del instalador del mismo borrador, y la
   firma del `.sig`. **Siguiente: el paso 7.**
7. ~~**Con permiso del usuario**, probar en este ordenador el instalador
   del borrador~~ (hecho el 2026-11-20: ver B). **Siguiente: el paso 8.**
8. Lo de D, y «Publish»: la v1.3.4 de esa persona la encuentra sola.
9. Una `v2.0.1` para probar el actualizador de Tauri de verdad. Lleva
   (pedido el 2026-11-20) quitar la carpeta de la v1 si solo le quedan
   carpetas vacías (`migracion::quitar_si_vacia`), al quitarla y al
   arrancar. Notas en `docs/releases/v2.0.1.md`, por revisar.

**D · Antes de dársela a esa persona**

- [ ] Su procesador tiene AVX2 (H3a).
- [ ] Un exportado de la v2 importado en Clickedu (lo pendiente de H2).
- [ ] La primera publicada es la `v2.0.0`; luego, `v2.0.1`… La prueba de
      verdad del actualizador de Tauri es la segunda.
- Antes de pasarla **a más gente**: un informe del registro (D8) de
  verdad, mirado línea a línea para ver que no lleva nada de nadie.

**Con H7 cerrado, la `v2.0.0` sustituye a la app Python**: la rama `v2`
pasa a `main`, y la v1 queda en el tag `v1.3.4`.

### H4 · Conectar con Clickedu

- [ ] Login con usuario, contraseña y archivo de seguridad; credenciales en el
      Administrador de credenciales (D3).
- [ ] **El archivo de seguridad caduca** (visto el 2026-11-20): distinguir
      «caducado» de «incorrecto» y decir que se pide otro al colegio. La v1
      lo copiaba a una carpeta temporal (`__file__` en PyInstaller): la v2
      lo guarda donde no se pierda.
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

### H8 · Estadísticas

- [ ] Media, mediana (bien), aprobados y distribución, con las reglas de
      gráficos de library-app (H8).
- [ ] Lo del dictado: notas por voz, aciertos a la primera, tiempo ahorrado.

### H9 · La ayuda guiada

- [ ] La primera vez, y una ayuda por pantalla, como H12 de library-app.
- [ ] **«Ayuda» en la barra lateral** (pedido por el usuario el
      2026-11-19): cómo funciona cada cosa de la app, en un sitio; el «?»
      de library-app. Se deja para aquí porque H4–H8 cambian las pantallas.

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
