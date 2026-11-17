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
| `--success` | `oklch(0.540 0.130 150)` · `#258343` | Evento confirmado | 4,6:1 |
| `--danger` | `oklch(0.520 0.180 28)` · `#BA2C25` | Error de verdad | 5,8:1 |
| `--warn` | `oklch(0.700 0.150 75)` | El punto «aquí hay algo que mirar» | (solo punto, sin texto) |

Los contrastes se calcularon con la fórmula de WCAG sobre `--bg` de «agua».

### Los neutros: tres familias, a elegir en el Muestrario

El acento es el mismo en las tres; cambian fondos, bordes y texto.

| Familia | Matiz | Carácter |
|---|---|---|
| **Agua** (por defecto) | 195 | Neutros teñidos del propio turquesa: la app entera «es» del logo |
| **Papel** | 85 | Neutros cálidos: el turquesa destaca más por ser su complementario |
| **Niebla** | 240 | Gris azulado frío: la familia de la Quality Test App |

⚠️ **Pendiente de elegir** sobre la pantalla (`npm run tauri:dev` → Muestrario).
En cuanto se elija, las otras dos se borran de `tokens.css` y el Muestrario
pierde el selector.

## 3. La regla de color

**El color marca eventos y excepciones. Nunca estados normales.** Es la de
library-app, aplicada a las notas:

| Estado de una nota | Cómo se ve | Por qué |
|---|---|---|
| Sin nota | `—` en gris (`--text-tertiary`) | Es lo normal al empezar, aunque sea la mayoría |
| Nota puesta | Texto normal, en mono, a la derecha | Es un dato, no un evento |
| **Suspenso** | **Igual que cualquier nota** | Una nota baja no es un error: pintarla de rojo gasta el único color que debe quedar para los problemas |
| Recién dictada | Verde, con el micrófono, **y se apaga sola** | Es un evento: «acaba de entrar por voz» |
| Dictado dudoso | **Borde discontinuo** + `--accent-wash`, con «¿Pérez Gómez, Irene? ✓ ✕» | Provisional: espera que lo confirmes |
| Valor que no es nota | Rojo, con anillo | Error de verdad: `12`, `abc` |

**La forma dice lo que el color no**: el borde discontinuo significa
«provisional, falta que una persona lo confirme», y **nada más lo usa**. Por
eso las pantallas vacías de H0 llevan borde continuo.

## 4. Lo que queda por decidir

No bloquea H1. Se resuelve cuando la pantalla exista:

- **La familia de neutros** (§2).
- **La celda de nota**: `<input type="number">` muestra `7,5` con la coma del
  sistema, y aquí la nota va con punto (es lo que acepta Clickedu). La celda
  de H2 será un campo de texto propio con `inputMode="decimal"`, que convierte
  la coma en punto al teclearla, como hacía la app de Python.
- **El indicador de «escuchando»** (H3a): dónde va y cómo se ve mientras se
  mantiene ESPACIO.
