import customtkinter as ctk
import pandas as pd
import re

class StudentGrid(ctk.CTkScrollableFrame):
    def __init__(self, master, on_grade_update, **kwargs):
        super().__init__(master, **kwargs)
        self.on_grade_update = on_grade_update
        self.current_cols = 1
        
        self.student_widgets = []
        self.entry_list = []
        self.grade_entries = {}
        
        self.placeholder_label = ctk.CTkLabel(self, text="Selecciona un archivo de Excel para comenzar.", text_color="gray")
        self.placeholder_label.pack(pady=50)
        
        # Validación de notas
        self.vcmd = (self.register(self.validate_grade), '%P')
        
    def show_placeholder(self, is_error=False):
        # Ocultar la tabla actual
        self._clear_grid()
        
        msg = "Error al cargar el Excel. Comprueba el formato." if is_error else "Selecciona un archivo de Excel para comenzar."
        color = "red" if is_error else "gray"
        
        self.placeholder_label.configure(text=msg, text_color=color)
        self.placeholder_label.pack(pady=50)

    def populate(self, students_data):
        self.placeholder_label.pack_forget()
        self._clear_grid()
        
        self.grade_entries = {}
        self.entry_list = []
        self.student_widgets = []
        
        self.header_name_1 = ctk.CTkLabel(self, text="Nombre del Alumno", font=ctk.CTkFont(weight="bold"))
        self.header_grade_1 = ctk.CTkLabel(self, text="Nota", font=ctk.CTkFont(weight="bold"))
        self.header_name_2 = ctk.CTkLabel(self, text="Nombre del Alumno", font=ctk.CTkFont(weight="bold"))
        self.header_grade_2 = ctk.CTkLabel(self, text="Nota", font=ctk.CTkFont(weight="bold"))
        
        for i, student in enumerate(students_data):
            lbl = ctk.CTkLabel(self, text=student['Nombre'])
            entry = ctk.CTkEntry(self, width=80, justify="center", validate="key", validatecommand=self.vcmd)
            
            nota = student.get('Nota')
            if pd.notna(nota):
                # Usamos insert(0, ...) pero limpiamos primero por si acaso
                entry.delete(0, 'end')
                entry.insert(0, str(nota))
                
            entry.bind("<KeyRelease>", lambda event, s_id=student['ID'], ent=entry: self._on_grade_change(event, s_id, ent))
            entry.bind("<Return>", lambda event, idx=i: self._focus_next_entry(event, idx))
            entry.bind("<Down>", lambda event, idx=i: self._focus_next_entry(event, idx))
            entry.bind("<Up>", lambda event, idx=i: self._focus_prev_entry(event, idx))
            
            self.grade_entries[student['ID']] = entry
            self.entry_list.append(entry)
            self.student_widgets.append((lbl, entry))
            
        self.resize(self.current_cols, force=True) # fuerza redibujado de las columnas
        
    def _clear_grid(self):
        # Desmontamos en lugar de destruir para CTkScrollableFrame
        if getattr(self, 'student_widgets', None):
            for lbl, entry in self.student_widgets:
                lbl.destroy()
                entry.destroy()
            self.student_widgets = []
            
        # Limpieza segura de cabeceras
        for attr in ['header_name_1', 'header_grade_1', 'header_name_2', 'header_grade_2']:
            if hasattr(self, attr):
                getattr(self, attr).destroy()
            
    def _draw_students_grid(self, cols):
        if not self.student_widgets:
            return
            
        for lbl, entry in self.student_widgets:
            lbl.grid_forget()
            entry.grid_forget()
            
        # Ocultamos cabeceras antes de re-posicionar
        for attr in ['header_name_1', 'header_grade_1', 'header_name_2', 'header_grade_2']:
            if hasattr(self, attr):
                getattr(self, attr).grid_forget()
        
        if cols == 1:
            self.header_name_1.grid(row=0, column=0, padx=10, pady=10, sticky="w")
            self.header_grade_1.grid(row=0, column=1, padx=10, pady=10, sticky="e")
            self.grid_columnconfigure(1, weight=1)
            self.grid_columnconfigure((2,3), weight=0)
            
            for i, (lbl, entry) in enumerate(self.student_widgets):
                lbl.grid(row=i+1, column=0, padx=10, pady=5, sticky="w")
                entry.grid(row=i+1, column=1, padx=10, pady=5, sticky="e")
        else:
            self.header_name_1.grid(row=0, column=0, padx=10, pady=10, sticky="w")
            self.header_grade_1.grid(row=0, column=1, padx=(10, 40), pady=10, sticky="e")
            self.header_name_2.grid(row=0, column=2, padx=10, pady=10, sticky="w")
            self.header_grade_2.grid(row=0, column=3, padx=10, pady=10, sticky="e")
            
            self.grid_columnconfigure(1, weight=1)
            self.grid_columnconfigure(3, weight=1)
            
            mid = (len(self.student_widgets) + 1) // 2
            for i, (lbl, entry) in enumerate(self.student_widgets):
                if i < mid:
                    lbl.grid(row=i+1, column=0, padx=10, pady=5, sticky="w")
                    entry.grid(row=i+1, column=1, padx=(10, 40), pady=5, sticky="e")
                else:
                    lbl.grid(row=(i-mid)+1, column=2, padx=10, pady=5, sticky="w")
                    entry.grid(row=(i-mid)+1, column=3, padx=10, pady=5, sticky="e")
                    
    def resize(self, cols, force=False):
        if force or self.current_cols != cols or not self.winfo_ismapped():
            self.current_cols = cols
            self._draw_students_grid(cols)

    def update_student_grade_ui(self, student_id, grade, text_color=("green", "lightgreen"), reset_delay_ms=2000):
        if student_id in self.grade_entries:
            entry = self.grade_entries[student_id]
            entry.delete(0, 'end')
            entry.insert(0, str(grade))
            entry.configure(text_color=text_color)
            if reset_delay_ms > 0:
                self.after(reset_delay_ms, lambda: entry.configure(text_color=["black", "white"]))

    def validate_grade(self, new_value):
        if new_value == "":
            return True
        if re.fullmatch(r'\d{0,3}([.,]\d{0,1})?', new_value):
            return True
        return False
        
    def _on_grade_change(self, event, student_id, entry_widget):
        val = entry_widget.get().strip()
        try:
            if val == "":
                self.on_grade_update(student_id, None)
                entry_widget.configure(text_color=["black", "white"])
            else:
                val_float = float(val.replace(',', '.'))
                self.on_grade_update(student_id, val_float)
                entry_widget.configure(text_color=["black", "white"])
        except ValueError:
            entry_widget.configure(text_color="red")
            
    def _focus_next_entry(self, event, current_idx):
        if current_idx + 1 < len(self.entry_list):
            self.entry_list[current_idx + 1].focus()
        return "break"
        
    def _focus_prev_entry(self, event, current_idx):
        if current_idx - 1 >= 0:
            self.entry_list[current_idx - 1].focus()
        return "break"
