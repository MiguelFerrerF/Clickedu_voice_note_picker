# Clickedu, por dentro

El mapa de la web de Clickedu del colegio para H4–H6, hecho **solo
leyendo** el 2026-11-20 con `cargo run --example explorar_clickedu` y la
cuenta del usuario. Fueron 300 páginas, todas con `GET`, una por segundo,
con dos muestras de cada forma de dirección. Lo crudo (con nombres) se
miró solo en local y se borró; aquí solo hay formas: `▢` es un número o
un texto tapado.

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
  seguidas). Sin medir: cuánto dura parada (`--sesion 60`).

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
- `accio=esborrar_arxiu&assig=…` sale solo en el JS. **No había ficheros
  subidos**, así que la lista de subidos y su enlace `accio=importar_notes`
  (el paso que importa de verdad en la v1) **no se han visto**: H6 los
  tendrá que mirar la primera vez, a mano y con el usuario.

**La plantilla** (`.xlsx`, 7 KB): hoja «Worksheet», cabecera en la fila 1
«Id | Alumnos | Nota», 30 alumnos, ids solo cifras, nombres «Apellidos,
Nombre». **`plantillas.rs` la lee tal cual** (`revisar_plantillas`).

## Leer las notas ya puestas

`ctr=Evaluate/EvaluateTermly&id=N&trm=T` es **la cuadrícula de notas**
(450 KB): `table.sticky-box-holder`, una fila por alumno, una columna
por ítem (con el nombre corto que le puso el profesor) y «Nota final 1ª
evaluación». Las notas
son `<input class="InputGrades" name="evalObject_▢_▢_▢">` (330: 30
alumnos × 11), con `evalObjectCalc_…` (las calculadas),
`evalObjectQuantifier_…` (un `<select>`: «---», «No ▢», «En ▢»…) y
`associatedText_…` (comentarios).

- Para que H6 compruebe que entró: leer aquí el `value` de cada
  `evalObject_`. Falta saber qué es cada `▢` del nombre (alumno, ítem…):
  se mira en local, con un alumno y solo números.
- Por JS hay `do=GetGrades&id=N&trm=T` y `do=GetInfoEvalObjects`, que no
  se abrieron (no son verbos de la lista). Por el nombre, leen: se pueden
  probar con `--permitir getgrades`.
- La misma página es un formulario `POST …do=Save` (`#FormEvaluate`):
  **otra forma de poner notas**, sin Excel. Más frágil y más peligrosa
  (manda la cuadrícula entera): no es el plan, pero conviene saberlo.

## Idioma y textos

- La interfaz sale **en castellano** (hay un `selector_idioma` es/en, que
  es un formulario y cambia la preferencia: no se toca).
- **Latin-1**: 270 de 288 páginas no son UTF-8 (15 sí). Al leer nombres de
  materias e ítems del HTML hay que decodificar bien, o «Evaluación» sale
  rota. El Excel no tiene ese problema.
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
   después (la lista de subidos e importar) está sin ver**. Comprobar, en
   la cuadrícula de `EvaluateTermly`.
5. **Latin-1** al leer el HTML.

**Sin mirar todavía**, para decidir con el usuario: cuánto dura la sesión
parada (`--sesion 60`), la forma de `GetGrades`, qué es cada número de
`evalObject_▢_▢_▢`, y la página de importar con un fichero subido.
