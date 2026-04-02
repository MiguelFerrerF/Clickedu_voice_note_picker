import flet as ft
from gui.theme import Theme

class Header(ft.Container):
    def __init__(self, page: ft.Page, state, on_cmd_click, on_stats_click):
        super().__init__()
        self.page = page
        self.state = state
        self.on_cmd_click = on_cmd_click
        self.on_stats_click = on_stats_click
        
        # Estilos extraídos de design_system.md
        self.padding = ft.padding.only(left=16, right=16, top=24, bottom=16)
        
        # --- LEFT BLOCK (Logo) ---
        self.logo_box = ft.Container(
            width=36,
            height=36,
            bgcolor=Theme.brand.c500,
            border_radius=Theme.border_radius_xl,
            alignment=ft.alignment.center,
            shadow=ft.BoxShadow(spread_radius=0, blur_radius=6, color=ft.Colors.with_opacity(0.1, ft.Colors.BLACK)), # shadow-md approx
            content=ft.Icon(ft.Icons.SCHOOL, color=ft.Colors.WHITE, size=20)
        )
        # TODO: Responsive hide for text-xl on mobile
        self.logo_text = ft.Text("Clickedu", size=20, weight=ft.FontWeight.BOLD, color=Theme.get_text_main(self.page))
        
        self.left_block = ft.Row([self.logo_box, self.logo_text], spacing=12)

        # --- CENTER BLOCK (Tabs) ---
        def create_tab(text, is_active=False):
            if is_active:
                bg = Theme.get_card_bg(self.page)
                text_col = Theme.get_text_main(self.page)
                border = ft.border.all(1, Theme.get_border_color(self.page))
                weight = ft.FontWeight.W_600
                shadow = ft.BoxShadow(blur_radius=2, color=ft.Colors.with_opacity(0.05, ft.Colors.BLACK))
            else:
                bg = ft.Colors.TRANSPARENT
                text_col = Theme.get_text_secondary(self.page)
                border = None
                weight = ft.FontWeight.W_500
                shadow = None

            return ft.Container(
                content=ft.Text(text, size=14, weight=weight, color=text_col),
                padding=ft.padding.symmetric(horizontal=16, vertical=8),
                bgcolor=bg,
                border_radius=Theme.border_radius_full,
                border=border,
                shadow=shadow,
                on_hover=lambda e: self.tab_hover(e, is_active),
                animate=ft.Animation(200, "ease")
            )

        self.tabs = ft.Row(
            controls=[
                create_tab("1º ESO A"),
                create_tab("2º Bach Ciencias", is_active=True),
                create_tab("3º ESO C"),
            ],
            spacing=8,
            scroll=ft.ScrollMode.HIDDEN
        )
        self.center_block = ft.Container(content=self.tabs, padding=ft.padding.symmetric(horizontal=8), expand=True)

        # --- RIGHT BLOCK (Actions) ---
        # Botón Comandos
        cmd_kbd = ft.Container(
            content=ft.Text("Ctrl K", size=12, color=Theme.slate.c400),
            padding=ft.padding.symmetric(horizontal=6, vertical=2),
            bgcolor=Theme.slate.c100 if not Theme.is_dark(self.page) else Theme.slate.c900,
            border_radius=4,
            border=ft.border.all(1, Theme.slate.c300 if not Theme.is_dark(self.page) else Theme.slate.c600)
        )
        
        self.cmd_btn = ft.Container(
            content=ft.Row([
                ft.Icon(ft.Icons.SEARCH, size=16, color=Theme.slate.c500),
                ft.Text("Comandos", size=14, weight=ft.FontWeight.W_500, color=Theme.slate.c500),
                cmd_kbd
            ], spacing=8),
            bgcolor=Theme.slate.c200 + "80", # 50% opacity
            padding=ft.padding.symmetric(horizontal=12, vertical=6),
            border_radius=Theme.border_radius_lg,
            on_click=self.on_cmd_click,
            on_hover=self.cmd_hover,
            animate=ft.Animation(200, "ease")
        )

        # Botones Cíclicos
        self.stats_btn = ft.IconButton(
            icon=ft.Icons.BAR_CHART,
            icon_color=Theme.slate.c500,
            on_click=self.on_stats_click
        )
        
        self.export_btn = ft.Container(
            content=ft.Icon(ft.Icons.DOWNLOAD, color=ft.Colors.WHITE, size=20),
            bgcolor=Theme.brand.c500,
            width=36, height=36,
            border_radius=Theme.border_radius_full,
            alignment=ft.alignment.center,
            shadow=ft.BoxShadow(blur_radius=2, color=ft.Colors.with_opacity(0.1, ft.Colors.BLACK))
        )

        self.right_block = ft.Row([self.cmd_btn, self.stats_btn, self.export_btn], spacing=8)

        # Ensamblar Container Row
        self.content = ft.Row(
            controls=[
                self.left_block,
                self.center_block,
                self.right_block
            ],
            alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
            vertical_alignment=ft.CrossAxisAlignment.CENTER
        )

    def tab_hover(self, e, is_active):
        if not is_active:
            e.control.bgcolor = Theme.slate.c200 if e.data == "true" else ft.Colors.TRANSPARENT
            e.control.update()

    def cmd_hover(self, e):
        # En light mode pasa al slate.c200 100%, en dark mode a slate.c700
        hov_col = Theme.slate.c200 if not Theme.is_dark(self.page) else Theme.slate.c700
        idl_col = Theme.slate.c200 + "80" if not Theme.is_dark(self.page) else Theme.slate.c800 + "80"
        e.control.bgcolor = hov_col if e.data == "true" else idl_col
        e.control.update()

    def update_theme(self):
        self.logo_text.color = Theme.get_text_main(self.page)
        # TODO: Refrescar tabs si es necesario (el activo cambiará de fondo negro/blanco según el tema)
        self.update()
