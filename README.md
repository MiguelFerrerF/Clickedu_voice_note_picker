# Gestor de Notas

Pon las notas de tus alumnos **dictándolas por voz** o con el teclado, sobre
las plantillas de Clickedu, y súbelas a Clickedu sin pasar por el navegador.

> **Esta rama es la v2, en construcción** (Tauri 2 + React + TypeScript).
> La versión que se usa hoy es la **v1.3.4**, en Python: está en la rama
> [`main`](https://github.com/MiguelFerrerF/Clickedu_voice_note_picker/tree/main)
> y su instalador, en
> [Releases](https://github.com/MiguelFerrerF/Clickedu_voice_note_picker/releases).
> Cuando salga la v2.0.0, la v1.3.4 se actualizará sola.

## Documentación

| Documento | Qué cuenta |
|---|---|
| [`docs/01-plan-de-trabajo.md`](docs/01-plan-de-trabajo.md) | Por qué se rehace, las decisiones D1–D7 y los hitos H0–H9 |
| [`docs/02-sistema-visual.md`](docs/02-sistema-visual.md) | La paleta del logo y la regla de color |
| [`CLAUDE.md`](CLAUDE.md) | El estado del proyecto y las reglas de trabajo |

## Para desarrollar

Hace falta Node 22, Rust 1.88 o posterior y, en Windows, WebView2 (viene con
Windows 10 y 11).

```powershell
npm install
npm run tauri:dev       # la app
npm run probar:diseno   # las pantallas a cuatro tamaños de ventana
```

## Licencia

MIT — ver [`LICENSE.txt`](LICENSE.txt). La tipografía IBM Plex va incluida
bajo la SIL Open Font License 1.1 (`src/assets/fonts/LICENSE.txt`).
