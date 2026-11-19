# 02 · Sistema visual

Decidido el **2026-11-17** (D5 y D6 de [`01-plan-de-trabajo.md`](01-plan-de-trabajo.md)).

La regla del proyecto: **`C:\dev\library-app` es el sistema de diseño** y
tiene precedencia. Antes de inventar un token, un componente o una carpeta, se
mira cómo lo resuelve ella. Este documento dice qué se hereda y qué es propio.

---

## 1. Qué se hereda

| Qué | Cómo |
|---|---|
| Tipografías | IBM Plex Sans (interfaz) · IBM Plex Mono (notas, cifras, códigos), **empaquetadas** |
| Espacio de color | `oklch` para todo |
| Nombres de token | Los mismos, uno a uno, para que los componentes se copien sin tocarlos |
| Los seis propios de library-app | `--accent-ink`, `--accent-wash`, `--bg-sunk`, `--border-strong`, `--nav-w`, `--row-h` — **y ni uno más** |
| Botones | Tratamiento «plano»: sin halo, borde = acento al 78 %, hover oscurece. Reglas duplicadas en `buttons.module.css` y en `globals.css` (`.btn-*`): **si se toca una, se toca la otra** |
| Campos | Tratamiento «papel»: `--bg-card` sobre la página, anillo de `--accent-wash` al enfocar, hover hacia el acento, flecha propia en el `<select>` |
| Tema | **Solo claro**, todo en `:root`, sin `data-theme` ni `ThemeToggle` |
| Primitivas | `Button`, `Card`, `Select`, `TextInput`/`NumInput`, `Dropdown`, `Icon`, `Progreso`, `Modal`, `Sidebar`/`SidebarSection`/`NavItem` |

### Lo que cambia respecto a library-app

- **Tipografía con `latin-ext`.** Allí bastaba `latin` (se contaron los
  caracteres de las fichas). Aquí se escriben nombres de alumnos, y Ștefan,
  Łukasz o Dvořák son normales en un aula. Con `unicode-range`, `latin-ext`
  solo se carga si hace falta. ~162 KB en total.
- **Logo en la barra lateral.** Allí no iba (el de la referencia es la marca de
  inBiot); aquí el logo es del producto y es de donde sale el color.
- **`Cabecera`** como componente: la cabecera de pantalla de library-app (22 px,
  semibold), compartida en vez de repetida en cada pantalla.

## 2. El color: el turquesa del logo

El logo es `#0F9096` = `oklch(0.592 0.100 195)`, y **tal cual no sirve de
fondo de botón**: con texto blanco da **3,9:1**, por debajo del 4,5:1 que pide
WCAG AA para texto normal. Se oscurece lo justo, sin cambiar el tono:

| Token | Valor | Para qué | Contraste |
|---|---|---|---|
| `--accent` | `oklch(0.540 0.095 197)` · `#007F82` | Fondo de botón, barra activa, progreso | blanco encima **4,8:1** |
| `--accent-ink` | `oklch(0.455 0.085 200)` · `#006469` | Texto e iconos turquesa sobre el fondo | sobre fondo **6,6:1** |
| `--accent-soft` | `oklch(0.760 0.075 195)` | Rellenos suaves, gráficos | — |
| `--accent-wash` | `--accent` al 14 % | Selección, foco, el dictado dudoso | — |
| `--success` | `oklch(0.540 0.130 150)` · `#258343` | Evento confirmado | 4,5:1 |
| `--danger` | `oklch(0.520 0.180 28)` · `#BA2C25` | Error de verdad | 5,7:1 |
| `--warn` | `oklch(0.700 0.150 75)` | El punto «aquí hay algo que mirar» | (solo punto, sin texto) |

Los contrastes se calcularon con la fórmula de WCAG sobre `--bg` (`#F6FAFD`).
El verde queda en **4,5:1**, justo en el mínimo: no se puede aclarar más.

### Los neutros: «niebla»

Elegidos sobre pantalla en el Muestrario el **2026-11-18**. Gris azulado frío
(matiz 240 en fondos y bordes, 250 en el texto), la familia de la Quality Test
App. El turquesa es el único color con croma de la pantalla, así que destaca
sin tener que subirlo.

