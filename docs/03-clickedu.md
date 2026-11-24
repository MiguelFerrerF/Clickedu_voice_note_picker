# Clickedu, por dentro

El mapa de la web de Clickedu del colegio para H4–H6, hecho **solo
leyendo** el 2026-11-20 con `cargo run --example explorar_clickedu` y la
cuenta del usuario. Fueron 300 páginas, todas con `GET`, una por segundo,
con dos muestras de cada forma de dirección. Lo crudo (con nombres) se
miró solo en local: el de esa pasada se borró; el de la segunda («La
segunda pasada») se guarda en
`%APPDATA%\es.gestornotas.app\clickedu\capturas\2026-11-20\`, **fuera del
repo**, para sacar de ahí, anonimizadas, las páginas de los tests de H4.
Aquí solo hay formas: `▢` es un número o un texto tapado.

Lo que **no** se abrió: todo lo que lleva un verbo que no es de leer
(`do=Save`, `do=Edit`, `accio=crear`…), cualquier palabra de escribir en
la dirección (`esborrar`, `importar`, `copiar`, `delete`…), lo AJAX, los
mensajes y las notificaciones (quedarían leídos), lo que cambia una
preferencia (`layout=`, el idioma), los datos de salud y de conducta,
`/` y `user.php` (ver «La sesión»). Ningún formulario se mandó.

> **Ojo con lo abierto**: el filtro dejó abrir `assignatures/substituir.php`
> (dos veces), que contesta con un `location` a
> `sessio.php?…&tipus=substitut`; las de pasar lista
> (`assistencia/index.php`) y las de justificar (`assignatures/assistencia.php`)
> solo enseñaron sus formularios. No parece que nada de eso apunte nada,
> pero `substituir` no se puede asegurar desde fuera. Para la próxima,
> fuera del filtro: `substitu`, `assistencia`, `sessio`, `fitxa_alumne`.

## Entrar y la sesión

1. `POST /user.php?action=doLogin` (`username`, `password`, `button`).
   Vuelve una página con `form#frmArxiuPas` → `controlArxiuPas`, con
   `MAX_FILE_SIZE`, `id_usuari` (ocultos) y `userfile`.
2. `POST /user.php?action=controlArxiuPas` (*multipart*). Si entra:
   **200, 100 bytes, solo un `location` a `/sumari/index.php`**, que es el
   inicio de verdad.
3. Cookies: `PHPSESSID` (del host; `secure`, `HttpOnly`,
   `SameSite=Strict`) es la sesión. `/sumari/index.php?p=▢` pone además
   `SID` con `Max-Age=300`. La página de entrar borra `idpSchoolIdentifier`
   (del dominio `clickedu.eu`, el de entrar con Google o Microsoft:
   `user.php?google=`, `?live=`).

- **Sin sesión no hay redirección HTTP**: cualquier página contesta **200**
  con `alert('No ha iniciado sesión')` y un `window.location` a
  `user.php?action=login`. `comprobar()` no lo veía: daba por entrado algo
  que no lo estaba. **Arreglado** (`es_pagina_de_entrar` lo reconoce, y el
  Clickedu falso de los tests contesta igual).
- **`/` cierra la sesión**: lleva a la página de entrar, y esa la tira.
  Dentro, nunca se piden `/` ni `user.php`.
- **Duración**: aguantó el recorrido entero (unos 6 minutos de peticiones
  seguidas). **Quieta** (2026-11-21, `--sesion "10,20,30,45"`, cada pausa
  sin pedir nada y una página al acabar): a los 10 min, **sigue**; a los
  20 min (35 desde que se entró), **sigue**; a los 30 (66), **sigue**; a
  los 45 (111), **sigue**. **Aguanta al menos 45 minutos quieta** y casi
  dos horas desde que se entró: entre dictar una clase y subirla no
  caduca, y si caduca, `Cliente::abrir` vuelve a entrar.
