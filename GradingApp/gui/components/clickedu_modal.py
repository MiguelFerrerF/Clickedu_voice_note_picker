import customtkinter as ctk
import os
import json
from tkinter import filedialog
import threading

class ClickeduModal(ctk.CTkToplevel):
    def __init__(self, master, config_path, login_callback, *args, **kwargs):
        super().__init__(master, *args, **kwargs)
        
        self.title("Conectar a Clickedu")
        self.geometry("450x450")
        self.resizable(False, False)
        
        # Centrar la ventana
        self.update_idletasks()
        try:
            x = master.winfo_x() + (master.winfo_width() // 2) - (450 // 2)
            y = master.winfo_y() + (master.winfo_height() // 2) - (450 // 2)
            self.geometry(f"+{x}+{y}")
        except:
            pass

        self.config_path = config_path
        self.login_callback = login_callback
        self.security_file_path = None
        
        self.grid_columnconfigure(0, weight=1)
        
        # UI Elements
        self.lbl_title = ctk.CTkLabel(self, text="Inicio de Sesión - Clickedu", font=ctk.CTkFont(size=20, weight="bold"))
        self.lbl_title.grid(row=0, column=0, pady=(20, 10))
        
        # Username
        self.lbl_user = ctk.CTkLabel(self, text="Nombre de usuario:")
        self.lbl_user.grid(row=1, column=0, sticky="w", padx=30, pady=(10, 0))
        self.entry_user = ctk.CTkEntry(self, width=390)
        self.entry_user.grid(row=2, column=0, padx=30, pady=(0, 10))
        
        # Password
        self.lbl_pwd = ctk.CTkLabel(self, text="Contraseña:")
        self.lbl_pwd.grid(row=3, column=0, sticky="w", padx=30, pady=(10, 0))
        self.entry_pwd = ctk.CTkEntry(self, width=390, show="*")
        self.entry_pwd.grid(row=4, column=0, padx=30, pady=(0, 10))
        
        # Archivo de Seguridad
        self.lbl_file = ctk.CTkLabel(self, text="Archivo de seguridad:")
        self.lbl_file.grid(row=5, column=0, sticky="w", padx=30, pady=(10, 0))
        
        self.file_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.file_frame.grid(row=6, column=0, padx=30, sticky="ew")
        self.file_frame.grid_columnconfigure(0, weight=1)
        
        self.lbl_file_path = ctk.CTkLabel(self.file_frame, text="Ningún archivo seleccionado", text_color="gray", anchor="w")
        self.lbl_file_path.grid(row=0, column=0, sticky="ew", padx=(0, 10))
        
        self.btn_browse = ctk.CTkButton(self.file_frame, text="Buscar...", width=80, command=self._browse_file)
        self.btn_browse.grid(row=0, column=1)
        
        self.lbl_warning = ctk.CTkLabel(
            self, 
            text="* Nota: No modifique el nombre original del archivo .txt descargado,\nya que el sistema lo usa para identificar su ID de usuario.",
            text_color="#E67E22", font=ctk.CTkFont(size=11, slant="italic"), justify="left"
        )
        self.lbl_warning.grid(row=7, column=0, padx=30, pady=(5, 10), sticky="w")
        
        # Botón de Login e Info Error
        self.lbl_error = ctk.CTkLabel(self, text="", text_color="#E74C3C")
        self.lbl_error.grid(row=8, column=0, padx=30, pady=5)
        
        self.btn_login = ctk.CTkButton(self, text="Conectar", font=ctk.CTkFont(weight="bold"), fg_color="#2ECC71", hover_color="#27AE60", text_color_disabled="white", command=self._on_login_click)
        self.btn_login.grid(row=9, column=0, padx=30, pady=(5, 20), sticky="ew")
        
        # Load saved data
        self._load_saved_credentials()
        
        # Trap focus
        self.grab_set()

    def _browse_file(self):
        import shutil
        filename = filedialog.askopenfilename(
            title="Seleccionar archivo de seguridad de Clickedu",
            filetypes=[("Archivos de texto", "*.txt"), ("Todos los archivos", "*.*")]
        )
        if filename:
            try:
                base_dir = os.path.dirname(os.path.dirname(os.path.dirname(__file__)))
                internal_dir = os.path.join(base_dir, "security_internal")
                os.makedirs(internal_dir, exist_ok=True)
                
                short_name = os.path.basename(filename)
                internal_path = os.path.join(internal_dir, short_name)
                
                if os.path.abspath(filename) != os.path.abspath(internal_path):
                    shutil.copy2(filename, internal_path)
                    
                self.security_file_path = internal_path
                self.lbl_file_path.configure(text=short_name, text_color="white")
            except Exception as e:
                self._show_error(f"Error copiando archivo de seguridad: {e}")

    def _load_saved_credentials(self):
        try:
            if os.path.exists(self.config_path):
                with open(self.config_path, "r", encoding="utf-8") as f:
                    config = json.load(f)
                    
                    if "clickedu_user" in config:
                        self.entry_user.insert(0, config["clickedu_user"])
                    if "clickedu_pwd" in config:
                        self.entry_pwd.insert(0, config["clickedu_pwd"])
                    if "clickedu_sec_file" in config and os.path.exists(config["clickedu_sec_file"]):
                        self.security_file_path = config["clickedu_sec_file"]
                        self.lbl_file_path.configure(text=os.path.basename(self.security_file_path), text_color="white")
        except Exception as e:
            print("Error cargando credenciales guardadas:", e)

    def _save_credentials(self, username, password, sec_file):
        try:
            config = {}
            if os.path.exists(self.config_path):
                with open(self.config_path, "r", encoding="utf-8") as f:
                    config = json.load(f)
                    
            config["clickedu_user"] = username
            config["clickedu_pwd"] = password
            config["clickedu_sec_file"] = sec_file
            
            with open(self.config_path, "w", encoding="utf-8") as f:
                json.dump(config, f)
        except Exception as e:
            print("Error guardando credenciales:", e)

    def _on_login_click(self):
        username = self.entry_user.get().strip()
        password = self.entry_pwd.get()
        sec_file = self.security_file_path
        
        if not username or not password or not sec_file:
            self.lbl_error.configure(text="Por favor, rellene todos los campos.")
            return
            
        self.lbl_error.configure(text="")
        self._set_loading_state(True)
        
        # Ejecutar en hilo para no bloquear UI
        threading.Thread(target=self._login_worker, args=(username, password, sec_file), daemon=True).start()

    def _login_worker(self, username, password, sec_file):
        try:
            # Esta función de callback debe devolver True en éxito o fallar con excepción
            self.login_callback(username, password, sec_file)
            
            # Si tiene éxito, guardar las credenciales y cerrar el popup
            self._save_credentials(username, password, sec_file)
            self.after(0, self.destroy)
        except Exception as e:
            self.after(0, lambda err=e: self._show_error(str(err)))

    def _show_error(self, message):
        self._set_loading_state(False)
        self.lbl_error.configure(text=message)

    def _set_loading_state(self, is_loading):
        if is_loading:
            self.btn_login.configure(state="disabled", text="Conectando...")
            self.entry_user.configure(state="disabled")
            self.entry_pwd.configure(state="disabled")
            self.btn_browse.configure(state="disabled")
        else:
            self.btn_login.configure(state="normal", text="Conectar")
            self.entry_user.configure(state="normal")
            self.entry_pwd.configure(state="normal")
            self.btn_browse.configure(state="normal")
