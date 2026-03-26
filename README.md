# 🎙️ Gestor de Notas para Clickedu (Dictado por Voz)

[![Release](https://img.shields.io/badge/Release-v1.2.0-blue.svg)](https://github.com/MiguelFerrerF/Clickedu_voice_note_picker/releases)
[![Python](https://img.shields.io/badge/Python-3.9+-blue.svg)](https://www.python.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Una aplicación de escritorio diseñada para profesores que permite calificar alumnos de **Clickedu** mediante el uso del lenguaje natural y dictado por voz. Olvídate de buscar nombres manualmente; simplemente mantén la barra espaciadora y di la nota.

---

## ✨ Características Principales

*   **🎙️ Dictado con Push-to-Talk (PTT)**: Pulsa la barra espaciadora para hablar, suéltala para procesar.
*   **🧠 Mapeo Inteligente (Fuzzy Matching)**: El programa reconoce al alumno por su nombre aunque no lo pronuncies exactamente igual.
*   **🔢 Procesamiento de Notas**: Reconoce formatos como "siete con cinco", "ocho con dos" o "nueve y medio".
*   **⚡ Navegación Ultrarrápida**: Usa las flechas y el Enter para moverte entre celdas sin tocar el ratón.
*   **📊 Panel de Estadísticas (Gamificación)**: Cuadro de mandos analítico que mide en vivo cuánto tiempo ahorras usando la aplicación, el modo ráfaga y visualiza la curva de distribución de aprobados.
*   **🗣️ Filtro Fonético Avanzado**: Corrige errores del micrófono al natural (ej. iguala "c" y "z" o "v" y "b" para encontrar a tu alumno siempre).
*   **⚡ Zero-Latency Splash Screen**: Pantalla de arranque instantáneo sin esperas al abrir el ejecutable.
*   **📂 Exportación Automática**: Genera copias del Excel original de Clickedu con las notas inyectadas y abre la carpeta automáticamente.

---

## 🚀 Novedades en v1.2.0

*   **Motor Ultrarrápido**: Sustituido el motor original por RapidFuzz (backend en C++) multiplicando exponencialmente la velocidad de búsqueda de alumnos.
*   **Descarte Magnético de la Voz**: Se puede dictar la nota a placer ("ponle a este alumno un siete con cinco") y la Inteligencia Artificial filtrará el nombre y la nota ignorando de lleno el resto de la frase gracias a los parches de intercepción lexical.
*   **Dashboards de Rendimiento**: Gamificación e interfaz ampliada al modo Premium.

---

## 🚀 Cómo Instalar (Windows)

Si solo quieres usar la aplicación, no necesitas instalar Python:

1.  Ve a la sección de **[Releases](https://github.com/MiguelFerrerF/Clickedu_voice_note_picker/releases)**.
2.  Descarga el archivo `Instalador_GestorNotas.exe`.
3.  Ejecuta el instalador y sigue las instrucciones.
4.  ¡Listo! Tendrás un acceso directo en tu escritorio.

> [!TIP]
> Al instalar, se creará un acceso directo en el menú inicio llamado **"Abrir Carpeta de Plantillas"**. Coloca allí tus archivos Excel descargados de Clickedu para que aparezcan en el programa.

---

## 🛠️ Para Desarrolladores

Si deseas ejecutar el código fuente o modificar la aplicación:

1.  Clona el repositorio:
    ```bash
    git clone https://github.com/MiguelFerrerF/Clickedu_voice_note_picker.git
    cd Clickedu_voice_note_picker/GradingApp
    ```

2.  Instala las dependencias:
    ```bash
    pip install -r requirements.txt
    ```

3.  Lanza el programa:
    ```bash
    python main.py
    ```

---

## 📝 Capturas de Pantalla e Instrucciones

Consulta el archivo [README_USO.txt](GradingApp/README_USO.txt) para una guía detallada de comandos de voz y navegación de teclado.

---

## 👤 Autor

*   **Miguel Ferrer** - [@MiguelFerrerF](https://github.com/MiguelFerrerF)

---

## ⚖️ Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.
