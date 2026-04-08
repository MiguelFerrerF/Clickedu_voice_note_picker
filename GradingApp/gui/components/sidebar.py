import customtkinter as ctk
import os
import json
import re

class Sidebar(ctk.CTkFrame):
    def __init__(self, master, config_path, display_names, 
                 on_class_select, on_export_notes, on_toggle_stats, 
                 on_connect_clickedu=None, on_upload_clickedu=None, **kwargs):
        super().__init__(master, corner_radius=0, **kwargs)
        
        self.config_path = config_path
        self._load_settings()
        self.display_names = display_names
        
        self.on_class_select = on_class_select
        self.on_export_notes = on_export_notes
        self.on_toggle_stats = on_toggle_stats
        self.on_connect_clickedu = on_connect_clickedu
        self.on_upload_clickedu = on_upload_clickedu

        
        self._build_ui()
        
    def _build_ui(self):
        self.grid_rowconfigure(4, weight=1)
        
        self.logo_label = ctk.CTkLabel(self, text="Gestor de Notas", font=ctk.CTkFont(size=20, weight="bold"))
        self.logo_label.grid(row=0, column=0, padx=20, pady=(20, 10))
        
        # Opciones y Filtrado Integrado
        self.class_combobox = ctk.CTkComboBox(
            self, 
            values=self.display_names if self.display_names else ["Sin clases"], 
            command=self.on_class_select,
            dropdown_hover_color="#2980B9"
        )
        self.class_combobox.grid(row=1, column=0, padx=20, pady=(20, 10), sticky="ew")
        self.class_combobox.bind("<KeyRelease>", self.filter_classes_event)
        
        # Seleccionar la primera clase por defecto
        if self.display_names:
            first_class = self.display_names[0]
            self.class_combobox.set(first_class)
            # Notificará automáticamente a app_window porque fue llamado al iniciar? No, se debe llamar manual en AppWindow
        else:
            self.class_combobox.set("Sin clases")

        self.btn_export_excel = ctk.CTkButton(self, text="Exportar Notas", command=self.on_export_notes, state="disabled", fg_color="#2ECC71", hover_color="#27AE60", text_color="white", text_color_disabled="white", height=40)
        self.btn_export_excel.grid(row=2, column=0, padx=20, pady=(0, 10), sticky="ew")
        
        # Clickedu Controls
        self.clickedu_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.clickedu_frame.grid(row=3, column=0, padx=20, pady=(10, 10), sticky="ew")
        self.clickedu_frame.grid_columnconfigure(0, weight=1)
        
        self.btn_connect = ctk.CTkButton(self.clickedu_frame, text="Conectar a Clickedu", command=self.on_connect_clickedu, fg_color="#8E44AD", hover_color="#9B59B6", height=32)
        self.btn_connect.grid(row=0, column=0, pady=(0, 5), sticky="ew")

        self.btn_upload = ctk.CTkButton(self.clickedu_frame, text="Subir Notas a Clickedu", command=self.on_upload_clickedu, state="disabled", fg_color="#7F8C8D", hover_color="#95A5A6", text_color_disabled="white", height=32)
        self.btn_upload.grid(row=1, column=0, sticky="ew")
        
        # Controles inferiores
        self.bottom_controls_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.bottom_controls_frame.grid(row=5, column=0, padx=20, pady=(10, 20), sticky="ew")
        self.bottom_controls_frame.grid_columnconfigure(0, weight=1)
        
        self.btn_stats = ctk.CTkButton(self.bottom_controls_frame, text="Estadísticas", height=40, font=ctk.CTkFont(weight="bold"), 
                                       command=self.on_toggle_stats, fg_color="#3498DB", hover_color="#2980B9", state="disabled")
        self.btn_stats.grid(row=0, column=0, pady=(0, 10), sticky="ew")
        
        theme_str = "Claro" if self.current_theme == "Light" else "Oscuro"
        sound_str = "ON" if self.sound_enabled else "OFF"
        
        self.settings_menu = ctk.CTkOptionMenu(
            self.bottom_controls_frame, 
            values=[f"Tema: {theme_str} (Cambiar)", f"Sonido: {sound_str} (Cambiar)"],
            command=self.handle_settings_menu,
            fg_color="#34495E", button_color="#2C3E50", button_hover_color="#1A252F",
            dropdown_hover_color="#2980B9"
        )
        self.settings_menu.set("⚙️ Ajustes")
        self.settings_menu.grid(row=1, column=0, sticky="ew")

    def filter_classes_event(self, event):
        typed_text = self.class_combobox.get().lower()
        if typed_text == "":
            filtered = self.display_names
        else:
            filtered = [name for name in self.display_names if typed_text in name.lower()]
        
        self.class_combobox.configure(values=filtered if filtered else ["No se encontró ninguna"])

    def handle_settings_menu(self, selected):
        if "Tema" in selected:
            self.toggle_theme_event()
        elif "Sonido" in selected:
            self.toggle_sound_event()
            
        theme_str = "Claro" if self.current_theme == "Light" else "Oscuro"
        sound_str = "ON" if self.sound_enabled else "OFF"
        self.settings_menu.configure(values=[f"Tema: {theme_str} (Cambiar)", f"Sonido: {sound_str} (Cambiar)"])
        self.settings_menu.set("⚙️ Ajustes")
        self._save_settings()

    def _load_settings(self):
        self.current_theme = "System"
        self.sound_enabled = True
        try:
            if os.path.exists(self.config_path):
                with open(self.config_path, "r", encoding="utf-8") as f:
                    config = json.load(f)
                    self.current_theme = config.get("theme", "System")
                    self.sound_enabled = config.get("sound_enabled", True)
        except Exception:
            pass
        
        if self.current_theme not in ["Light", "Dark", "System"]:
            self.current_theme = "System"
        ctk.set_appearance_mode(self.current_theme)
        
    def _save_settings(self):
        try:
            config = {
                "theme": self.current_theme,
                "sound_enabled": self.sound_enabled
            }
            with open(self.config_path, "w", encoding="utf-8") as f:
                json.dump(config, f)
        except Exception as e:
            print("Error guardando ajustes:", e)

    def toggle_theme_event(self):
        if self.current_theme == "Dark" or self.current_theme == "System":
            ctk.set_appearance_mode("Light")
            self.current_theme = "Light"
        else:
            ctk.set_appearance_mode("Dark")
            self.current_theme = "Dark"

    def toggle_sound_event(self):
        self.sound_enabled = not getattr(self, 'sound_enabled', True)
