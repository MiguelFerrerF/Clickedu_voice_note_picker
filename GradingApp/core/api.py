import ctypes
from ctypes import wintypes
import logging

# Silenciamos los logs ruidosos de pywebview
logging.getLogger('pywebview').setLevel(logging.CRITICAL)

class Api:
    def __init__(self):
        self._window = None
        self._is_maximized = False

    def set_window(self, window):
        self._window = window

    def _get_work_area(self):
        """Obtiene el área de trabajo del monitor principal (excluyendo barra de tareas)."""
        try:
            user32 = ctypes.windll.user32
            rect = wintypes.RECT()
            # SPI_GETWORKAREA = 0x0030
            user32.SystemParametersInfoW(0x0030, 0, ctypes.byref(rect), 0)
            return rect.left, rect.top, rect.right, rect.bottom
        except Exception:
            # Fallback a dimensiones genéricas si falla Win32
            return 0, 0, 1920, 1080

    def snap_left(self):
        if self._window:
            left, top, right, bottom = self._get_work_area()
            width = (right - left) // 2
            height = bottom - top
            self._window.move(left, top)
            self._window.resize(width, height)
            self._is_maximized = False

    def snap_right(self):
        if self._window:
            left, top, right, bottom = self._get_work_area()
            width = (right - left) // 2
            height = bottom - top
            self._window.move(left + width, top)
            self._window.resize(width, height)
            self._is_maximized = False

    def snap_top(self):
        """Maximiza la ventana ocupando todo el área de trabajo."""
        if self._window:
            left, top, right, bottom = self._get_work_area()
            self._window.move(left, top)
            self._window.resize(right - left, bottom - top)
            self._is_maximized = True

    def snap_bottom(self):
        """Restaura al tamaño por defecto compensado o minimiza si ya está restaurada."""
        if self._window:
            if self._is_maximized:
                # Restaurar al tamaño exacto solicitado 1100x750 (compensado para Windows)
                left, top, right, bottom = self._get_work_area()
                work_width = right - left
                work_height = bottom - top
                
                new_width, new_height = 1116, 789 
                new_x = left + (work_width - new_width) // 2
                new_y = top + (work_height - new_height) // 2
                
                self._window.restore()
                self._window.move(new_x, new_y)
                self._window.resize(new_width, new_height)
                self._is_maximized = False
            else:
                self._window.minimize()

    def minimize_window(self):
        if self._window:
            self._window.minimize()

    def toggle_maximize(self):
        if self._window:
            if self._is_maximized:
                # Restaurar al tamaño por defecto centrado
                left, top, right, bottom = self._get_work_area()
                work_width = right - left
                work_height = bottom - top
                
                new_width, new_height = 1116, 789
                new_x = left + (work_width - new_width) // 2
                new_y = top + (work_height - new_height) // 2
                
                self._window.restore()
                self._window.move(new_x, new_y)
                self._window.resize(new_width, new_height)
                self._is_maximized = False
            else:
                self._window.maximize()
                self._is_maximized = True

    def close_window(self):
        if self._window:
            self._window.destroy()

    # --- FUTURAS INTEGRACIONES ---
    def login(self, username, password, security_file_path):
        """Lógica para autenticación con Clickedu."""
        print(f"Intento de login para: {username}")
        # Aquí se llamará a ClickeduClient
        return {"success": True, "user": username}

    def get_students(self):
        """Obtiene la lista de alumnos de la clase actual."""
        # Aquí se llamará a excel_handler
        return []

    def export_grades(self, data):
        """Exporta las notas a un archivo Excel."""
        # Aquí se llamará a excel_handler
        return True