| Token | Valor | Contraste |
|---|---|---|
| `--bg` | `oklch(0.982 0.006 240)` · `#F6FAFD` | — |
| `--bg-card` | `oklch(0.998 0.002 240)` · `#FDFFFF` | — |
| `--text-primary` | `oklch(0.250 0.018 250)` · `#1B222A` | **15,2:1** |
| `--text-secondary` | `oklch(0.470 0.018 250)` · `#535C65` | 6,5:1 |
| `--text-tertiary` | `oklch(0.620 0.014 250)` · `#80878E` | 3,5:1 (solo rótulos y `—`) |

Se descartaron «agua» (neutros teñidos del propio turquesa, matiz 195) y
«papel» (cálidos, matiz 85).

## 3. La regla de color

**El color marca eventos y excepciones. Nunca estados normales.** Es la de
library-app, aplicada a las notas:

| Estado de una nota | Cómo se ve | Por qué |
|---|---|---|
| Sin nota | `—` en gris (`--text-tertiary`) | Es lo normal al empezar, aunque sea la mayoría |
| Nota puesta | Texto normal, en mono, a la derecha | Es un dato, no un evento |
| **Suspenso** | **Igual que cualquier nota** | Una nota baja no es un error: pintarla de rojo gasta el único color que debe quedar para los problemas |
| Recién dictada | Verde, con el micrófono, y su fila teñida de verde (en `oklab`: en `oklch` sale azul), **y se apaga sola** | Es un evento: «acaba de entrar por voz» |
| Dictado dudoso | La nota no cambia; al lado del nombre, la propuesta con **borde discontinuo** + `--accent-wash` («¿6.8?»), y la pregunta en el panel del dictado | Provisional: espera que elijas |
| Valor que no es nota | Rojo, con anillo | Error de verdad: `12`, `abc` |

**La forma dice lo que el color no**: el borde discontinuo significa
«provisional, falta que una persona lo confirme», y **nada más lo usa**. Por
eso las pantallas vacías de H0 llevan borde continuo.

### La cuadrícula de alumnos, compacta (2026-11-19)

Para ver la clase entera sin bajar ni usar el ratón:

- **Dos columnas** desde 760 px de ancho de la cuadrícula (a 1100 ya
  caben), en el orden de la lista: la primera mitad a la izquierda. Enter y
  ↓ siguen al siguiente, también de una columna a otra. Con menos ancho,
  una.
- **Filas de 32 px** (`--row-h` es 36) y el campo de nota de 26. La letra
  no cambia: 13 px el nombre, mono la nota.
- **Sin nota, sin caja**: solo el `—` gris. La caja vuelve al pasar por la
  fila (que se oscurece a `--bg-sunk`) o al escribir. Con 30 cajas vacías,
  la lista era ruido.

A 1280×800, una clase de 26 se ve entera con los avisos de la plantilla
encima.

### Lo apuntado por voz, en el panel del dictado (2026-11-19)

Es lo que se lee de un vistazo mientras se dicta, así que va grande: la
nota en mono a 18 px (en verde mientras está encendida) y el nombre a 16.
Debajo, en 12 y gris, lo que se oyó. **Las dos de antes**, a 13 y en
`--text-secondary`, cada una con su «Deshacer» pequeño: dictando seguido, la
de antes no se va sin comprobarla. Solo tres: el panel va pegado arriba y,
con más, tapa la lista.

**Las dudas**, una fila por alumno con su nombre y sus notas como botones
cortos (si es un solo alumno, el nombre va en la pregunta y no se repite).
Cada nota lleva su tecla dibujada, 1–4, y «Descartar», Esc: se contesta sin
el ratón. Las teclas no valen escribiendo en un campo, donde una cifra es
parte de la nota.

## 4. Lo que queda por decidir

No bloquea H1. Se resuelve cuando la pantalla exista:

- **La celda de nota**: `<input type="number">` muestra `7,5` con la coma del
  sistema, y aquí la nota va con punto (es lo que acepta Clickedu). La celda
  de H2 será un campo de texto propio con `inputMode="decimal"`, que convierte
  la coma en punto al teclearla, como hacía la app de Python.
- ~~**El indicador de «escuchando»** (H3a)~~. Resuelto el 2026-11-19: la
  primera línea del **panel del dictado**, que en reposo dice cómo se
  dicta (en `--text-secondary`, que se lee; ESPACIO, dibujado como una
  tecla con `<kbd>`); mientras se mantiene ESPACIO, «Escuchando…» en `--accent-ink`
  con el micrófono latiendo (turquesa: algo está pasando, no es todavía un
  resultado, que sería verde). Sin fondo ni borde: el discontinuo es solo
  para lo provisional.