- **Caducada no se distingue de incorrecta**: la sesión caducada contesta
  lo mismo que sin entrar (el `alert`). `Cliente::abrir` lo ve
  (`es_pagina_de_entrar`), entra otra vez con lo de la última vez (solo en
  memoria, nunca en un fichero) y vuelve a pedir la página; si aun así
  sale la de entrar, es `NoEntiendo`. Probado contra el Clickedu falso
  con una sesión que caduca a mitad.

## La navegación

El menú (en todas las páginas): inicio (`/sumari/index.php`), materias
(`go/Index.php?ctr=Subjects/Subjects`), calendario, archivos
(`/files/`), contenidos, gestión (`/gestio/`), comunicación, chat,
institución. Lo de profesor cuelga de `/assignatures/*.php` (lo antiguo,
PHP suelto: alumnos, evaluaciones, evaluar por alumno, horario,
sustituciones…) y de `go/Index.php?ctr=…` (lo nuevo, con controladores:
`Subject/…`, `Evaluate/…`, `GradesTargets`, `Rubrics`…). Conviven los
dos: la importación de notas sigue en lo antiguo.

## Materias

`ctr=Subjects/Subjects`: **lo de la v1 sigue valiendo**. 10
`div.bloc-assignatura`, cada uno con `div.title` (la materia) y
`div.sub-title` (curso y grupo). De cada materia salen
`ctr=Subject/Subject&do=Index&id=N` (su página),
`ctr=Evaluate/EvaluateTermly&id=N` («Evaluar») y
`assignatures/sessions.php?assig=N`: **el mismo `N` es `id` en lo nuevo y
`assig` en lo antiguo**, como en la v1.

- **Los cursos** («Tercero de ESO», «Primero de Bachillerato»…) son
  encabezados `div.panel-separador` entre los bloques: cada materia es del
  último que hay antes. El `sub-title` lleva curso y grupo
  («3º E.S.O.-A»).
- La misma materia en dos grupos son dos materias con dos `id`. Hay
  también **pendientes** de cursos anteriores («‹materia› 1º -
  PENDIENTE…») y de Bachillerato: H5 tiene que enseñarlas tal cual o
  dejarlas elegir.
- No hay selector de curso académico en la página de materias. Lo que sí
  hay es `&layout=▢`, que **cambia una preferencia**: no se toca.

## Evaluaciones e ítems

`ctr=Subject/Items&do=Index&id=N[&trm=T]` («Ítems»):

- Un formulario `GET` con `<select name="trm">`: `all` («Todas») y las
  tres evaluaciones con sus fechas: «1ª evaluación (de 04/09/2026 a
  20/11/2026)». **Con las fechas, la app puede proponer la evaluación de
  hoy.** Los `trm` son números.
- La tabla de ítems: «Ponderación | ▢ | Nombre | Opciones | Acciones», un
  ítem por fila, con `ctr=Subject/Item/Item&do=Edit&id=N&it=I&trm=T`.
- **Desmiente la v1**: ya no hay `a.popupEvent` con `do=See`, que es de
  donde la v1 sacaba los ítems. La v1, hoy, no encontraría ninguno.

**Lo mejor para H6 es la página de importar**, que trae los ítems ya
listos: su `<select name="id_criteri_avaluacio">` tiene `---` y un
`<option>` por ítem de esa evaluación, con su id como valor y su nombre
como texto. Es justo lo que se elige al subir.

En otras páginas, las evaluaciones llevan letra: `trm=331p` en
`Evaluate/EvaluatePreTermly`, `aval=333f` y `aval=321e` en
`alumnes_assignatura.php` (parece: `p` preevaluación, `f` final, `e`
extraordinaria). No hacen falta para importar.

## Importar notas

`/assignatures/importar_notes_items.php?assig=N&id_aval=T` («Importar
notas: ‹materia›»), enlazada desde «Evaluar». **Como en la v1:**

- «Descargar plantilla»: `accio=descarregar_plantilla&assig=N&id_aval=T`.
- El formulario: `POST …?accio=carregar_plantilla&assig=N&id_aval=T`,
  *multipart*, con `id_criteri_avaluacio` (el ítem), `v_arxiu_importar`
  (el fichero) y `BotoGenerar`.
