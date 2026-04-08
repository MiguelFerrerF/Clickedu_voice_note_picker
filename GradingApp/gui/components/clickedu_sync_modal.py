import customtkinter as ctk
import os
import threading
from urllib.parse import urljoin

class ClickeduSyncModal(ctk.CTkToplevel):
    def __init__(self, master, clickedu_client, plantillas_dir, on_success_callback, *args, **kwargs):
        super().__init__(master, *args, **kwargs)
        
        self.title("Sincronizar Plantillas Clickedu")
        self.geometry("450x400")
        self.resizable(False, False)
        
        # Centrar la ventana
        self.update_idletasks()
        try:
            x = master.winfo_x() + (master.winfo_width() // 2) - (450 // 2)
            y = master.winfo_y() + (master.winfo_height() // 2) - (400 // 2)
            self.geometry(f"+{x}+{y}")
        except:
            pass

        # Configurar icono
        import sys
        if getattr(sys, 'frozen', False):
            icon_path = os.path.join(sys._MEIPASS, "icon.ico")
        else:
            base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
            icon_path = os.path.join(base_dir, "icon.ico")
            
        if os.path.exists(icon_path):
            self.after(200, lambda: self._safe_set_icon(icon_path))
            
        self.client = clickedu_client
        self.plantillas_dir = plantillas_dir
        self.on_success_callback = on_success_callback
        
        self.materias = []
        self.evaluaciones = []
        self.selected_evaluacion = None
        
        self.grid_columnconfigure(0, weight=1)
        
        self.lbl_title = ctk.CTkLabel(self, text="Sincronización de Plantillas", font=ctk.CTkFont(size=20, weight="bold"))
        self.lbl_title.grid(row=0, column=0, pady=(20, 10))
        
        self.lbl_desc = ctk.CTkLabel(self, text="Selecciona el periodo escolar para descargar las\nplantillas de todas tus asignaturas automáticamente.", justify="center")
        self.lbl_desc.grid(row=1, column=0, pady=(0, 20), padx=30)
        
        # Seleccionador de Evaluación
        self.lbl_eval = ctk.CTkLabel(self, text="1. Selecciona el periodo de evaluación:")
        self.lbl_eval.grid(row=2, column=0, sticky="w", padx=30, pady=(5, 0))
        
        self.combo_eval = ctk.CTkOptionMenu(self, values=["Cargando evaluaciones..."], width=390, command=self._on_eval_select, state="disabled")
        self.combo_eval.grid(row=3, column=0, padx=30, pady=(0, 20))
        
        # Barra de progreso
        self.progress = ctk.CTkProgressBar(self, width=390)
        self.progress.grid(row=4, column=0, padx=30, pady=(10, 5))
        self.progress.set(0)
        
        # Estado
        self.lbl_status = ctk.CTkLabel(self, text="Iniciando conexión...", text_color="gray")
        self.lbl_status.grid(row=5, column=0, padx=30, pady=5)
        
        # Botón de Sincronización
        self.btn_sync = ctk.CTkButton(self, text="Sincronizar ahora", font=ctk.CTkFont(weight="bold"), 
                                      fg_color="#8E44AD", hover_color="#9B59B6", state="disabled", 
                                      command=self._on_sync_click)
        self.btn_sync.grid(row=6, column=0, padx=30, pady=(15, 20), sticky="ew")
        
        self.grab_set()
        
        # Iniciar carga de datos inicial
        threading.Thread(target=self._initial_load_worker, daemon=True).start()

    def _set_status(self, text, is_error=False, is_loading=False):
        color = "#E74C3C" if is_error else ("#F39C12" if is_loading else "#2ECC71")
        self.lbl_status.configure(text=text, text_color=color)

    def _initial_load_worker(self):
        try:
            self.materias = self.client.get_subjects()
            if not self.materias:
                self.after(0, lambda: self._set_status("No se encontraron materias en Clickedu.", is_error=True))
                return
            
            # Cogemos las evaluaciones de la primera materia como referencia global
            self.evaluaciones = self.client.get_evaluations(self.materias[0]['id_asignatura'])
            self.after(0, self._populate_evaluations)
        except Exception as e:
            self.after(0, lambda err=e: self._set_status(f"Error cargando datos: {err}", is_error=True))

    def _populate_evaluations(self):
        if not self.evaluaciones:
            self._set_status("No se encontraron periodos de evaluación.", is_error=True)
            return
            
        values = [e['nombre'] for e in self.evaluaciones]
        self.combo_eval.configure(values=values, state="normal")
        self.combo_eval.set(values[-1] if values else "Seleccionar...") # Marcar la última por defecto (suele ser la actual)
        self.selected_evaluacion = self.evaluaciones[-1] if self.evaluaciones else None
        
        self.btn_sync.configure(state="normal")
        self._set_status("Listo para sincronizar", is_loading=False)

    def _on_eval_select(self, choice):
        idx = self.combo_eval._values.index(choice)
        self.selected_evaluacion = self.evaluaciones[idx]

    def _on_sync_click(self):
        self.btn_sync.configure(state="disabled", text="Sincronizando...")
        self.combo_eval.configure(state="disabled")
        threading.Thread(target=self._sync_worker, daemon=True).start()

    def _sync_worker(self):
        try:
            total = len(self.materias)
            id_eval = self.selected_evaluacion['id']
            
            for i, materia in enumerate(self.materias):
                nombre_limpio = self.client.limpiar_nombre_archivo(materia['nombre'])
                curso_limpio = self.client.limpiar_nombre_archivo(materia['curso'])
                
                # Nombre de archivo estandarizado
                filename = f"{nombre_limpio} - {curso_limpio}.xlsx"
                save_path = os.path.join(self.plantillas_dir, filename)
                
                self.after(0, lambda n=materia['nombre']: self._set_status(f"Descargando: {n}", is_loading=True))
                
                success = self.client.descargar_plantilla_excel(materia['id_asignatura'], id_eval, save_path)
                
                progress_val = (i + 1) / total
                self.after(0, lambda v=progress_val: self.progress.set(v))
                
            self.after(0, self._on_sync_complete)
        except Exception as e:
            self.after(0, lambda err=e: self._set_status(f"Error en descarga: {err}", is_error=True))
            self.after(0, lambda: self.btn_sync.configure(state="normal", text="Reintentar Sincronización"))

    def _on_sync_complete(self):
        self._set_status("¡Sincronización completada!", is_error=False)
        self.after(1000, self.on_success_callback)
        self.after(2000, self.destroy)

    def _safe_set_icon(self, icon_path):
        try:
            self.iconbitmap(icon_path)
        except Exception:
            try:
                from PIL import Image, ImageTk
                png_path = icon_path.replace(".ico", ".png")
                if os.path.exists(png_path):
                    img = Image.open(png_path)
                    self._icon_photo = ImageTk.PhotoImage(img)
                    self.iconphoto(False, self._icon_photo)
            except Exception:
                pass
