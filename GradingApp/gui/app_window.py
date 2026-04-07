import customtkinter as ctk
import os
import re
import time
import winsound
import threading

from core.excel_manager import ExcelManager
from core.voice_processor import VoiceProcessor
from .components.sidebar import Sidebar
from .components.student_grid import StudentGrid
from .components.stats_view import StatsView

class AppWindow(ctk.CTk):
    def __init__(self):
        super().__init__()
        
        self.title("Gestor de Notas - Clickedu")
        self.geometry("1000x700")
        self.minsize(900, 600)
        
        self.excel_manager = ExcelManager()
        self.voice_processor = VoiceProcessor()
        
        self.stats = {
            'grading_timestamps': [],
            'voice_attempts': 0,
            'voice_successes': 0,
            'lowest_match': {'name': None, 'score': 100}
        }
        self.is_stats_view_open = False
        self.stats_main_frame = None
        
        try:
            import ctypes
            myappid = 'clickedu.gestordenotas.v1'
            ctypes.windll.shell32.SetCurrentProcessExplicitAppUserModelID(myappid)
        except Exception:
            pass
            
        import sys
        if getattr(sys, 'frozen', False):
            icon_path = os.path.join(sys._MEIPASS, "icon.ico")
        else:
            icon_path = os.path.join(os.path.dirname(os.path.dirname(__file__)), "icon.ico")
            
        if os.path.exists(icon_path):
            self.iconbitmap(icon_path)
        
        self.grid_columnconfigure(0, minsize=240, weight=0)
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(1, weight=1)
        
        self._init_data_paths()
        self._setup_main_area()
        
        display_names = list(self.file_mapping.keys())
        self.sidebar = Sidebar(
            self,
            config_path=self.config_path,
            display_names=display_names,
            on_class_select=self.select_class_event,
            on_export_notes=self.export_excel_event,
            on_toggle_stats=self.toggle_stats_view
        )
        self.sidebar.grid(row=0, column=0, rowspan=3, sticky="nsew")
        
        if display_names:
            first_class = display_names[0]
            self.sidebar.class_combobox.set(first_class)
            self.select_class_event(first_class)
            
    def _init_data_paths(self):
        import sys
        if getattr(sys, 'frozen', False):
            exe_dir = os.path.dirname(sys.executable)
            self.clickedu_dir = exe_dir
            if not os.path.exists(os.path.join(exe_dir, "plantillas_clickedu_aulas")) and os.path.exists(os.path.join(os.path.dirname(exe_dir), "plantillas_clickedu_aulas")):
                self.clickedu_dir = os.path.dirname(exe_dir)
        else:
            base_dir = os.path.dirname(os.path.abspath(__file__))
            self.clickedu_dir = os.path.dirname(os.path.dirname(base_dir))
            
        self.plantillas_dir = os.path.join(self.clickedu_dir, "plantillas_clickedu_aulas")
        os.makedirs(self.plantillas_dir, exist_ok=True)
        self.config_path = os.path.join(self.clickedu_dir, "config.json")
        
        self.all_files = []
        if os.path.exists(self.plantillas_dir):
            self.all_files = [f for f in os.listdir(self.plantillas_dir) if f.endswith('.xlsx')]
            
        self.file_mapping = {self._format_filename(f): f for f in self.all_files}

    def _format_filename(self, filename):
        name_without_ext = os.path.splitext(filename)[0]
        parts = name_without_ext.split('_')
        if len(parts) >= 1:
            subject = re.sub(r"([A-Z])", r" \1", parts[0]).strip().title()
            res = [subject]
            if len(parts) >= 2:
                res.append(f"{parts[1]}º")
            if len(parts) >= 3:
                res.append(parts[2].upper())
            if len(parts) >= 4:
                level = parts[3].lower()
                if "bach" in level:
                    res.append("Bachillerato")
                elif "eso" in level:
                    res.append("ESO")
                else:
                    res.append(parts[3].upper())
            return " ".join(res)
        return name_without_ext.replace("_", " ").title()

    def _setup_main_area(self):
        self.header_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.header_frame.grid(row=0, column=1, sticky="ew", padx=20, pady=10)
        self.header_frame.grid_columnconfigure(0, weight=1)
        self.header_frame.grid_columnconfigure(1, weight=1)
        
        self.title_label = ctk.CTkLabel(self.header_frame, text="Clase Seleccionada: Ninguna", font=ctk.CTkFont(size=18, weight="bold"))
        self.title_label.grid(row=0, column=0, sticky="w")
        
        self.voice_controls_frame = ctk.CTkFrame(self.header_frame, fg_color="transparent")
        self.voice_controls_frame.grid(row=0, column=2, sticky="e")
        self.voice_controls_frame.grid_columnconfigure(1, minsize=40)
        
        self.voice_hint_label = ctk.CTkLabel(self.voice_controls_frame, text="Mantén ESPACIO para dictar", font=ctk.CTkFont(size=12, slant="italic"), text_color="gray")
        self.voice_hint_label.grid(row=0, column=0, padx=10)
        
        self.btn_voice = ctk.CTkButton(self.voice_controls_frame, text="", width=20, height=20, corner_radius=10, fg_color="gray", hover_color="gray", state="disabled")
        self.btn_voice.grid(row=0, column=1)
        
        self.bind("<KeyPress-space>", self.on_space_press)
        self.bind("<KeyRelease-space>", self.on_space_release)
        self.is_space_pressed = False
        
        self.student_grid = StudentGrid(self, on_grade_update=self._on_grade_update)
        self.student_grid.grid(row=1, column=1, sticky="nsew", padx=20, pady=10)
        
        self.bind("<Configure>", self._on_window_resize)

    def _on_window_resize(self, event):
        if hasattr(event, 'widget') and event.widget != self:
            return
        width = event.width
        target_cols = 2 if width >= 1000 else 1
        self.student_grid.resize(target_cols)

    def select_class_event(self, selected_name):
        if selected_name in self.file_mapping:
            file_path = os.path.join(self.plantillas_dir, self.file_mapping[selected_name])
            print(f"Excel seleccionado: {file_path}")
            students_data = self.excel_manager.load_excel(file_path)
            if students_data is not None:
                self.current_students_data = students_data
                self.title_label.configure(text=f"Clase: {selected_name}")
                self.sidebar.btn_export_excel.configure(state="normal")
                self.sidebar.btn_stats.configure(state="normal")
                self.btn_voice.configure(state="normal")
                self.stats = {'grading_timestamps': [], 'voice_attempts': 0, 'voice_successes': 0, 'lowest_match': {'name': None, 'score': 100}}
                
                if self.is_stats_view_open:
                    self.toggle_stats_view()
                    
                self.student_grid.populate(students_data)
            else:
                self.student_grid.show_placeholder(is_error=True)

    def _on_grade_update(self, student_id, grade):
        if grade is None:
            self.excel_manager.update_grade(student_id, None)
        else:
            self.excel_manager.update_grade(student_id, grade)
        self.stats['grading_timestamps'].append(time.time())

    def on_space_press(self, event):
        if not self.excel_manager.current_file or self.is_space_pressed:
            return
            
        focused = self.focus_get()
        if isinstance(focused, ctk.CTkEntry):
            return
            
        self.is_space_pressed = True
        self.btn_voice.configure(fg_color="#E74C3C")
        self.voice_hint_label.configure(text="Escuchando...", text_color="#E74C3C")
        
        self.voice_processor.start_recording()

    def on_space_release(self, event):
        if not self.is_space_pressed:
            return
        self.is_space_pressed = False
        self.btn_voice.configure(fg_color="gray")
        self.voice_hint_label.configure(text="Procesando...", text_color="#F39C12")
        
        audio = self.voice_processor.stop_recording()
        if not audio:
            self.voice_hint_label.configure(text="Mantén ESPACIO para dictar", text_color="gray")
            return
            
        threading.Thread(target=self.process_voice_command, args=(audio,), daemon=True).start()

    def process_voice_command(self, audio):
        result = self.voice_processor.process_audio(audio, getattr(self, 'current_students_data', []))
        
        self.voice_hint_label.configure(text="Mantén ESPACIO para dictar", text_color="gray")
        
        if result == "ERROR":
            self.show_toast("No se entendió el audio.", is_error=True)
            return
            
        self.stats['voice_attempts'] += 1
        
        student_id, grade, score, best_match = result
        if student_id is not None:
            self.stats['voice_successes'] += 1
            if score < self.stats['lowest_match']['score']:
                self.stats['lowest_match'] = {'name': best_match, 'score': score}
                
            self.excel_manager.update_grade(student_id, grade)
            self.stats['grading_timestamps'].append(time.time())
            
            self.student_grid.update_student_grade_ui(student_id, grade)
            self.show_toast(f"Nota de {grade} añadida.", is_error=False)
        else:
            self.show_toast("No se entendió o no se encontró el alumno.", is_error=True)

    def show_toast(self, message, is_error=False):
        if is_error:
            bg_color = "#E74C3C" 
            if self.sidebar.sound_enabled:
                winsound.MessageBeep(winsound.MB_ICONHAND)
        else:
            bg_color = "#2ECC71"
            if self.sidebar.sound_enabled:
                winsound.MessageBeep(winsound.MB_OK)
            
        toast = ctk.CTkFrame(self, fg_color=bg_color, corner_radius=10)
        lbl = ctk.CTkLabel(toast, text=message, text_color="white", font=ctk.CTkFont(weight="bold", size=14))
        lbl.pack(padx=20, pady=10)
        
        toast.place(relx=0.5, rely=0.9, anchor="center")
        self.after(3000, toast.destroy)

    def export_excel_event(self):
        if not self.excel_manager.current_file:
            self.show_toast("Error: No hay clase seleccionada.", True)
            return
            
        import shutil
        import datetime
        
        notas_dir = os.path.join(self.clickedu_dir, "notas")
        notas_viejas_dir = os.path.join(notas_dir, "notas_viejas")
        
        os.makedirs(notas_dir, exist_ok=True)
        os.makedirs(notas_viejas_dir, exist_ok=True)
        
        for f in os.listdir(notas_dir):
            file_path = os.path.join(notas_dir, f)
            if os.path.isfile(file_path) and f.endswith('.xlsx'):
                dest_path = os.path.join(notas_viejas_dir, f)
                if os.path.exists(dest_path):
                    ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
                    name, ext = os.path.splitext(f)
                    dest_path = os.path.join(notas_viejas_dir, f"{name}_{ts}{ext}")
                try:
                    shutil.move(file_path, dest_path)
                except Exception as e:
                    print(f"Error moviendo {f}: {e}")
                    
        original_filename = os.path.basename(self.excel_manager.current_file)
        new_filename = f"notas_{original_filename}"
        new_save_path = os.path.join(notas_dir, new_filename)
        
        success = self.excel_manager.export_excel(new_save_path)
        if success:
            self.show_toast(f"Plantilla exportada a /notas/", is_error=False)
            os.startfile(notas_dir)
        else:
            self.show_toast("Error al intentar exportar.", is_error=True)

    def _cleanup_stats_wrapper(self):
        if hasattr(self, 'stats_wrapper_frame') and self.stats_wrapper_frame:
            self.stats_wrapper_frame.destroy()
            self.stats_wrapper_frame = None
            self.stats_main_frame = None

    def toggle_stats_view(self):
        if not self.is_stats_view_open:
            df_last_column = None
            if self.excel_manager.df is not None:
                df_last_column = self.excel_manager.df.iloc[:, -1]
                
            # Preconstruir la vista de forma invisible para que no de tirones
            new_wrapper = ctk.CTkFrame(self, fg_color="transparent")
            new_stats = StatsView(new_wrapper, self.stats, df_last_column)
            new_stats.pack(expand=True, fill="both")
            
            # Ocultar las otras
            self.header_frame.grid_forget()
            self.student_grid.grid_forget()
            
            # Limpiar si hubiera anterior
            if getattr(self, 'stats_wrapper_frame', None):
                self.stats_wrapper_frame.destroy()
                
            self.stats_wrapper_frame = new_wrapper
            self.stats_main_frame = new_stats
            
            # Mostrar la nueva de forma fluida
            self.stats_wrapper_frame.grid(row=0, column=1, rowspan=2, sticky="nsew", padx=20, pady=20)
            
            self.sidebar.btn_stats.configure(text="Volver a Notas", fg_color="#E67E22", hover_color="#D35400")
            self.is_stats_view_open = True
        else:
            # Restaurar la vista primero para que no haya salto en blanco
            self.header_frame.grid(row=0, column=1, sticky="ew", padx=20, pady=10)
            self.student_grid.grid(row=1, column=1, sticky="nsew", padx=20, pady=10)
            self.student_grid.tkraise()
            
            # Ocultar la de stats y luego destruirla perezosamente en el fondo
            if getattr(self, 'stats_wrapper_frame', None):
                self.stats_wrapper_frame.grid_forget()
                self.after(50, self._cleanup_stats_wrapper)


            
            self.sidebar.btn_stats.configure(text="Estadísticas", fg_color="#3498DB", hover_color="#2980B9")
            self.is_stats_view_open = False

    def show_update_banner(self, latest_version, download_url, file_size, updater_instance):
        self.title_label.grid(row=1, column=0, sticky="w", pady=(10, 0))
        self.voice_controls_frame.grid(row=1, column=2, sticky="e", pady=(10, 0))
        
        self.update_banner = ctk.CTkFrame(self.header_frame, fg_color="#2980B9", corner_radius=8)
        self.update_banner.grid(row=0, column=0, columnspan=3, sticky="ew", pady=(0, 5))
        self.update_banner.grid_columnconfigure(0, weight=1)
        
        lbl = ctk.CTkLabel(self.update_banner, text=f"¡Nueva versión disponible! ({latest_version})", text_color="white", font=ctk.CTkFont(weight="bold"))
        lbl.grid(row=0, column=0, padx=15, pady=8, sticky="w")
        
        btn_frame = ctk.CTkFrame(self.update_banner, fg_color="transparent")
        btn_frame.grid(row=0, column=1, padx=10, pady=8, sticky="e")
        
        btn_ignore = ctk.CTkButton(btn_frame, text="Rechazar", width=80, fg_color="#E74C3C", hover_color="#C0392B", command=self.hide_update_banner)
        btn_ignore.pack(side="left", padx=5)
        
        btn_accept = ctk.CTkButton(btn_frame, text="Actualizar", width=80, fg_color="#27AE60", hover_color="#2ECC71", command=lambda: self.start_inline_update(download_url, file_size, updater_instance))
        btn_accept.pack(side="left", padx=5)

    def hide_update_banner(self):
        if hasattr(self, 'update_banner') and self.update_banner and self.update_banner.winfo_exists():
            self.update_banner.destroy()
            self.title_label.grid(row=0, column=0, sticky="w", pady=0)
            self.voice_controls_frame.grid(row=0, column=2, sticky="e", pady=0)

    def start_inline_update(self, download_url, file_size, updater_instance):
        for widget in self.update_banner.winfo_children():
            widget.destroy()
            
        self.update_banner.configure(fg_color="#34495E")
            
        lbl = ctk.CTkLabel(self.update_banner, text="Descargando actualización, espere...", text_color="white", font=ctk.CTkFont(weight="bold"))
        lbl.grid(row=0, column=0, padx=15, pady=8, sticky="w")
        
        self.update_progress = ctk.CTkProgressBar(self.update_banner, width=300)
        self.update_progress.grid(row=0, column=1, padx=15, pady=8, sticky="e")
        self.update_progress.set(0)
        
        hilo = threading.Thread(
            target=updater_instance.download_and_install_inline, 
            args=(download_url, file_size, self.update_progress, self),
            daemon=True
        )
        hilo.start()
