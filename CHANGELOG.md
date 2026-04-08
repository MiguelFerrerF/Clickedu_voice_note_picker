# 📈 Registro de Cambios (Changelog)

Aquí se documentan todas las novedades, mejoras y correcciones implementadas en cada versión del Gestor de Notas para Clickedu.

## 🚀 Novedades en v1.4.0

*   **Integración Nativa con Clickedu (Login Seguro)**: Se ha implementado un sistema de autenticación robusto que permite conectar la aplicación directamente con tu cuenta de Clickedu mediante usuario, contraseña y archivo de seguridad.
*   **Asistente de Subida Automatizada**: Nueva funcionalidad para enviar las notas dictadas directamente a la plataforma Clickedu. El programa permite seleccionar dinámicamente la asignatura, el periodo de evaluación y el ítem evaluativo de destino, realizando el volcado de datos automáticamente y evitando el proceso manual de importación en el navegador.
*   **Identificación Dinámica de Usuario**: Mejorado el motor de conexión para extraer el ID de usuario directamente de Clickedu. Ya no es necesario que el archivo de seguridad mantenga un nombre específico; el sistema lo identifica por su contenido.
*   **Refinado Estético de Ventanas**: Se han actualizado los iconos y el diseño de los mensajes de advertencia en las ventanas modales para una experiencia más limpia y profesional.

---

## 🚀 Novedades en v1.3.4

*   **Hotfix del Micrófono (Asincronía Real)**: Corregidos múltiples errores internos (`AttributeError`) causados por una desincronización en los eventos del micrófono. Se ha rediseñado por completo el flujo de grabación y procesamiento: la lectura de voz desde Google y el algoritmo de "matching" ultrarrápido ahora suceden exclusivamente en segundo plano, devolviendo instantáneamente el control al programa mientras terminan de calcularse. Se acabó la congelación visual mientras dictas notas.

---

## 🚀 Novedades en v1.3.3

*   **Pantalla de Carga (Splash Screen) Mejorada**: Se ha parcheado la forma en que la aplicación arranca en Windows. Ahora el icono de inicio durante la fase de descompresión (Splash) se dibuja sobre un bloque de textura oscura y sólida, solventando un *bug nativo de PyInstaller* que generaba un contorno magenta o morado al chocar con transparencias (halo de colorkeying en fondos Alpha). Con esto, la aplicación luce mucho más natural y estéticamente superior mientras se abre.

---

## 🚀 Novedades en v1.3.2

*   **Soporte de Doble Decimal**: Se ha ampliado el rango de precisión permitiendo introducir hasta dos decimales en las notas (ej. `8.45`).
*   **Estandarización de Separador (Punto)**: En cumplimiento con los estándares internacionales y de Clickedu, la aplicación ahora utiliza exclusivamente el punto (`.`) como separador decimal:
    *   En la interfaz, si pulsas la coma (`,`), se convierte automáticamente en punto al instante.
    *   En el Excel exportado, las notas se guardan con formato de punto decimal y se fuerzan como texto para evitar re-conversiones automáticas de Excel.
*   **Nueva Identidad Visual**: Se ha actualizado el icono oficial de la aplicación para una estética más moderna y profesional.
*   **Corrección Crítica de Navegación**: Solucionado el error que provocaba que la lista de alumnos apareciera vacía al cambiar entre diferentes archivos de clase en la misma sesión.
*   **Limpieza de Datos al Cargar**: El cargador de archivos ahora limpia automáticamente cualquier residuo de formato (espacios, comas) en las notas existentes para asegurar la compatibilidad total.

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
