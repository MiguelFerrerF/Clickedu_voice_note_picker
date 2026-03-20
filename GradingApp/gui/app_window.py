import customtkinter as ctk
from tkinter import filedialog
import os
import re
import pandas as pd
import threading
import winsound
from core.excel_manager import ExcelManager
from core.voice_processor import VoiceProcessor

class AppWindow(ctk.CTk):
    def __init__(self):
        super().__init__()
        
        self.title("Gestor de Notas - Clickedu")
        self.geometry("900x600")
        self.minsize(600, 400)
        
        self.excel_manager = ExcelManager()
        self.voice_processor = VoiceProcessor()
        
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
        
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(1, weight=1)
        
        self._setup_main_area()
        self._setup_sidebar()
        
        # Seleccionar la primera clase por defecto si existe
        display_names = list(self.file_mapping.keys())
        if display_names:
            first_class = display_names[0]
            self.class_optionmenu.set(first_class)
            self.select_class_event(first_class)
        else:
            self.class_optionmenu.set("Sin clases")
        
    def _setup_sidebar(self):
        # Sidebar for controls
        self.sidebar_frame = ctk.CTkFrame(self, width=200, corner_radius=0)
        self.sidebar_frame.grid(row=0, column=0, rowspan=3, sticky="nsew")
        self.sidebar_frame.grid_rowconfigure(5, weight=1)
        
        self.logo_label = ctk.CTkLabel(self.sidebar_frame, text="Gestor de Notas", font=ctk.CTkFont(size=20, weight="bold"))
        self.logo_label.grid(row=0, column=0, padx=20, pady=(20, 10))
        
        # Directorio de plantillas adaptado para PyInstaller (.exe)
        import sys
        if getattr(sys, 'frozen', False):
            exe_dir = os.path.dirname(sys.executable)
            # Primero probamos la carpeta del exe
            clickedu_dir = exe_dir
            # Si resulta que la carpeta ya existe un nivel por encima, la ubicamos ahí
            if not os.path.exists(os.path.join(exe_dir, "plantillas_clickedu_aulas")) and os.path.exists(os.path.join(os.path.dirname(exe_dir), "plantillas_clickedu_aulas")):
                clickedu_dir = os.path.dirname(exe_dir)
        else:
            base_dir = os.path.dirname(os.path.abspath(__file__))
            clickedu_dir = os.path.dirname(os.path.dirname(base_dir))
            
        self.plantillas_dir = os.path.join(clickedu_dir, "plantillas_clickedu_aulas")
        os.makedirs(self.plantillas_dir, exist_ok=True)
        
        self.all_files = []
        if os.path.exists(self.plantillas_dir):
            self.all_files = [f for f in os.listdir(self.plantillas_dir) if f.endswith('.xlsx')]
            
        self.file_mapping = {self._format_filename(f): f for f in self.all_files}
        
        # Filtro
        self.filter_entry = ctk.CTkEntry(self.sidebar_frame, placeholder_text="🔍 Filtrar clase...")
        self.filter_entry.grid(row=1, column=0, padx=20, pady=(10, 5), sticky="ew")
        self.filter_entry.bind("<KeyRelease>", self.filter_classes_event)
        
        # Opciones
        display_names = list(self.file_mapping.keys())
        
        self.class_optionmenu = ctk.CTkOptionMenu(self.sidebar_frame, values=display_names if display_names else ["Sin clases"], command=self.select_class_event)
        self.class_optionmenu.grid(row=2, column=0, padx=20, pady=(5, 10), sticky="ew")
        
        self.btn_export_excel = ctk.CTkButton(self.sidebar_frame, text="Exportar Notas", command=self.export_excel_event, state="disabled", fg_color="#2ECC71", hover_color="#27AE60", text_color="white")
        self.btn_export_excel.grid(row=3, column=0, padx=20, pady=20)
        
        # Controles inferiores (Tema y Sonido)
        self.bottom_controls_frame = ctk.CTkFrame(self.sidebar_frame, fg_color="transparent")
        self.bottom_controls_frame.grid(row=6, column=0, padx=20, pady=(10, 20), sticky="ew")
        self.bottom_controls_frame.grid_columnconfigure(0, weight=1)
        
        self.sound_enabled = False
        self.sound_button = ctk.CTkButton(self.bottom_controls_frame, text="Sonido: OFF", font=ctk.CTkFont(size=12, weight="bold"), command=self.toggle_sound_event, fg_color="#95A5A6", hover_color="#7F8C8D", text_color="white", height=32)
        self.sound_button.grid(row=0, column=0, pady=5, sticky="ew")
        
        self.current_theme = ctk.get_appearance_mode()
        if self.current_theme == "Light":
            theme_text = "Modo: Claro"
            theme_color, theme_hover = "#F39C12", "#D68910"
        else:
            theme_text = "Modo: Oscuro"
            theme_color, theme_hover = "#8E44AD", "#7D3C98"
            
        self.theme_button = ctk.CTkButton(self.bottom_controls_frame, text=theme_text, font=ctk.CTkFont(size=12, weight="bold"), command=self.toggle_theme_event, fg_color=theme_color, hover_color=theme_hover, text_color="white", height=32)
        self.theme_button.grid(row=1, column=0, sticky="ew")
        
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
        
    def filter_classes_event(self, event):
        search_term = self.filter_entry.get().lower()
        filtered_names = [name for name in self.file_mapping.keys() if search_term in name.lower()]
        
        if not filtered_names:
            filtered_names = ["Sin resultados"]
            self.class_optionmenu.configure(values=filtered_names)
            self.class_optionmenu.set(filtered_names[0])
        else:
            self.class_optionmenu.configure(values=filtered_names)
            # Only change selection if searching narrows it down to 1
            if len(filtered_names) == 1:
                self.class_optionmenu.set(filtered_names[0])
                self.select_class_event(filtered_names[0])
        
    def _setup_main_area(self):
        # Header
        self.header_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.header_frame.grid(row=0, column=1, sticky="ew", padx=20, pady=10)
        self.header_frame.grid_columnconfigure(0, weight=1)
        self.header_frame.grid_columnconfigure(1, weight=1)
        
        self.title_label = ctk.CTkLabel(self.header_frame, text="Clase Seleccionada: Ninguna", font=ctk.CTkFont(size=18, weight="bold"))
        self.title_label.grid(row=0, column=0, sticky="w")
        
        # Voice input Area
        self.voice_controls_frame = ctk.CTkFrame(self.header_frame, fg_color="transparent")
        self.voice_controls_frame.grid(row=0, column=2, sticky="e")
        self.voice_controls_frame.grid_columnconfigure(1, minsize=40)
        
        self.voice_hint_label = ctk.CTkLabel(self.voice_controls_frame, text="Mantén ESPACIO para dictar", font=ctk.CTkFont(size=12, slant="italic"), text_color="gray")
        self.voice_hint_label.grid(row=0, column=0, padx=10)
        
        self.btn_voice = ctk.CTkButton(self.voice_controls_frame, text="", width=20, height=20, corner_radius=10, fg_color="gray", hover_color="gray", state="disabled")
        self.btn_voice.grid(row=0, column=1)
        
        # PTT Bindings
        self.bind("<KeyPress-space>", self.on_space_press)
        self.bind("<KeyRelease-space>", self.on_space_release)
        self.is_space_pressed = False
        
        # Scrollable area for students
        self.students_frame = ctk.CTkScrollableFrame(self)
        self.students_frame.grid(row=1, column=1, sticky="nsew", padx=20, pady=10)
        self.students_frame.grid_columnconfigure(1, weight=1)
        
        # Placeholder text
        self.placeholder_label = ctk.CTkLabel(self.students_frame, text="Selecciona un archivo de Excel para comenzar.", text_color="gray")
        self.placeholder_label.pack(pady=50)
        
    def select_class_event(self, selected_name):
        if selected_name in self.file_mapping:
            file_path = os.path.join(self.plantillas_dir, self.file_mapping[selected_name])
            print(f"Excel seleccionado: {file_path}")
            students_data = self.excel_manager.load_excel(file_path)
            if students_data is not None:
                self.title_label.configure(text=f"Clase: {selected_name}")
                self.btn_export_excel.configure(state="normal")
                self.btn_voice.configure(state="normal")
                if hasattr(self, 'placeholder_label') and self.placeholder_label.winfo_exists():
                    self.placeholder_label.pack_forget()
                self._populate_studentsList(students_data)
            else:
                if hasattr(self, 'placeholder_label') and self.placeholder_label.winfo_exists():
                    self.placeholder_label.configure(text="Error al cargar el Excel. Comprueba el formato.", text_color="red")
                    self.placeholder_label.pack(pady=10)
            
    def _populate_studentsList(self, data):
        for widget in self.students_frame.winfo_children():
            widget.destroy()
            
        self.grade_entries = {}
        self.entry_list = []
        
        vcmd = (self.register(self.validate_grade), '%P')
        
        header_name = ctk.CTkLabel(self.students_frame, text="Nombre del Alumno", font=ctk.CTkFont(weight="bold"))
        header_name.grid(row=0, column=0, padx=10, pady=10, sticky="w")
        header_grade = ctk.CTkLabel(self.students_frame, text="Nota", font=ctk.CTkFont(weight="bold"))
        header_grade.grid(row=0, column=1, padx=10, pady=10, sticky="e")
        
        for i, student in enumerate(data):
            row = i + 1
            lbl = ctk.CTkLabel(self.students_frame, text=student['Nombre'])
            lbl.grid(row=row, column=0, padx=10, pady=5, sticky="w")
            
            entry = ctk.CTkEntry(self.students_frame, width=80, justify="center", validate="key", validatecommand=vcmd)
            nota = student.get('Nota')
            if pd.notna(nota):
                entry.insert(0, str(nota))
            entry.grid(row=row, column=1, padx=10, pady=5, sticky="e")
            
            entry.bind("<KeyRelease>", lambda event, s_id=student['ID'], ent=entry: self._on_grade_change(event, s_id, ent))
            entry.bind("<Return>", lambda event, idx=i: self._focus_next_entry(event, idx))
            entry.bind("<Down>", lambda event, idx=i: self._focus_next_entry(event, idx))
            entry.bind("<Up>", lambda event, idx=i: self._focus_prev_entry(event, idx))
            
            self.grade_entries[student['ID']] = entry
            self.entry_list.append(entry)
            
    def validate_grade(self, new_value):
        if new_value == "":
            return True
        import re
        if re.fullmatch(r'\d{0,3}([.,]\d{0,1})?', new_value):
            return True
        return False
            
    def _on_grade_change(self, event, student_id, entry_widget):
        val = entry_widget.get().strip()
        try:
            if val == "":
                self.excel_manager.update_grade(student_id, None)
                entry_widget.configure(text_color=["black", "white"])
            else:
                val_float = float(val.replace(',', '.'))
                self.excel_manager.update_grade(student_id, val_float)
                entry_widget.configure(text_color=["black", "white"])
        except ValueError:
            entry_widget.configure(text_color="red")
            
    def export_excel_event(self):
        if not self.excel_manager.current_file:
            self.show_toast("Error: No hay clase seleccionada.", True)
            return
            
        import shutil
        import datetime
        
        clickedu_dir = os.path.dirname(self.plantillas_dir)
        notas_dir = os.path.join(clickedu_dir, "notas")
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
            print("Exportado exitosamente a", new_save_path)
            # Abrir carpeta en el explorador de Windows
            os.startfile(notas_dir)
        else:
            self.show_toast("Error al intentar exportar.", is_error=True)
            print("Error exportando")
        
    def on_space_press(self, event):
        fw = self.focus_get()
        if hasattr(self, 'search_entry') and fw == self.search_entry._entry: return
        
        if self.excel_manager.df is None: return
        if self.btn_voice.cget("state") != "normal": return
            
        if hasattr(self, '_stop_ptt_after_id'):
            self.after_cancel(self._stop_ptt_after_id)
            delattr(self, '_stop_ptt_after_id')
            
        if not self.is_space_pressed:
            self.is_space_pressed = True
            self.voice_hint_label.configure(text="Escuchando...", text_color="#E74C3C")
            self.voice_processor.start_recording()
            
            self._blink_state = True
            self._blink_recording_indicator()
            
    def _blink_recording_indicator(self):
        if not getattr(self, 'is_space_pressed', False):
            self.btn_voice.configure(fg_color="gray")
            return
            
        new_color = "#E74C3C" if self._blink_state else "gray"
        self.btn_voice.configure(fg_color=new_color)
        self._blink_state = not self._blink_state
        self.after(400, self._blink_recording_indicator)

    def on_space_release(self, event):
        fw = self.focus_get()
        if hasattr(self, 'search_entry') and fw == self.search_entry._entry: return
        
        if self.is_space_pressed:
            self._stop_ptt_after_id = self.after(50, self._execute_stop_ptt)
            
    def _execute_stop_ptt(self):
        if hasattr(self, '_stop_ptt_after_id'):
            delattr(self, '_stop_ptt_after_id')
            
        self.is_space_pressed = False
        self.btn_voice.configure(fg_color="gray")
        self.voice_hint_label.configure(text="Procesando...", text_color="#3498DB")
        threading.Thread(target=self._process_ptt).start()
        
    def _focus_next_entry(self, event, current_idx):
        if hasattr(self, 'entry_list') and current_idx + 1 < len(self.entry_list):
            next_entry = self.entry_list[current_idx + 1]
            next_entry.focus_set()
            next_entry.select_range(0, 'end')

    def _focus_prev_entry(self, event, current_idx):
        if hasattr(self, 'entry_list') and current_idx - 1 >= 0:
            prev_entry = self.entry_list[current_idx - 1]
            prev_entry.focus_set()
            prev_entry.select_range(0, 'end')

    def _process_ptt(self):
        students_list = self.excel_manager.df.to_dict('records')
        student_id, grade = self.voice_processor.stop_recording_and_process(students_list)
        self.after(0, self._on_voice_result, student_id, grade)
        
    def _on_voice_result(self, student_id, grade):
        self.voice_hint_label.configure(text="Mantén ESPACIO para dictar", text_color="gray")
        if student_id is not None and grade is not None:
            self.excel_manager.update_grade(student_id, grade)
            if student_id in self.grade_entries:
                entry = self.grade_entries[student_id]
                entry.delete(0, 'end')
                entry.insert(0, str(grade))
                entry.configure(text_color=["green", "lightgreen"])
                self.show_toast(f"Nota de {grade} añadida.", is_error=False)
                self.after(2000, lambda: entry.configure(text_color=["black", "white"]))
        else:
            self.show_toast("No se entendió o no se encontró el alumno.", is_error=True)

    def show_toast(self, message, is_error=False):
        if is_error:
            bg_color = "#E74C3C" 
            if getattr(self, 'sound_enabled', True):
                winsound.MessageBeep(winsound.MB_ICONHAND)
        else:
            bg_color = "#2ECC71"
            if getattr(self, 'sound_enabled', True):
                winsound.MessageBeep(winsound.MB_OK)
            
        toast = ctk.CTkFrame(self, fg_color=bg_color, corner_radius=10)
        lbl = ctk.CTkLabel(toast, text=message, text_color="white", font=ctk.CTkFont(weight="bold", size=14))
        lbl.pack(padx=20, pady=10)
        
        toast.place(relx=0.5, rely=0.9, anchor="center")
        self.after(3000, toast.destroy)

    def toggle_theme_event(self):
        if self.current_theme == "Dark":
            ctk.set_appearance_mode("Light")
            self.current_theme = "Light"
            self.theme_button.configure(text="Modo: Claro", fg_color="#F39C12", hover_color="#D68910")
        else:
            ctk.set_appearance_mode("Dark")
            self.current_theme = "Dark"
            self.theme_button.configure(text="Modo: Oscuro", fg_color="#8E44AD", hover_color="#7D3C98")

    def toggle_sound_event(self):
        self.sound_enabled = not getattr(self, 'sound_enabled', True)
        if self.sound_enabled:
            self.sound_button.configure(text="Sonido: ON", fg_color="#3498DB", hover_color="#2980B9")
        else:
            self.sound_button.configure(text="Sonido: OFF", fg_color="#95A5A6", hover_color="#7F8C8D")