- **La acción del formulario es relativa** (`importar_notes_items.php?accio=…`,
  sin `/assignatures/`): `fichas::importar` la devuelve tal cual y quien
  sube la resuelve.

**Subir no es importar** (la primera subida, 2026-11-23, `cargo run
--example probar_subir`, en el ítem de prueba de Química 2; abajo, «Qué
cambia del plan»). Son tres pasos, y los tres escriben:

1. **Subir**: el `POST` de `carregar_plantilla` responde con
   `alert("Archivo subido correctamente.")` y el fichero sale en la tabla
   «Importación de datos» (Núm., Archivo, Ítem evaluativo, fecha y usuario
   de creación y de ejecución, Acciones; vacía, «No hay datos»). El nombre
   lo cambia Clickedu (`notas_prueba_42700xlsx`, sin el punto). **Las
   notas aún no están**: `GetGrades` no cambia.
2. **Importar**: el enlace `accio=importar_notes&assig=N&id_aval=T&id=S`
   de esa fila (`S`, el número del subido; un `GET`) responde con
   `alert("Importación correcta")`. La fila gana fecha y usuario de
   ejecución y **pierde el enlace de importar**: quedan `accio=veure`,
   «Descargar» y «Eliminar». Las notas, **en `nota_mod`** (10 de 10
   iguales, `nota` nula, `type` 0); el resto de `GetGrades`, idéntico.
   **Importar reemplaza el ítem entero** (2026-11-24, con la app en el
   42700): quien va sin nota en el Excel **pierde la que tenía** (de 10,
   quedaron las 3 subidas). Por eso la app pone en el Excel la nota de
   Clickedu de quien no tiene en la app (`subir::completar`); la que no
   es un número (un «NP») no cabe y se avisa antes de que se borre.
3. **Eliminar** no es un enlace: es `<a class="BotoEsborrarImportacio"
   cgap="N" aval="T" ind="S">`, y el JS, tras un `confirm`, va a
   `accio=esborrar_arxiu&assig=N&id_aval=T&id=S` (`GET`):
   `alert("Eliminado correctamente")`. **Borrar lo subido no quita las
   notas importadas**: siguen en `nota_mod`. Para quitarlas, hay que
   borrar el ítem o ponerlas a mano.

`accio=veure` no se ha abierto (puede enseñar nombres). La respuesta de
cada paso es un `alert` en un `<script>`: es lo que hay que leer para
saber si fue bien.

**La plantilla** (`.xlsx`, 7 KB): hoja «Worksheet», cabecera en la fila 1
«Id | Alumnos | Nota», 30 alumnos, ids solo cifras, nombres «Apellidos,
Nombre». **`plantillas.rs` la lee tal cual** (`revisar_plantillas`).

- **No depende de la evaluación** (2026-11-22, `comparar_plantillas`, las
  10 materias): con cualquier `id_aval` de la materia sale la misma,
  celda a celda, y sin notas. El fichero se llama
  `evaluate_‹AAAAMMDD›_‹HHMMSS›.xlsx` (la hora de bajarla).
- **Sin alumnos, sale vacía**: una pendiente sin nadie da un `.xlsx` con
  la hoja «Worksheet» y ninguna celda, ni la cabecera.
- Los `id_aval` son **de cada curso** (los de 3º ESO no son los de 4º; 1º
  Bach tiene una cuarta, «JUNIO»).

## Leer las notas ya puestas

`ctr=Evaluate/EvaluateTermly&id=N&trm=T` es **la cuadrícula de notas**
(450 KB): `table.sticky-box-holder`, una fila por alumno, una columna
por ítem (con el nombre corto que le puso el profesor) y «Nota final 1ª
evaluación». Las notas
son `<input class="InputGrades" name="evalObject_▢_▢_▢">` (330: 30
alumnos × 11), con `evalObjectCalc_…` (las calculadas),
`evalObjectQuantifier_…` (un `<select>`: «---», «No ▢», «En ▢»…) y
`associatedText_…` (comentarios).

