import customtkinter as ctk
import os
import threading

class ClickeduUploadModal(ctk.CTkToplevel):
    def __init__(self, master, clickedu_client, current_excel_path, on_success_callback, *args, **kwargs):
        super().__init__(master, *args, **kwargs)
        
        self.title("Subir Notas a Clickedu")
        self.geometry("450x550")
        self.resizable(False, False)
        
        # Centrar la ventana
        self.update_idletasks()
        try:
            x = master.winfo_x() + (master.winfo_width() // 2) - (450 // 2)
            y = master.winfo_y() + (master.winfo_height() // 2) - (550 // 2)
            self.geometry(f"+{x}+{y}")
        except:
            pass

        # Configurar icono
        import sys
        if getattr(sys, 'frozen', False):
            icon_path = os.path.join(sys._MEIPASS, "icon.ico")
        else:
            # Subir 2 niveles desde gui/components/ para llegar a la raíz
            base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
            icon_path = os.path.join(base_dir, "icon.ico")
            
        if os.path.exists(icon_path):
            # Usar un pequeño retraso para asegurar que la ventana esté lista antes de poner el icono
            self.after(200, lambda: self._safe_set_icon(icon_path))
            
        self.client = clickedu_client
        self.current_excel_path = current_excel_path
        self.on_success_callback = on_success_callback
        
        self.materias = []
        self.evaluaciones = []
        self.items = []
        self.selected_materia = None
        self.selected_evaluacion = None
        self.selected_item = None
        
        self.grid_columnconfigure(0, weight=1)
        
        self.lbl_title = ctk.CTkLabel(self, text="Subida de Notas", font=ctk.CTkFont(size=20, weight="bold"))
        self.lbl_title.grid(row=0, column=0, pady=(20, 10))
        
        self.lbl_info = ctk.CTkLabel(self, text=f"Archivo a subir:\n{os.path.basename(self.current_excel_path)}", text_color="#3498DB")
        self.lbl_info.grid(row=1, column=0, pady=(0, 20), padx=20)
        
        # Seleccionador de Asignatura
        self.lbl_asig = ctk.CTkLabel(self, text="1. Selecciona la asignatura:")
        self.lbl_asig.grid(row=2, column=0, sticky="w", padx=30, pady=(5, 0))
        
        self.combo_asig = ctk.CTkOptionMenu(self, values=["Cargando asignaturas..."], width=390, command=self._on_asig_select, state="disabled")
        self.combo_asig.grid(row=3, column=0, padx=30, pady=(0, 15))
        
        # Seleccionador de Evaluación
        self.lbl_eval = ctk.CTkLabel(self, text="2. Selecciona la evaluación:")
        self.lbl_eval.grid(row=4, column=0, sticky="w", padx=30, pady=(5, 0))
        
        self.combo_eval = ctk.CTkOptionMenu(self, values=["Esperando asignatura..."], width=390, command=self._on_eval_select, state="disabled")
        self.combo_eval.grid(row=5, column=0, padx=30, pady=(0, 15))
        
        # Seleccionador de Ítem Evaluativo
        self.lbl_item = ctk.CTkLabel(self, text="3. Selecciona el ítem evaluativo:")
        self.lbl_item.grid(row=6, column=0, sticky="w", padx=30, pady=(5, 0))
        
        self.combo_item = ctk.CTkOptionMenu(self, values=["Esperando evaluación..."], width=390, command=self._on_item_select, state="disabled")
        self.combo_item.grid(row=7, column=0, padx=30, pady=(0, 15))
        
        # Estado/Error
        self.lbl_status = ctk.CTkLabel(self, text="", text_color="#E74C3C")
        self.lbl_status.grid(row=8, column=0, padx=30, pady=10)
        
        # Botón de Subida
        self.btn_upload = ctk.CTkButton(self, text="Subir a Clickedu", font=ctk.CTkFont(weight="bold"), 
                                        fg_color="#3498DB", hover_color="#2980B9", state="disabled", 
                                        command=self._on_upload_click)
        self.btn_upload.grid(row=9, column=0, padx=30, pady=(5, 20), sticky="ew")
        
        self.grab_set()
        
        # Iniciar carga de datos
        threading.Thread(target=self._load_subjects_worker, daemon=True).start()

    def _set_status(self, text, is_error=False, is_loading=False):
        color = "#E74C3C" if is_error else ("#F39C12" if is_loading else "#2ECC71")
        self.lbl_status.configure(text=text, text_color=color)
        
    def _load_subjects_worker(self):
        self.after(0, lambda: self._set_status("Obteniendo asignaturas de Clickedu...", is_loading=True))
        try:
            self.materias = self.client.get_subjects()
            self.after(0, self._populate_subjects)
        except Exception as e:
            self.after(0, lambda err=e: self._set_status(f"Error cargando asignaturas:\n{err}", is_error=True))

    def _populate_subjects(self):
        if not self.materias:
            self._set_status("No se encontraron asignaturas.", is_error=True)
            self.combo_asig.configure(values=["Sin datos"])
            return
            
        values = [f"{m['nombre']} ({m['curso']})" for m in self.materias]
        self.combo_asig.configure(values=["Seleccione una asignatura..."] + values, state="normal")
        self.combo_asig.set("Seleccione una asignatura...")
        self._set_status("", is_error=False)

    def _on_asig_select(self, choice):
        if choice == "Seleccione una asignatura...":
            self._reset_evaluations()
            return
            
        idx = self.combo_asig._values.index(choice) - 1
        self.selected_materia = self.materias[idx]
        
        self.combo_eval.configure(values=["Cargando evaluaciones..."], state="disabled")
        self.combo_eval.set("Cargando evaluaciones...")
        self._reset_items()
        
        threading.Thread(target=self._load_evaluations_worker, args=(self.selected_materia['id_asignatura'],), daemon=True).start()

    def _reset_evaluations(self):
        self.selected_materia = None
        self.selected_evaluacion = None
        self.combo_eval.configure(values=["Esperando asignatura..."], state="disabled")
        self.combo_eval.set("Esperando asignatura...")
        self._reset_items()

    def _load_evaluations_worker(self, id_asig):
        self.after(0, lambda: self._set_status("Obteniendo evaluaciones...", is_loading=True))
        try:
            self.evaluaciones = self.client.get_evaluations(id_asig)
            self.after(0, self._populate_evaluations)
        except Exception as e:
            self.after(0, lambda err=e: self._set_status(f"Error obteniendo evaluaciones:\n{err}", is_error=True))

    def _populate_evaluations(self):
        if not self.evaluaciones:
            self._set_status("No hay evaluaciones disponibles.", is_error=True)
            self.combo_eval.configure(values=["Sin datos"])
            return
            
        values = [e['nombre'] for e in self.evaluaciones]
        self.combo_eval.configure(values=["Seleccione una evaluación..."] + values, state="normal")
        self.combo_eval.set("Seleccione una evaluación...")
        self._set_status("", is_error=False)

    def _on_eval_select(self, choice):
        if choice == "Seleccione una evaluación...":
            self._reset_items()
            return
            
        idx = self.combo_eval._values.index(choice) - 1
        self.selected_evaluacion = self.evaluaciones[idx]
        
        self.combo_item.configure(values=["Cargando ítems evaluativos..."], state="disabled")
        self.combo_item.set("Cargando ítems evaluativos...")
        self.btn_upload.configure(state="disabled")
        
        threading.Thread(target=self._load_items_worker, args=(self.selected_materia['id_asignatura'], self.selected_evaluacion['id']), daemon=True).start()

    def _reset_items(self):
        self.selected_item = None
        self.combo_item.configure(values=["Esperando evaluación..."], state="disabled")
        self.combo_item.set("Esperando evaluación...")
        self.btn_upload.configure(state="disabled")

    def _load_items_worker(self, id_asig, id_eval):
        self.after(0, lambda: self._set_status("Obteniendo ítems evaluativos...", is_loading=True))
        try:
            todos_items = self.client.get_evaluation_items(id_asig)
            self.items = [it for it in todos_items if it['id_evaluacion'] == id_eval]
            self.after(0, self._populate_items)
        except Exception as e:
            self.after(0, lambda err=e: self._set_status(f"Error obteniendo ítems:\n{err}", is_error=True))

    def _populate_items(self):
        if not self.items:
            self._set_status("No hay ítems evaluativos en esta evaluación.", is_error=True)
            self.combo_item.configure(values=["Sin datos"])
            return
            
        values = [it['nombre'] for it in self.items]
        self.combo_item.configure(values=["Seleccione un ítem..."] + values, state="normal")
        self.combo_item.set("Seleccione un ítem...")
        self._set_status("", is_error=False)

    def _on_item_select(self, choice):
        if choice == "Seleccione un ítem...":
            self.selected_item = None
            self.btn_upload.configure(state="disabled")
            return
            
        idx = self.combo_item._values.index(choice) - 1
        self.selected_item = self.items[idx]
        self.btn_upload.configure(state="normal")
        self._set_status("", is_error=False)

    def _on_upload_click(self):
        self.btn_upload.configure(state="disabled", text="Subiendo datos...")
        self._set_status("Ejecutando importación en Clickedu...", is_loading=True)
        self.combo_asig.configure(state="disabled")
        self.combo_eval.configure(state="disabled")
        self.combo_item.configure(state="disabled")
        
        threading.Thread(target=self._upload_worker, daemon=True).start()

    def _upload_worker(self):
        try:
            self.client.subir_notas_excel(
                self.selected_materia['id_asignatura'],
                self.selected_evaluacion['id'],
                self.selected_item['id_item'],
                self.current_excel_path
            )
            self.after(0, self._on_upload_success)
        except Exception as e:
            self.after(0, lambda err=e: self._on_upload_error(str(err)))

    def _on_upload_success(self):
        self._set_status("¡Éxito! Notas importadas correctamente.", is_error=False)
        self.on_success_callback()
        self.after(2000, self.destroy)

    def _on_upload_error(self, message):
        self._set_status(f"Fallo en la importación:\n{message}", is_error=True)
        self.btn_upload.configure(state="normal", text="Subir a Clickedu")
        self.combo_asig.configure(state="normal")
        self.combo_eval.configure(state="normal")
        self.combo_item.configure(state="normal")

    def _safe_set_icon(self, icon_path):
        try:
            self.iconbitmap(icon_path)
        except Exception:
            try:
                # Fallback para otros formatos o si falla iconbitmap
                from PIL import Image, ImageTk
                png_path = icon_path.replace(".ico", ".png")
                if os.path.exists(png_path):
                    img = Image.open(png_path)
                    # Es fundamental guardar la referencia para que no se borre de memoria (Garbage Collection)
                    self._icon_photo = ImageTk.PhotoImage(img)
                    self.iconphoto(False, self._icon_photo)
            except Exception:
                pass
