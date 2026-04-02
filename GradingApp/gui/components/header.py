import flet as ft
from gui.theme import Theme
from gui.components.lucide import LucideIcon

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
            shadow=Theme.shadow_md,
            content=LucideIcon("graduation-cap", size=20, color=ft.Colors.WHITE)
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
                shadow = Theme.shadow_sm
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
                LucideIcon("search", size=16, color=Theme.slate.c500),
                ft.Text("Comandos", size=14, weight=ft.FontWeight.W_500, color=Theme.slate.c500),
                cmd_kbd
            ], spacing=8),
            bgcolor=ft.Colors.with_opacity(0.5, Theme.slate.c200) if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.slate.c800),
            padding=ft.padding.symmetric(horizontal=12, vertical=6),
            border_radius=Theme.border_radius_lg,
            on_click=self.on_cmd_click,
            on_hover=self.cmd_hover,
            animate=ft.Animation(200, "ease")
        )

        # Botones Cíclicos
        self.stats_btn = ft.Container(
            content=LucideIcon("bar-chart-2", size=20, color=Theme.slate.c600 if not Theme.is_dark(self.page) else Theme.slate.c300),
            padding=ft.padding.all(8),
            border_radius=Theme.border_radius_full,
            on_click=self.on_stats_click,
            on_hover=lambda e: self._icon_hover(e)
        )
        
        self.export_btn = ft.Container(
            content=LucideIcon("download", size=20, color=ft.Colors.WHITE),
            bgcolor=Theme.brand.c500,
            padding=ft.padding.all(8),
            border_radius=Theme.border_radius_full,
            alignment=ft.alignment.center,
            shadow=Theme.shadow_sm,
            on_hover=lambda e: self._download_hover(e)
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
        idl_col = ft.Colors.with_opacity(0.5, Theme.slate.c200) if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.slate.c800)
        e.control.bgcolor = hov_col if e.data == "true" else idl_col
        e.control.update()

    def _icon_hover(self, e):
        e.control.bgcolor = Theme.slate.c200 if e.data == "true" else ft.Colors.TRANSPARENT
        e.control.update()

    def _download_hover(self, e):
        e.control.bgcolor = Theme.brand.c600 if e.data == "true" else Theme.brand.c500
        e.control.update()

    def update_theme(self):
        self.logo_text.color = Theme.get_text_main(self.page)
        self.stats_btn.content.color = Theme.slate.c600 if not Theme.is_dark(self.page) else Theme.slate.c300
        
        # Reset cmd_btn colors correctly
        self.cmd_btn.bgcolor = ft.Colors.with_opacity(0.5, Theme.slate.c200) if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.slate.c800)
        cmd_kbd = self.cmd_btn.content.controls[2]
        cmd_kbd.bgcolor = Theme.slate.c100 if not Theme.is_dark(self.page) else Theme.slate.c900
        cmd_kbd.border = ft.border.all(1, Theme.slate.c300 if not Theme.is_dark(self.page) else Theme.slate.c600)
        
        self.update()