- **`evalObject_A_B_C`** (segunda pasada, 2026-11-20, cruzando solo
  recuentos de ids en 8 cuadrículas): **A es el alumno** (los 30 son los
  `Id` de la plantilla y los `alum=` de los enlaces); **B es el ítem**
  (los `it=` y las opciones de `id_criteri_avaluacio`), más uno que es el
  `id` de la materia: la «Nota final»; C tiene dos valores (uno solo
  donde solo está la nota final) y no es ni la evaluación ni la materia:
  parece el tipo (ítem o final), como el `type` de `GetGrades`.
- **El HTML llega sin notas** (0 de 360 campos con `value`): las pone el
  JS después, con **`do=GetGrades&id=N&trm=T`**, que lee y devuelve JSON:
  una lista de `{id_alumne, id_criteri, nota, nota_mod, id_quantifier,
  text, type}` (266 en una 1ª evaluación; en otra, 30 con `id_avaluacio`
  y `nota` como texto). Los mismos ids que la plantilla y el `<select>`
  de ítems. **Para que H6 compruebe que entró: `GetGrades`**, filtrando
  `id_criteri` = el ítem y cruzando `id_alumne` con el `Id` de la
  plantilla. Es JSON con nombres de campo: más resistente que el HTML.
  **`type`** dice qué es cada fila: `0`, la nota de un ítem; `114`, la
  nota final de la evaluación (con el id de la materia como `id_criteri`);
  la otra forma trae `120`, y en Química 2 sale también `112`. **Las
  notas de los ítems van sobre todo en `nota_mod`** (177 filas, frente a
  30 con `nota`; en la final, las dos): `nota` es la calculada y
  `nota_mod` la puesta, **confirmado con la primera subida** (H6,
  2026-11-23, arriba). Contado en las páginas anonimizadas, sin nombres.
- `do=GetInfoEvalObjects` (también JSON): `{id_alumne, quantitat_ok,
  type}` por alumno; no hace falta.
- La misma página es un formulario `POST …do=Save` (`#FormEvaluate`):
  **otra forma de poner notas**, sin Excel. Más frágil y más peligrosa
  (manda la cuadrícula entera): no es el plan, pero conviene saberlo.

## Dar de alta un ítem

Mirado solo leyendo el 2026-11-23 (`cargo run --example mirar_alta_item
-- --materia N`, en Química 2), para el ítem de prueba de H6. **La app no
da de alta ítems**: sube a los que ya hay. Esto es para saber qué toca
cada cosa.

- «Nuevo ítem» es `ctr=Subject/Item/ItemDefault&do=New&id=N` (`GET`, el
  formulario). Se guarda con `POST …ItemDefault&do=Save&id=N&it=&trm=`
  (`#formulari_items`, sin fichero): `v_nom` (Nombre), `v_codi_criteri`
  (Código), `i_ordre`, `id_avaluacio` (la evaluación, el `trm`),
  `tipus_avaluacio[]` (`1` nota numérica, `2` cuantificadores),
  `ind_visible_pm` («Visible a padres»), `ind_text_associat`,
  `tipus_item` (`0` ítem, `1` evaluación suspendida, `2` ponderada) y los
  de asistencia. Editar es lo mismo con `it=` y `trm=`.
- **El alta no tiene ponderación.** Se pone en la tabla de Ítems
  (`percent_‹it›_‹it›_0`), que la guarda por AJAX con
  `Evaluate/EvaluateTermly&do=SavePercentage`. Un ítem con la ponderación
  vacía es posible (había uno, «Recuperación…»).
- Las casillas de la tabla de Ítems (`CheckEdicioRapida`) **escriben al
  marcarlas**: `ItemDefault&do=ChangeVisiblePm`, `ChangeTextAssociat`…
  (`POST` con `item` y `valor`).
- Borrar: `Subject/Item/Item&do=Delete&id=N&it=I` es un **`GET`** (con un
  `confirm` en JS); en la edición, «Forzar eliminación» va por
  `Academic/AssesmentsItemsDeleteAjax&do=delete`. Borrar un ítem borra sus
  notas.
- Al elegir la evaluación, el formulario pide por `POST` los ítems de los
  que puede colgar (`do=StructureItems`, `itemPare[]`); en este colegio no
  es obligatorio (`config_crear_items = 1`): sin elegir, cuelga de la nota
  final, como los demás.

