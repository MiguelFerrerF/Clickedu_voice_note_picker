import flet as ft
from gui.theme import Theme
from gui.components.student_card import StudentCard

class StudentGrid(ft.Container):
    def __init__(self, page: ft.Page, state, on_student_click=None):
        super().__init__()
        self.page = page
        self.state = state
        self.on_student_click = on_student_click
        self.cards_dict = {} # Para actualizar el flash por ID
        
        # Ocupa todo el disponible ancho max de 1024
        self.expand = True
        self.padding = ft.padding.only(left=16, right=16, bottom=128) # Bottom padding para no pisar el Dock
        
        # Título y Contador
        self.title_text = ft.Text("2º Bachillerato Ciencias", size=24, weight=ft.FontWeight.BOLD, color=Theme.get_text_main(self.page))
        self.counter_text = ft.Text(f"{len(self.state.students)} Alumnos", size=14, weight=ft.FontWeight.W_500, color=Theme.get_text_secondary(self.page))
        
        self.header_row = ft.Row(
            [self.title_text, self.counter_text],
            alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
            vertical_alignment=ft.CrossAxisAlignment.END
        )
        self.header_container = ft.Container(content=self.header_row, padding=ft.padding.only(bottom=24))

        # Cuadrícula
        self.grid = ft.ResponsiveRow(
            spacing=16,
            run_spacing=16,
            columns=12
        )
        
        # Renderear lista inicial
        self.build_grid()
        
        # Si dictamos, cambia blur
        self.animate = ft.Animation(300, "ease")
        
        self.content = ft.Column([
            self.header_container,
            self.grid
        ], scroll=ft.ScrollMode.HIDDEN)

    def build_grid(self):
        self.grid.controls.clear()
        self.cards_dict.clear()
        
        for student in self.state.students:
            card = StudentCard(self.page, student, self.on_student_click)
            # En móvil (sm) ocupa las 12 cols, en pantallas medianas (md) ocupa 6.
            wrapper = ft.Container(content=card, col={"sm": 12, "md": 6})
            self.grid.controls.append(wrapper)
            self.cards_dict[student.id] = card
            
        self.update()

    def update_blur_state(self, is_listening):
        # Cuando está escuchando, baja opacity y mete blur temporalmente
        if is_listening:
            self.opacity = 0.3
            self.blur = ft.Blur(2, 2, ft.BlurTileMode.CLAMP)
        else:
            self.opacity = 1.0
            self.blur = None
        self.update()

    def update_theme(self):
        self.title_text.color = Theme.get_text_main(self.page)
        self.counter_text.color = Theme.get_text_secondary(self.page)
        # Forzar recarga de cards
        self.build_grid()
        self.update()
