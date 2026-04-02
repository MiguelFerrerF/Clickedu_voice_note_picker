import flet as ft
from gui.theme import Theme
import time, threading

class StatsModal(ft.Container):
    def __init__(self, page: ft.Page):
        super().__init__()
        self.page = page
        
        # Oculto de inicio
        self.visible = False
        self.opacity = 0
        self.expand = True
        self.animate_opacity = ft.Animation(300, "ease")
        self.alignment = ft.alignment.center
        self.bgcolor = ft.Colors.with_opacity(0.3, Theme.slate.c900) if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.7, Theme.slate.c900)
        self.blur = ft.Blur(4, 4, ft.BlurTileMode.CLAMP)
        self.on_click = self.hide

        # Modal
        self.modal = ft.Container(
            width=896, # max-w-4xl
            # height se auto-calcula, si acaso le ponemos padding y maxh
            padding=ft.padding.all(32),
            border_radius=Theme.border_radius_3xl,
            bgcolor=Theme.get_card_bg(self.page),
            border=ft.border.all(1, Theme.get_border_color(self.page)),
            shadow=ft.BoxShadow(blur_radius=50, spread_radius=-12, color=ft.Colors.with_opacity(0.25, ft.Colors.BLACK)),
            scale=0.95,
            animate_scale=ft.Animation(300, "ease"),
            on_click=lambda e: None
        )

        self._build_content()
        self.content = self.modal

    def _build_content(self):
        # Header
        close_btn = ft.Container(
            content=ft.Icon(ft.Icons.CLOSE, size=20, color=Theme.slate.c600 if not Theme.is_dark(self.page) else Theme.slate.c300),
            padding=ft.padding.all(8),
            bgcolor=Theme.slate.c100 if not Theme.is_dark(self.page) else Theme.slate.c700,
            border_radius=Theme.border_radius_full,
            on_click=self.hide
        )
        
        header = ft.Row([
            ft.Column([
                ft.Text("Estadísticas de Sesión", size=24, weight=ft.FontWeight.BOLD, color=Theme.get_text_main(self.page)),
                ft.Text("2º Bachillerato Ciencias", size=14, color=Theme.get_text_secondary(self.page))
            ], spacing=0),
            close_btn
        ], alignment=ft.MainAxisAlignment.SPACE_BETWEEN, vertical_alignment=ft.CrossAxisAlignment.START)

        # Content Grid
        def mk_section_title(icon, text, color):
            return ft.Row([
                ft.Icon(icon, size=16, color=color),
                ft.Text(text, size=14, weight=ft.FontWeight.W_600, color=Theme.slate.c400)
            ], spacing=8)

        # Columna 1: Eficiencia
        col1 = ft.Column([
            mk_section_title(ft.Icons.BOLT, "EFICIENCIA", ft.Colors.AMBER_500),
            ft.Container(
                content=ft.Column([
                    ft.Text("1.8s", size=30, weight=ft.FontWeight.BOLD, color=ft.Colors.AMBER_600),
                    ft.Text("por nota dictada", size=14, weight=ft.FontWeight.W_500, color=ft.Colors.AMBER_700)
                ], spacing=4),
                bgcolor=ft.Colors.AMBER_50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.1, ft.Colors.AMBER_500),
                border=ft.border.all(1, ft.Colors.AMBER_100 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.2, ft.Colors.AMBER_500)),
                padding=ft.padding.all(20),
                border_radius=Theme.border_radius_2xl
            ),
            ft.Row([
                ft.Container(
                    content=ft.Column([ft.Text("15m", size=20, weight=ft.FontWeight.BOLD, color=Theme.get_text_main(self.page)), ft.Text("Ahorro est.", size=12, color=Theme.slate.c500)], spacing=4),
                    padding=ft.padding.all(16), bgcolor=Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.3, Theme.slate.c700), border=ft.border.all(1, Theme.get_border_color(self.page)), border_radius=Theme.border_radius_2xl, expand=True
                ),
                ft.Container(
                    content=ft.Column([ft.Text("24", size=20, weight=ft.FontWeight.BOLD, color=Theme.get_text_main(self.page)), ft.Text("Modo Ráfaga", size=12, color=Theme.slate.c500)], spacing=4),
                    padding=ft.padding.all(16), bgcolor=Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.3, Theme.slate.c700), border=ft.border.all(1, Theme.get_border_color(self.page)), border_radius=Theme.border_radius_2xl, expand=True
                )
            ], spacing=16)
        ], spacing=16)

        # Columna 2: Examen
        # Dummificamos el chart con un simple row de contenedores altos
        col2 = ft.Column([
            mk_section_title(ft.Icons.SCHOOL, "EXAMEN", Theme.brand.c500),
            ft.Row([
                ft.Container(
                    content=ft.Column([ft.Text("7.2", size=24, weight=ft.FontWeight.BOLD, color=Theme.brand.c600), ft.Text("Nota Media", size=12, weight=ft.FontWeight.W_500, color=Theme.brand.c700)], spacing=4),
                    padding=ft.padding.all(16), bgcolor=Theme.brand.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.1, Theme.brand.c500), border=ft.border.all(1, Theme.brand.c100 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.2, Theme.brand.c500)), border_radius=Theme.border_radius_2xl, expand=True
                ),
                ft.Container(
                    content=ft.Column([ft.Text("85%", size=24, weight=ft.FontWeight.BOLD, color=ft.Colors.BLUE_600), ft.Text("Aprobados", size=12, weight=ft.FontWeight.W_500, color=ft.Colors.BLUE_700)], spacing=4),
                    padding=ft.padding.all(16), bgcolor=ft.Colors.BLUE_50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.1, ft.Colors.BLUE_500), border=ft.border.all(1, ft.Colors.BLUE_100 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.2, ft.Colors.BLUE_500)), border_radius=Theme.border_radius_2xl, expand=True
                )
            ], spacing=16),
             ft.Container(
                 content=ft.Column([
                     ft.Row([
                         ft.Container(bgcolor=ft.Colors.with_opacity(0.5, ft.Colors.RED_400), height=30, border_radius=ft.border_radius.only(top_left=6, top_right=6), expand=True),
                         ft.Container(bgcolor=ft.Colors.with_opacity(0.5, ft.Colors.AMBER_400), height=60, border_radius=ft.border_radius.only(top_left=6, top_right=6), expand=True),
                         ft.Container(bgcolor=Theme.brand.c500, height=120, border_radius=ft.border_radius.only(top_left=6, top_right=6), shadow=ft.BoxShadow(blur_radius=15, color=ft.Colors.with_opacity(0.4, Theme.brand.c500)), expand=True),
                         ft.Container(bgcolor=ft.Colors.with_opacity(0.5, ft.Colors.BLUE_400), height=80, border_radius=ft.border_radius.only(top_left=6, top_right=6), expand=True),
                         ft.Container(bgcolor=ft.Colors.with_opacity(0.5, ft.Colors.PURPLE_400), height=40, border_radius=ft.border_radius.only(top_left=6, top_right=6), expand=True),
                     ], alignment=ft.MainAxisAlignment.SPACE_AROUND, vertical_alignment=ft.CrossAxisAlignment.END, height=120, spacing=4),
                     ft.Row([
                         ft.Text("SUS", size=10, color=Theme.slate.c400, expand=True, text_align=ft.TextAlign.CENTER),
                         ft.Text("SUF", size=10, color=Theme.slate.c400, expand=True, text_align=ft.TextAlign.CENTER),
                         ft.Text("BIE", size=10, color=Theme.get_text_main(self.page), weight=ft.FontWeight.BOLD, expand=True, text_align=ft.TextAlign.CENTER),
                         ft.Text("NOT", size=10, color=Theme.slate.c400, expand=True, text_align=ft.TextAlign.CENTER),
                         ft.Text("SOB", size=10, color=Theme.slate.c400, expand=True, text_align=ft.TextAlign.CENTER),
                     ], alignment=ft.MainAxisAlignment.SPACE_AROUND, spacing=4)
                 ], spacing=8),
                 bgcolor=Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.3, Theme.slate.c700), border=ft.border.all(1, Theme.get_border_color(self.page)), padding=24, border_radius=Theme.border_radius_2xl
             )
        ], spacing=16)

        # Columna 3: Voz
        col3 = ft.Column([
            mk_section_title(ft.Icons.MIC, "RECONOCIMIENTO VOZ", ft.Colors.PURPLE_500),
            ft.Container(
                content=ft.Column([
                    ft.Text("98.5%", size=30, weight=ft.FontWeight.BOLD, color=ft.Colors.PURPLE_600),
                    ft.Text("Tasa acierto fonético", size=14, weight=ft.FontWeight.W_500, color=ft.Colors.PURPLE_700)
                ], spacing=4),
                bgcolor=ft.Colors.PURPLE_50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.1, ft.Colors.PURPLE_500),
                border=ft.border.all(1, ft.Colors.PURPLE_100 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.2, ft.Colors.PURPLE_500)),
                padding=ft.padding.all(20),
                border_radius=Theme.border_radius_2xl
            ),
             ft.Container(
                 content=ft.Row([ft.Text("Nota rep.", size=12, weight=ft.FontWeight.BOLD, color=Theme.slate.c500), ft.Text("7.5", size=14, weight=ft.FontWeight.BOLD, color=Theme.get_text_main(self.page))], alignment=ft.MainAxisAlignment.SPACE_BETWEEN),
                 bgcolor=Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.3, Theme.slate.c700), border=ft.border.all(1, Theme.get_border_color(self.page)), padding=ft.padding.symmetric(horizontal=16, vertical=12), border_radius=Theme.border_radius_xl
             ),
             ft.Container(
                 content=ft.Column([
                     ft.Row([ft.Text("Alumno difícil", size=12, weight=ft.FontWeight.BOLD, color=Theme.slate.c500), ft.Icon(ft.Icons.WARNING, size=14, color=ft.Colors.RED_400)], alignment=ft.MainAxisAlignment.SPACE_BETWEEN),
                     ft.Text('"Díaz Carrillo" (5 reintentos)', size=14, weight=ft.FontWeight.W_500, color=Theme.get_text_main(self.page))
                     ], spacing=4),
                 bgcolor=Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.3, Theme.slate.c700), border=ft.border.all(1, Theme.get_border_color(self.page)), padding=ft.padding.symmetric(horizontal=16, vertical=12), border_radius=Theme.border_radius_xl
             )
        ], spacing=16)

        grid = ft.ResponsiveRow([
            ft.Container(content=col1, col={"sm":12, "md":4}),
            ft.Container(content=col2, col={"sm":12, "md":4}),
            ft.Container(content=col3, col={"sm":12, "md":4})
        ], spacing=24)

        self.modal.content = ft.Column([
            header,
            ft.Container(height=32), # spacer
            grid
        ], spacing=0)

    def show(self, e=None):
        self._build_content()
        self.visible = True
        self.update()
        
        self.opacity = 1
        self.modal.scale = 1
        self.update()

    def hide(self, e=None):
        self.opacity = 0
        self.modal.scale = 0.95
        self.update()
        
        def end():
            self.visible = False
            self.update()
        threading.Timer(0.3, end).start()

    def update_theme(self):
        self.bgcolor = ft.Colors.with_opacity(0.3, Theme.slate.c900) if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.7, Theme.slate.c900)
        self.modal.bgcolor = Theme.get_card_bg(self.page)
        self.modal.border = ft.border.all(1, Theme.get_border_color(self.page))
        # Rebuild full content
        self._build_content()
        self.update()