## Idioma y textos

- La interfaz sale **en castellano** (hay un `selector_idioma` es/en, que
  es un formulario y cambia la preferencia: no se toca).
- **ISO-8859-15**: 270 de 288 páginas no son UTF-8 (15 sí), y la
  cabecera no lo dice; lo dice el pie de la página. Al leer nombres de
  materias e ítems del HTML hay que decodificar bien, o «Evaluación» sale
  rota (`cliente::decodificar`). El Excel no tiene ese problema.
- **Lo resistente**: los parámetros (`ctr`, `do`, `id`, `assig`,
  `id_aval`, `trm`, `it`), los nombres de los campos
  (`id_criteri_avaluacio`, `v_arxiu_importar`, `evalObject_`) y de los
  `<select>`. **Lo frágil**: los textos («Importar notas», «Archivo de
  seguridad INCORRECTO», el `alert` sin sesión) y las clases CSS
  (`bloc-assignatura`, `InputGrades`).

## Lo demás que puede servir

- Alumnos por grupo: `assignatures/alumnes.php`,
  `excel_llista_alu_cgap.php?cgap=N` (un CSV servido como Excel) y la
  propia plantilla.
- `ctr=Subject/Items/ItemsWsdlGradesConfiguration`: elige qué nota
  («nota media»…) se manda **por WSDL**. Hay un servicio SOAP de notas
  para integraciones, pero no es para nosotros (no nos dan credenciales).
- Calendario (`/calendari/`, `export.php?view=day`), horario
  (`Subject/Timetable`), asistencia y fichas de alumnos: no hacen falta.

## Qué cambia del plan de H4–H6

1. **`comprobar()`**, arreglado (el aviso por JS). La «prueba con
   Clickedu de verdad» de antes pudo darse por buena en falso; la
   exploración confirma que se entra (la página de materias llega
   entera).
2. **Dentro, nunca `/` ni `user.php`**; el inicio es `/sumari/index.php`.
3. **H5**: materias de `Subjects/Subjects` (como la v1); evaluaciones del
   `<select name="trm">` de Ítems, con fechas; la plantilla, como la v1.
4. **H6**: los ítems, del `<select>` de la página de importar (no de los
   *popups* de la v1, que ya no existen); subir como la v1; **el paso de
   después (la lista de subidos e importar) está sin ver**. Comprobar, con
   `GetGrades` (JSON), no con el HTML de la cuadrícula, que llega vacío.
   `id_aval` es el `trm` de Ítems (los dos de la pasada están entre sus
   opciones), y el `Id` de la plantilla, el alumno de todo lo demás.
5. **ISO-8859-15** al leer el HTML.

**La segunda pasada** (2026-11-20, 27 páginas, con `--solo` para no
salir de materias, Ítems, `EvaluateTermly`, importar y `GetGrades`, y
`--permitir getgrades,getinfoevalobjects`; fuera del filtro ya
`substitu`, `assistenc`, `sessio`, `fitxa`): `evalObject_` y `GetGrades`,
arriba.

**Mirado en la primera subida** (H6, 2026-11-23, con el usuario): la
página de importar con un fichero subido, y que la nota va en
`nota_mod` (arriba, «Importar notas»). **Dónde** (2026-11-20): desde la cuenta no se crean
materias, solo ítems («Nuevo ítem», `Subject/Item/ItemDefault&do=New`, y
`do=Delete`). Se hace en **Química 2** (2026-11-23), en un ítem «PRUEBA
Gestor de Notas - borrar» de la 2ª evaluación, sin ponderación y sin
«Visible a padres», **creado a mano** en el navegador (arriba, «Dar de
alta un ítem»).

## Cuando Clickedu cambie

