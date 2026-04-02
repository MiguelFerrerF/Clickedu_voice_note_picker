import flet as ft
from gui.theme import Theme

class StudentCard(ft.Container):
    def __init__(self, page: ft.Page, student, on_click=None):
        super().__init__()
        self.page = page
        self.student = student
        self.on_click_handler = on_click
        
        self.padding = ft.padding.all(16)
        self.border_radius = Theme.border_radius_2xl
        self.bgcolor = Theme.get_card_bg(self.page)
        self.border = ft.border.all(1, Theme.get_border_color(self.page))
        self.shadow = ft.BoxShadow(blur_radius=2, color=ft.Colors.with_opacity(0.05, ft.Colors.BLACK))
        self.animate = ft.Animation(200, "ease")
        self.on_hover = self.card_hover
        self.on_click = self.handle_click

        # Avatar
        initials = student.name.split(',')[0][:2].upper()
        self.avatar = ft.Container(
            width=40, height=40,
            border_radius=Theme.border_radius_full,
            bgcolor=student.avatar_color_bg,
            alignment=ft.alignment.center,
            content=ft.Text(initials, color=student.avatar_color_text, weight=ft.FontWeight.BOLD, size=14)
        )

        # Name
        self.name_text = ft.Text(
            student.name, 
            size=14, 
            weight=ft.FontWeight.W_500, 
            color=Theme.get_text_main(self.page),
            max_lines=1,
            overflow=ft.TextOverflow.ELLIPSIS
        )

        # Box Nota
        has_grade = bool(student.grade)
        grade_text = student.grade if has_grade else "-"
        grade_color = Theme.get_text_main(self.page) if has_grade else Theme.slate.c400
        grade_weight = ft.FontWeight.BOLD if has_grade else ft.FontWeight.NORMAL
        
        # En Tailwind usan classes tabulares, aquí forzamos mono si es número, pero en web Inter lo maneja bien.
        self.grade_box = ft.Container(
            width=64, height=40,
            bgcolor=Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.slate.c900),
            border_radius=Theme.border_radius_lg,
            border=ft.border.all(1, Theme.slate.c200 if not Theme.is_dark(self.page) else Theme.slate.c700),
            alignment=ft.alignment.center,
            content=ft.Text(grade_text, size=18, weight=grade_weight, color=grade_color, font_family="Consolas" if has_grade else "Inter")
        )

        self.content = ft.Row(
            controls=[
                ft.Row([self.avatar, self.name_text], spacing=16, expand=True),
                self.grade_box
            ],
            alignment=ft.MainAxisAlignment.SPACE_BETWEEN
        )

    def card_hover(self, e):
        is_hovered = e.data == "true"
        
        # Estado activo hover
        if is_hovered:
            self.border = ft.border.all(1, Theme.brand.c200 if not Theme.is_dark(self.page) else Theme.slate.c600)
            self.shadow = ft.BoxShadow(blur_radius=6, spread_radius=-1, color=ft.Colors.with_opacity(0.1, ft.Colors.BLACK))
            self.name_text.color = Theme.brand.c600
        else:
            self.border = ft.border.all(1, Theme.get_border_color(self.page))
            self.shadow = ft.BoxShadow(blur_radius=2, color=ft.Colors.with_opacity(0.05, ft.Colors.BLACK))
            self.name_text.color = Theme.get_text_main(self.page)
        
        self.update()

    def handle_click(self, e):
        if self.on_click_handler:
            self.on_click_handler(self.student)
            
    def trigger_flash(self, new_grade):
        # Actualiza el grade de momento simulado
        has_grade = bool(new_grade)
        grade_color = Theme.get_text_main(self.page) if has_grade else Theme.slate.c400
        self.grade_box.content.value = new_grade if has_grade else "-"
        self.grade_box.content.color = grade_color
        self.grade_box.content.weight = ft.FontWeight.BOLD if has_grade else ft.FontWeight.NORMAL
        
        # Secuencia animada de Success Flash
        # Primero hace pop y cambia color
        orig_bg = self.bgcolor
        self.bgcolor = Theme.brand.c100 if not Theme.is_dark(self.page) else Theme.brand.c800
        self.scale = 1.02
        self.shadow = ft.BoxShadow(blur_radius=15, spread_radius=-3, color=ft.Colors.with_opacity(0.1, ft.Colors.BLACK))
        self.update()
        
        # Timeout return
        def restore():
            self.bgcolor = Theme.get_card_bg(self.page)
            self.scale = 1.0
            self.shadow = ft.BoxShadow(blur_radius=2, color=ft.Colors.with_opacity(0.05, ft.Colors.BLACK))
            self.update()
        
        # Asíncronamente lanzar restore
        import threading
        t = threading.Timer(1.0, restore)
        t.start()
