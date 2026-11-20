# Notas de versión

Un fichero por versión, `vX.Y.Z.md`, como en library-app. Se escribe
**antes** de `npm run release`, que se para si el de la versión no existe o
no trae titular y cambios.

## Formato

```md
---
highlights: Una frase que resuma la versión. Sale en el diálogo de la app.
entries:
  - feat: Lo que se puede hacer ahora y antes no.
  - fix: Lo que estaba mal y ya no.
  - perf: Lo que va más rápido.
  - chore: Lo que hay que saber aunque no se vea.
---

El texto libre, en Markdown: lo que se lee en la página del release.
```

De ahí salen el titular y los cambios del diálogo de actualización, **sin
una segunda petición**: `scripts/notas-de-version.mjs` los mete en el cuerpo
del release como un bloque `RELEASE_DATA` comentado, `tauri-action` lo copia
en `latest.json` y la app lo lee (`src-tauri/src/actualizacion.rs`).
`node scripts/notas-de-version.mjs 2.0.1 --cuerpo` enseña el cuerpo tal cual
saldrá.

En el diálogo, cada tipo sale con su insignia y su color, como en
library-app (una versión nueva es un evento; nunca rojo, D6): `feat`
«Nuevo» en verde, `fix` «Arreglo» en turquesa, `perf` «Más rápido» en
ámbar, `chore` «Info» en gris. Una línea con
otro tipo **se ignora**: revisa la cuenta de entradas que imprime el script.

## Publicar una versión

1. Escribir `docs/releases/vX.Y.Z.md`.
2. `npm run release X.Y.Z` (o con `-- --dry-run` para ver lo que haría):
   sube la versión en los cuatro ficheros, hace el commit y el tag. **No
   hace push.**
3. `git push origin <rama> vX.Y.Z`. CI (`.github/workflows/release.yml`)
   audita las dependencias, compila, firma y deja un **borrador** con
   `Instalador_GestorNotas.exe`, su `.sig` y `latest.json`. Comprueba
   además que el instalador subido es el que se compiló (su SHA-256) y lo
   deja en el resumen de la ejecución.
4. Revisar el borrador y pulsar **«Publish release»**. Desde ese momento lo
   ven la app (al arrancar) y la v1.3.4, que busca ese mismo nombre.
