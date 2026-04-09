"""
main.py
-------
Punto de entrada de la aplicación Clickedu Pro.
"""

from gui.app_window import WebViewApp

if __name__ == "__main__":
    app = WebViewApp()
    # Inicia la aplicación.
    app.run(debug=False)