La capa resistente (`docs/01`, H4) está en `src-tauri/src/clickedu/`:
**las fichas** (`fichas.rs`: cada dato, cómo se busca y qué tiene que
cumplir; bien, degradado o roto) y **la huella** (`huella.rs`: la forma de
la página, sin nombres, comparada con la última buena, en
`%APPDATA%\es.gestornotas.app\clickedu\huellas\`). Una página que no se
entiende se guarda entera en local (`clickedu\paginas\`). Para arreglarlo:

1. **Capturar la página nueva**, solo leyendo:
   `cargo run --example explorar_clickedu -- --solo "<trozo de la dirección>"`.
   Lo crudo (con nombres) queda en `clickedu\exploracion\`: se mira solo en
   local y se mueve a `clickedu\capturas\<fecha>\`, **nunca al repo**.
2. **Anonimizarla** para los tests:
   `cargo run --example anonimizar_clickedu -- <carpeta de la captura>`
   escribe en `src-tauri/tests/datos/clickedu/` solo lo que pasa la lista
   blanca (rótulos, estructura, ids inventados) y no escribe nada si queda
   una palabra de un nombre de las plantillas. **Se mira antes del
   commit**: el repo es público.
3. **Arreglar la ficha** hasta que su test pase con la página nueva (y
   siga pasando con la vieja, si Clickedu sirve las dos), y publicar.

Las páginas de los tests (2026-11-21, de la segunda pasada): `materias`,
`items` (con las evaluaciones), `importar` y dos de `GetGrades`
(`notas-1`, `notas-2`).

**Qué lee cada ficha** (2026-11-21) y cuándo deja de leer bien:

| Ficha | De dónde | Bien | Degradado | Roto |
|---|---|---|---|---|
| `materias` | `div.bloc-assignatura` (`.title`, `.sub-title`) y el `div.panel-separador` de antes (el curso) | por los bloques, con el mismo id en `Subject/…&id=`, «Evaluar» (`id=`) y sesiones (`assig=`) | los bloques no están o no cuadran (dos ids distintos), pero sí los enlaces `ctr=Subject/Subject&id=`: el nombre, el texto del enlace; sin curso | ni bloques ni enlaces que cuadren |
| `evaluaciones` | las opciones numéricas de `select[name="trm"]` de Ítems, con las fechas `d/m/aaaa` del texto | fechas en todas y seguidas | alguna sin fechas o no seguidas: no se propone la de hoy | sin el `<select>`, sin opciones o una sin nombre |
| `items` | `select[name="id_criteri_avaluacio"]` de la página de importar (sin el `---`) | ids numéricos, con nombre, sin repetir | — | sin el `<select>`, un id no numérico, sin nombre o repetido |
| `importar` | el `form` de la página de importar | `POST`, *multipart*, `accio=carregar_plantilla` con el mismo `assig` e `id_aval`, `v_arxiu_importar` (fichero) e `id_criteri_avaluacio` | **no hay**: para subir notas no vale «más o menos» | falta el formulario o cualquiera de esas cosas |
| `notas` | el JSON de `GetGrades` | una lista de filas con `id_alumne` e `id_criteri` numéricos y `nota` o `nota_mod` | — | no es JSON ni lista, o a una fila le falta eso |

**Cómo se anonimiza** (`anonimizar_clickedu`, seguro por construcción):
fuera scripts, estilos, comentarios e imágenes; solo los atributos de una
lista blanca; los textos pasan por `Tapar` (quedan los rótulos de la
interfaz, las palabras cortas y los números de hasta 4 cifras; lo demás,
`▢`); los ids de más de 100 se cambian por inventados que empiezan por 9,
con las mismas cifras y los mismos en todas las páginas (así siguen
cuadrando `assig`, `id_aval`, `trm`, `it`); las notas del JSON, inventadas.
Al final se comprueba con los nombres de las plantillas (textos, valores
de atributos y comentarios, no las etiquetas: un alumno «Li» casaba con
`<li>`). Lo que se escapó en la primera prueba y ya no: **los teléfonos
del colegio** (ahora, número de 3 cifras o más que no sea un año → `▢`) y
la ponderación «100» cambiada por un id (los ≤ 100 se quedan), y la
inicial «Mª» del nombre del profesor en la cabecera (ahora `#nomUsuari`
se tapa entero, `SIN_TEXTO`). Lo legible que queda son rótulos de la
interfaz, materias y cursos.
