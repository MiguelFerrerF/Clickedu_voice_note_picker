import webview
import os
import sys
import logging
from core.api import Api

# Silenciamos los logs ruidosos de pywebview
logging.getLogger('pywebview').setLevel(logging.CRITICAL)

class WebViewApp:
    def __init__(self):
        self.window = None
        self.api = Api()

    def run(self, debug=False, start_url='login.html'):
        # Resolve the path to the HTML file
        base_path = os.path.dirname(__file__)
        html_path = os.path.join(base_path, start_url)
        
        # Check if the file exists
        if not os.path.exists(html_path):
            print(f"Error: No se encontró el archivo HTML en {html_path}")
            return

        # Compensamos la diferencia detectada en Windows (aprox 16px ancho, 39px alto)
        # para que el área de dibujo útil sea exactamente 1100x750
        self.window = webview.create_window(
            'Clickedu Pro',
            url=html_path,
            width=1116,
            height=789,
            min_size=(800, 600),
            frameless=True,
            easy_drag=True,
            background_color='#f0f0f3',
            js_api=self.api
        )

        # Conectamos la ventana a la API
        self.api.set_window(self.window)

        # Iniciamos la aplicación
        webview.start(debug=debug, gui='edgechromium')

if __name__ == "__main__":
    app = WebViewApp()
    app.run(debug=True)
