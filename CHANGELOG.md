# 📈 Registro de Cambios (Changelog)

Aquí se documentan todas las novedades, mejoras y correcciones implementadas en cada versión del Gestor de Notas para Clickedu.

---

## 🚀 Novedades en v1.3.1

*   **Optimización de Rendimiento UI y Transiciones FPS**: Se ha reestructurado profundamente el motor de dibujado del módulo de gráficos pasando a ejecutarse de forma pre-evaluada en *background* y usando caché persistente. El salto entre la pantalla de Notas y las Estadísticas ahora es inmediato (`O(1)` redraw time), eliminando por completo cualquier congelación ("flickering" y "trompicones") que sucedía al construir componentes pesados.
*   **Corrección de Renderizado CTkScrollableFrame**: Solucionado un problema originado por el gestor de layout interno de customtKinter, donde la lista de alumnos quedaba gráficamente invisibilizada bajo otras capas tras intentar regresar desde el panel de promedios al hub principal de notas.
*   **Optimizador DevOps nativo**: Se ha empaquetado y subido al repositorio principal el generador de pipelines (`build_release.py`).

---

## 🚀 Novedades en v1.3.0

*   **Integración de Actualizador Automático (OTA)**: Comunicación dinámica e invisible en segundo plano con GitHub. Al encontrar nueva versión se notifica discretamente en el propio banner del programa sin pop-ups intrusivos e instala el archivo de fondo de modo fluido.
*   **UI Dinámica Responsiva (Columnas gemelas)**: Renderizado "Smart". Si expandes tu app más allá de 1000 píxeles, tu lista infinita de alumnos se parte estéticamente en *modo de 2 columnas de ancho central*, optimizando el espacio al máximo y reduciendo el tamaño del pergamino de la clase por la mitad.
*   **Ajustes Persistentes Minimalistas**: Se ha introducido una Rueda de Ajustes global (`config.json`) que almacena de por vida el estado del 'Modo Noche / Día' y los tonos de sonido elegidos, recordando cada acción que hiciste para tu próxima apertura.
*   **Filtrado Activo en Cajón Híbrido**: Barras y listas redundantes comprimidas. Ahora un combobox asume el rol de buscador filtrando resultados en directo ante cada pulsación.
*   **Estandarización Dimensional de Barras**: Alineadas todas las anchuras y componentes primarios de la barra lateral mediante inyección en grillas simétricas para evitar asimetría visual en los comandos.
