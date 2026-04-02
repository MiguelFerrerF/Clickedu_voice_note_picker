import flet as ft
from gui.theme import Theme
from gui.components.lucide import LucideIcon

class CommandPalette(ft.Container):
    def __init__(self, page: ft.Page, state, on_action):
        super().__init__()
        self.page = page
        self.state = state
        self.on_action = on_action
        
        # Oculto de inicio
        self.visible = False
        self.opacity = 0
        self.expand = True # Ocupa todo el stack
        self.animate_opacity = ft.Animation(200, "easeOut")

        # Fondo con blur
        self.bgcolor = ft.Colors.with_opacity(0.4, Theme.slate.c900)
        self.blur = ft.Blur(4, 4, ft.BlurTileMode.CLAMP)
        self.padding = ft.padding.only(top=self.page.window.height * 0.15 if self.page.window and self.page.window.height else 100, left=16, right=16)
        self.alignment = ft.alignment.top_center
        self.on_click = self._handle_bg_click

        # Input Search
        self.search_input = ft.TextField(
            hint_text="Busca un comando...",
            border=ft.InputBorder.NONE,
            color=Theme.get_text_main(self.page),
            bgcolor=ft.Colors.TRANSPARENT,
            text_size=18,
            cursor_color=Theme.brand.c500,
            selection_color=ft.Colors.with_opacity(0.3, Theme.brand.c500),
            expand=True
        )
        
        esc_kb = ft.Container(
            content=ft.Text("ESC", size=12, color=Theme.slate.c400),
            padding=ft.padding.symmetric(horizontal=8, vertical=4),
            bgcolor=Theme.slate.c100 if not Theme.is_dark(self.page) else Theme.slate.c700,
            border_radius=4
        )

        search_header = ft.Container(
            content=ft.Row([
                LucideIcon("search", size=20, color=Theme.slate.c400),
                self.search_input,
                esc_kb
            ], spacing=12),
            padding=ft.padding.all(12),
            border=ft.border.only(bottom=ft.border.BorderSide(1, Theme.get_border_color(self.page)))
        )

        # Actions list
        self.actions_col = ft.Column(spacing=4, tight=True)
        self._build_actions()
        
        actions_scroll = ft.Container(
            content=self.actions_col,
            padding=ft.padding.all(8),
            height=300
        )
        
        # Modal body
        self.modal = ft.Container(
            content=ft.Column([
                search_header,
                actions_scroll,
                ft.Container(
                    content=ft.Row([
                        ft.Text("Usa el ratón o el buscador", size=12),
                        ft.Text("Clickedu Pro Mode", size=12)
                    ], alignment=ft.MainAxisAlignment.SPACE_BETWEEN),
                    padding=ft.padding.symmetric(horizontal=16, vertical=8),
                    bgcolor=Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.slate.c800),
                    border=ft.border.only(top=ft.border.BorderSide(1, Theme.get_border_color(self.page)))
                )
            ]),
            bgcolor=Theme.get_card_bg(self.page),
            width=672,
            border_radius=Theme.border_radius_2xl,
            border=ft.border.all(1, Theme.get_border_color(self.page)),
            shadow=Theme.shadow_2xl,
            scale=0.95,
            animate_scale=ft.Animation(200, "easeOut"),
            offset=ft.Offset(0, -0.05),
            animate_offset=ft.Animation(200, "easeOut"),
            on_click=lambda e: None # Detener propagación
        )

        self.content = self.modal

    def _build_actions(self):
        self.actions_col.controls.clear()
        
        # Titulo
        self.actions_col.controls.append(ft.Container(
            content=ft.Text("AJUSTES RÁPIDOS", size=12, weight=ft.FontWeight.W_600, color=Theme.slate.c400),
            padding=ft.padding.symmetric(horizontal=12, vertical=8)
        ))
        
        # Crear item Theme
        theme_icon = "sun" if Theme.is_dark(self.page) else "moon"
        self.actions_col.controls.append(self._create_action_item(
            "Alternar Modo Oscuro / Claro",
            "Activar",
            theme_icon,
            "toggle_theme"
        ))
        
        # Crear item Sonido
        sound_icon = "volume-2" if self.state.is_sound_enabled else "volume-x"
        sound_sub = "Actualmente activado" if self.state.is_sound_enabled else "Actualmente silenciado"
        self.actions_col.controls.append(self._create_action_item(
            "Feedback por Voz (Accesibilidad)",
            sound_sub,
            sound_icon,
            "toggle_sound",
            is_brand=True
        ))
        
        # Cloud
        self.actions_col.controls.append(ft.Container(
            content=ft.Text("ACCIONES DE NUBE", size=12, weight=ft.FontWeight.W_600, color=Theme.slate.c400),
            padding=ft.padding.symmetric(horizontal=12, vertical=8)
        ))
        self.actions_col.controls.append(self._create_action_item(
            "Subir notas a Clickedu",
            "Sincroniza la sesión actual con el servidor",
            "cloud-upload",
            "upload_cloud",
            is_cloud=True
        ))

    def _create_action_item(self, title, sub, icon, action_id, is_brand=False, is_cloud=False):
        # Configurar colores base
        icon_bg = Theme.slate.c100 if not Theme.is_dark(self.page) else Theme.slate.c800
        icon_color = Theme.slate.c600 if not Theme.is_dark(self.page) else Theme.slate.c300
        if is_brand or is_cloud:
            icon_color = Theme.brand.c600 if not Theme.is_dark(self.page) else Theme.brand.c400
        if is_cloud:
            icon_bg = Theme.brand.c100 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.brand.c900)

        icon_ctl = ft.Container(
            content=LucideIcon(icon, size=16, color=icon_color),
            padding=ft.padding.all(8),
            bgcolor=icon_bg,
            border_radius=Theme.border_radius_lg,
        )

        title_ctl = ft.Text(title, size=14, weight=ft.FontWeight.W_500, color=Theme.get_text_main(self.page))
        sub_ctl = ft.Text(sub, size=12, color=Theme.slate.c400)

        content_row = ft.Row([
            icon_ctl,
            ft.Column([title_ctl, sub_ctl], spacing=0)
        ])

        container = ft.Container(
            content=content_row,
            padding=ft.padding.all(12),
            border_radius=Theme.border_radius_xl,
            bgcolor=ft.Colors.TRANSPARENT,
            on_hover=lambda e: self._hover_action(e, is_cloud),
            on_click=lambda e: self._execute_action(action_id),
            animate=ft.Animation(150, "ease")
        )
        return container

    def _hover_action(self, e, is_cloud):
        if e.data == "true":
            if is_cloud:
                e.control.bgcolor = Theme.brand.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.2, Theme.brand.c900)
            else:
                e.control.bgcolor = Theme.slate.c100 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.slate.c700)
        else:
            e.control.bgcolor = ft.Colors.TRANSPARENT
        e.control.update()

    def _execute_action(self, action_id):
        self.hide()
        if self.on_action:
            self.on_action(action_id)

    def _handle_bg_click(self, e):
        self.hide()

    def show(self):
        self._build_actions() # Reconstuye para cargar updates de iconos
        self.visible = True
        self.update()
        # Anim in
        self.opacity = 1
        self.modal.scale = 1
        self.modal.offset = ft.Offset(0, 0)
        self.update()
        self.search_input.focus()

    def hide(self):
        self.opacity = 0
        self.modal.scale = 0.95
        self.modal.offset = ft.Offset(0, -0.05)
        self.update()
        
        import threading
        def end():
            self.visible = False
            self.update()
        threading.Timer(0.2, end).start()

    def update_theme(self):
        self.bgcolor = ft.Colors.with_opacity(0.4, Theme.slate.c900)
        self.modal.bgcolor = Theme.get_card_bg(self.page)
        self.modal.border = ft.border.all(1, Theme.get_border_color(self.page))
        self.search_input.color = Theme.get_text_main(self.page)
        self.modal.content.controls[0].border = ft.border.only(bottom=ft.border.BorderSide(1, Theme.get_border_color(self.page)))
        self.modal.content.controls[2].bgcolor = Theme.slate.c50 if not Theme.is_dark(self.page) else ft.Colors.with_opacity(0.5, Theme.slate.c800)
        self.modal.content.controls[2].border = ft.border.only(top=ft.border.BorderSide(1, Theme.get_border_color(self.page)))
        self._build_actions()
        self.update()
