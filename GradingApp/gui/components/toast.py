import flet as ft
from gui.theme import Theme
from gui.components.lucide import LucideIcon
import threading

class NotificationToast(ft.Container):
    def __init__(self, page: ft.Page):
        super().__init__()
        self.page = page
        
        # Oculto por defecto
        self.opacity = 0
        self.top = 24
        self.left = 0 # Flet no tiene translate -50% fácil sin width definido, así que podemos centrarlo en un Row global
        self.shadow = ft.BoxShadow(blur_radius=15, color=ft.Colors.with_opacity(0.15, ft.Colors.BLACK))
        self.border_radius = Theme.border_radius_full
        self.animate_opacity = ft.Animation(300, "ease")
        self.visible = False
        
        self.icon_ctl = LucideIcon("info", size=16, color=ft.Colors.WHITE)
        self.text_ctl = ft.Text("Notificación", size=14, weight=ft.FontWeight.W_500, color=ft.Colors.WHITE)
        
        self.content = ft.Row([self.icon_ctl, self.text_ctl], spacing=8, alignment=ft.MainAxisAlignment.CENTER)
        self.padding = ft.padding.symmetric(horizontal=16, vertical=8)

        # Timer
        self._timer = None

    def show(self, message: str, type="info"):
        self.text_ctl.value = message
        
        if type in ["success", "cloud"]:
            self.bgcolor = Theme.brand.c500
            name = "check-circle-2" if type == "success" else "cloud-upload"
            self.icon_ctl.src = f"icons/{name}.svg"
            self.icon_ctl.color = ft.Colors.WHITE
            self.text_ctl.color = ft.Colors.WHITE
            self.padding = ft.padding.symmetric(horizontal=16, vertical=12) # py-3
        else:
            self.bgcolor = Theme.slate.c800 if not Theme.is_dark(self.page) else Theme.slate.c100
            self.icon_ctl.src = "icons/info.svg"
            self.icon_ctl.color = ft.Colors.WHITE if not Theme.is_dark(self.page) else Theme.slate.c900
            self.text_ctl.color = ft.Colors.WHITE if not Theme.is_dark(self.page) else Theme.slate.c900
            self.padding = ft.padding.symmetric(horizontal=16, vertical=8)

        self.visible = True
        self.opacity = 1
        self.update()
        
        # Limpiar timer anterior si existe
        if self._timer:
            self._timer.cancel()
            
        self._timer = threading.Timer(3.0, self.hide)
        self._timer.start()

    def hide(self):
        self.opacity = 0
        self.update()
        # Darle tiempo a la animación antes de hacer visible=False
        def set_invisible():
            self.visible = False
            self.update()
        threading.Timer(0.3, set_invisible).start()
