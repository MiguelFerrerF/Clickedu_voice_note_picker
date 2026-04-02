import webview
import os
import sys
import json
import threading
import time
from core.excel_manager import ExcelManager
from core.voice_processor import VoiceProcessor

class Api:
    def __init__(self, window, app):
        self.window = window
        self.app = app
        self.excel_manager = app.excel_manager
        self.voice_processor = app.voice_processor

    def init_app(self):
        """Called by JS when pywebviewready fires."""
        self.app.sync_state_to_js()

    def select_class(self, class_name):
        """Loads a class and syncs state."""
        self.app.load_class(class_name)

    def update_grade(self, student_id, grade):
        """Updates a grade manually (if we add that feature to JS later)."""
        self.excel_manager.update_grade(student_id, grade)
        self.app.stats['grading_timestamps'].append(time.time())

    def start_recording(self):
        """Starts voice recording."""
        if not self.excel_manager.current_file:
            return
        self.voice_processor.start_recording()

    def stop_recording(self):
        """Stops voice recording and processes audio in a background thread."""
        if not self.voice_processor.is_recording:
            return
            
        def _process():
            # Notificar al JS que estamos procesando
            self.window.evaluate_js("showToast('Procesando audio...', 'info')")
            
            result = self.voice_processor.stop_recording_and_process(self.excel_manager.df.to_dict('records'))
            student_id, grade, score, best_match = result
            
            if student_id:
                self.app.stats['voice_successes'] += 1
                self.app.stats['grading_timestamps'].append(time.time())
                self.excel_manager.update_grade(student_id, grade)
                
                # Actualizar UI
                self.window.evaluate_js(f"updateStudentGrade({student_id}, {grade}, true)")
                self.window.evaluate_js(f"showToast('Nota de {grade} para {best_match}', 'success')")
            else:
                if score > 0:
                    self.window.evaluate_js(f"showToast('No se entendió o no se encontró el alumno (Match: {score}%)', 'error')")
                else:
                    self.window.evaluate_js("showToast('No se detectó voz clara.', 'error')")

        threading.Thread(target=_process, daemon=True).start()

    def toggle_theme(self):
        """Toggles dark mode."""
        self.app.is_dark_mode = not self.app.is_dark_mode
        self.window.evaluate_js(f"setAppState({json.dumps(self.app.get_full_state())})")

    def toggle_sound(self):
        """Toggles sound/voice feedback."""
        self.app.is_sound_enabled = not self.app.is_sound_enabled
        self.window.evaluate_js(f"updateSoundUI({json.dumps(self.app.is_sound_enabled)})")

    def export_excel(self):
        """Exports the current grades."""
        if not self.excel_manager.current_file:
            self.window.evaluate_js("showToast('No hay clase cargada.', 'error')")
            return
            
        self.window.evaluate_js("showToast('Exportando Excel...', 'info')")
        
        # Lógica de exportación similar a app_window.py
        import shutil
        import datetime
        
        notas_dir = os.path.join(self.app.clickedu_dir, "notas")
        os.makedirs(notas_dir, exist_ok=True)
        
        original_filename = os.path.basename(self.excel_manager.current_file)
        new_filename = f"notas_{original_filename}"
        save_path = os.path.join(notas_dir, new_filename)
        
        success = self.excel_manager.export_excel(save_path)
        if success:
            self.window.evaluate_js(f"showToast('Guardado en /notas/{new_filename}', 'success')")
            os.startfile(notas_dir)
        else:
            self.window.evaluate_js("showToast('Error al exportar Excel.', 'error')")

    def get_stats(self):
        """Returns calculated stats to JS."""
        grades = self.excel_manager.df['Nota'].dropna().values if self.excel_manager.df is not None else []
        mean_grade = sum(grades) / len(grades) if len(grades) > 0 else 0
        
        # Calcular tiempo medio entre notas
        timestamps = self.app.stats['grading_timestamps']
        avg_time = 0
        if len(timestamps) > 1:
            diffs = [timestamps[i] - timestamps[i-1] for i in range(1, len(timestamps))]
            avg_time = sum(diffs) / len(diffs)

        return {
            "mean_grade": round(mean_grade, 2),
            "avg_time": round(avg_time, 1),
            "voice_successes": self.app.stats['voice_successes'],
            "total_students": len(self.excel_manager.df) if self.excel_manager.df is not None else 0
        }

class WebViewApp:
    def __init__(self):
        self.excel_manager = ExcelManager()
        self.voice_processor = VoiceProcessor()
        
        self.is_dark_mode = False
        self.is_sound_enabled = True
        self.stats = {
            'grading_timestamps': [],
            'voice_successes': 0
        }
        
        self._init_paths()
        self.api = Api(None, self) # El objeto window se asignará luego o no es necesario si usamos js_api
        self.window = webview.create_window(
            'Clickedu - Minimalismo Zen',
            url=self.html_path,
            js_api=self.api,
            width=1000,
            height=700,
            min_size=(900, 600),
            background_color='#F9FAF7'
        )
        self.api.window = self.window # Asignar la referencia de la ventana a la API

    def _init_paths(self):
        if getattr(sys, 'frozen', False):
            self.base_dir = sys._MEIPASS
            self.clickedu_dir = os.path.dirname(sys.executable)
        else:
            # __file__ es .../GradingApp/gui/webview_app.py
            # Queremos la raíz de GradingApp (o sea, un nivel arriba de gui)
            self.gui_dir = os.path.dirname(os.path.abspath(__file__))
            self.base_dir = os.path.dirname(self.gui_dir) # O sea, GradingApp/
            self.clickedu_dir = self.base_dir
            
        self.html_path = os.path.join(self.gui_dir, 'webview', 'index.html') if not getattr(sys, 'frozen', False) else os.path.join(self.base_dir, 'gui', 'webview', 'index.html')
        
        if not os.path.exists(self.html_path):
             # Fallback secundario
             print(f"DEBUG: Intentando fallback de ruta. No existe: {self.html_path}")
             self.html_path = os.path.join(os.getcwd(), 'gui', 'webview', 'index.html')
             
        print(f"DEBUG: Cargando HTML desde: {self.html_path}")
             
        self.plantillas_dir = os.path.join(self.clickedu_dir, "plantillas_clickedu_aulas")
        self.classList = []
        if os.path.exists(self.plantillas_dir):
            self.classList = [f for f in os.listdir(self.plantillas_dir) if f.endswith('.xlsx')]

    def load_class(self, display_name):
        file_path = os.path.join(self.plantillas_dir, display_name)
        students = self.excel_manager.load_excel(file_path)
        if students is not None:
            self.current_class_name = display_name
            self.sync_state_to_js()

    def get_full_state(self):
        return {
            "students": self.excel_manager.df.to_dict('records') if self.excel_manager.df is not None else [],
            "className": getattr(self, 'current_class_name', None),
            "classList": self.classList,
            "isDarkMode": self.is_dark_mode,
            "isSoundEnabled": self.is_sound_enabled
        }

    def sync_state_to_js(self):
        state = self.get_full_state()
        self.window.evaluate_js(f"setAppState({json.dumps(state)})")

    def run(self):
        webview.start(debug=True)

if __name__ == "__main__":
    app = WebViewApp()
    app.run()
