import flet as ft
from gui.theme import Theme
import threading
import time

class MicrophoneDock(ft.Container):
    def __init__(self, page: ft.Page, state, on_mic_down, on_mic_up):
        super().__init__()
        self.page = page
        self.state = state
        self.on_mic_down = on_mic_down
        self.on_mic_up = on_mic_up
        self._pulse_active = False

        self.padding = ft.padding.all(8)
        self.border_radius = Theme.border_radius_full
        
        # Simulación de Glassmorphism
        bg_col = ft.Colors.with_opacity(0.8, ft.Colors.WHITE if not Theme.is_dark(self.page) else Theme.slate.c800)
        border_col = ft.Colors.with_opacity(0.2, ft.Colors.WHITE if not Theme.is_dark(self.page) else Theme.slate.c700)
        self.bgcolor = bg_col
        self.border = ft.border.all(1, border_col)
        self.blur = ft.Blur(10, 10, ft.BlurTileMode.CLAMP)
        self.shadow = ft.BoxShadow(blur_radius=25, spread_radius=-12, color=ft.Colors.with_opacity(0.25, ft.Colors.BLACK))
        
        self.animate = ft.Animation(300, "ease")
        self.animate_scale = ft.Animation(300, "bounceOut")

        # Texto Izquierdo
        kbd_pill = ft.Container(
            content=ft.Text("Espacio", size=12, color=Theme.slate.c500),
            padding=ft.padding.symmetric(horizontal=8, vertical=4),
            bgcolor=Theme.slate.c100 if not Theme.is_dark(self.page) else Theme.slate.c700,
            border_radius=4
        )
        self.left_col = ft.Container(
            content=ft.Row([ft.Text("Mantén", size=14, color=Theme.get_text_secondary(self.page)), kbd_pill], spacing=4, alignment=ft.MainAxisAlignment.CENTER),
            width=128,
            alignment=ft.alignment.center
        )
        # Red warning left text (oculto por defecto)
        self.left_col_listening = ft.Container(
            content=ft.Text("Escuchando...", size=14, weight=ft.FontWeight.BOLD, color=ft.Colors.RED_500),
            width=128,
            alignment=ft.alignment.center,
            visible=False
        )

        # Texto Derecho
        self.right_col = ft.Container(
            content=ft.Text("para dictar", size=14, color=Theme.get_text_secondary(self.page), weight=ft.FontWeight.W_500),
            width=128,
            alignment=ft.alignment.center
        )

        # Controles Centro: Anillo de pulso (absoluto trasero) y botón
        self.pulse_ring = ft.Container(
            width=64, height=64,
            border_radius=Theme.border_radius_full,
            bgcolor=ft.Colors.RED_500,
            opacity=0,
            scale=0.8,
            animate_scale=ft.Animation(1500, "easeOutCubic"),
            animate_opacity=ft.Animation(1500, "easeOutCubic"),
            top=0, left=0
        )

        self.mic_btn = ft.Container(
            width=64, height=64,
            border_radius=Theme.border_radius_full,
            bgcolor=Theme.slate.c900 if not Theme.is_dark(self.page) else Theme.brand.c500,
            alignment=ft.alignment.center,
            content=ft.Icon(ft.Icons.MIC, color=ft.Colors.WHITE, size=24),
            shadow=ft.BoxShadow(blur_radius=15, spread_radius=-3, color=ft.Colors.with_opacity(0.1, ft.Colors.BLACK)),
            animate=ft.Animation(200, "ease"),
            animate_scale=ft.Animation(200, "ease")
        )

        # Haremos un GestureDetector para mouse down/up simulando espacio
        self.detector = ft.GestureDetector(
            content=ft.Stack([
                self.pulse_ring,
                self.mic_btn
            ], width=64, height=64),
            on_pan_start=self._handle_down,    # Sustituyes mousedown
            on_pan_end=self._handle_up,        # Sustituyes mouseup
            on_hover=self._handle_hover,
            mouse_cursor=ft.MouseCursor.CLICK
        )

        # Stack de textos
        left_stack = ft.Stack([self.left_col, self.left_col_listening], width=128, height=24)

        self.content = ft.Row(
            controls=[
                left_stack,
                self.detector,
                self.right_col
            ],
            spacing=8,
            vertical_alignment=ft.CrossAxisAlignment.CENTER
        )

    def _handle_down(self, e):
        if self.on_mic_down: self.on_mic_down()

    def _handle_up(self, e):
        if self.on_mic_up: self.on_mic_up()

    def _handle_hover(self, e):
        if not self.state.is_listening:
            self.mic_btn.scale = 1.05 if e.data == "true" else 1.0
            self.mic_btn.update()

    def set_listening(self, active: bool):
        if active:
            # Estado activo
            self.scale = 1.05
            self.mic_btn.bgcolor = ft.Colors.RED_500
            self.mic_btn.scale = 1.0 # resetea hover
            self.left_col.visible = False
            self.left_col_listening.visible = True
            self._pulse_active = True
            threading.Thread(target=self._run_pulse, daemon=True).start()
        else:
            self.scale = 1.0
            self.mic_btn.bgcolor = Theme.slate.c900 if not Theme.is_dark(self.page) else Theme.brand.c500
            self.left_col.visible = True
            self.left_col_listening.visible = False
            self._pulse_active = False
            # Frena ring
            self.pulse_ring.opacity = 0
            self.pulse_ring.scale = 0.8
            self.pulse_ring.update()

        self.update()

    def _run_pulse(self):
        while self._pulse_active:
            # Reseteo silente
            self.pulse_ring.animate_scale = None
            self.pulse_ring.animate_opacity = None
            self.pulse_ring.scale = 0.8
            self.pulse_ring.opacity = 0.8
            self.pulse_ring.update()
            
            time.sleep(0.05) # pequeño respiro para que flet trague el update
            
            if not self._pulse_active: break
            
            # Animar
            self.pulse_ring.animate_scale = ft.Animation(1500, "easeOutCubic")
            self.pulse_ring.animate_opacity = ft.Animation(1500, "easeOutCubic")
            self.pulse_ring.scale = 1.5
            self.pulse_ring.opacity = 0.0
            self.pulse_ring.update()
            
            time.sleep(1.5)

    def update_theme(self):
        self.bgcolor = ft.Colors.with_opacity(0.8, ft.Colors.WHITE if not Theme.is_dark(self.page) else Theme.slate.c800)
        self.border = ft.border.all(1, ft.Colors.with_opacity(0.2, ft.Colors.WHITE if not Theme.is_dark(self.page) else Theme.slate.c700))
        self.left_col.content.controls[0].color = Theme.get_text_secondary(self.page)
        self.left_col.content.controls[1].bgcolor = Theme.slate.c100 if not Theme.is_dark(self.page) else Theme.slate.c700
        self.right_col.content.color = Theme.get_text_secondary(self.page)
        if not self.state.is_listening:
            self.mic_btn.bgcolor = Theme.slate.c900 if not Theme.is_dark(self.page) else Theme.brand.c500
        self.update()
