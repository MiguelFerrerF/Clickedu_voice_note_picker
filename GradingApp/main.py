"""
main.py
-------
Punto de entrada de la aplicación Clickedu Pro.
"""

from gui.app_window import WebViewApp

if __name__ == "__main__":
    app = WebViewApp()
    # Inicia la aplicación con la pantalla de login por defecto.
    app.run(debug=True, start_url='login.html')
