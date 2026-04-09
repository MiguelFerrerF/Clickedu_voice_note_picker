import webview
import os
import sys

import logging

# Silenciamos los logs ruidosos de pywebview que causan ruido en la terminal en Windows
logging.getLogger('pywebview').setLevel(logging.CRITICAL)

class Api:
    def __init__(self):
        self._window = None
        self._is_maximized = False

    def set_window(self, window):
        self._window = window

    def minimize_window(self):
        if self._window:
            self._window.minimize()

    def toggle_maximize(self):
        if self._window:
            # Simple toggle state
            if self._is_maximized:
                self._window.restore()
                self._is_maximized = False
            else:
                self._window.maximize()
                self._is_maximized = True

    def close_window(self):
        if self._window:
            self._window.destroy()

class WebViewApp:
    def __init__(self):
        self.window = None
        self.api = Api()

    def run(self, debug=False):
        # Resolve the path to UX.html
        base_path = os.path.dirname(__file__)
        html_path = os.path.join(base_path, 'UX.html')
        
        # Check if the file exists
        if not os.path.exists(html_path):
            print(f"Error: No se encontró el archivo HTML en {html_path}")
            return

        # Create the window and pass the API instance
        self.window = webview.create_window(
            'Clickedu Pro',
            url=html_path,
            width=1100,
            height=750,
            min_size=(800, 600),
            frameless=True,
            easy_drag=True,
            background_color='#f0f0f3',
            js_api=self.api
        )

        # Connect the window back to the API instance
        self.api.set_window(self.window)

        # Start the application
        webview.start(debug=debug, gui='edgechromium')

if __name__ == "__main__":
    app = WebViewApp()
    app.run(debug=True)
