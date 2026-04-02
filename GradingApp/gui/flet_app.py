import flet as ft
from dataclasses import dataclass
import random

from gui.theme import Theme
from gui.components.header import Header
from gui.components.student_grid import StudentGrid
from gui.components.microphone_dock import MicrophoneDock
from gui.components.toast import NotificationToast
from gui.components.command_palette import CommandPalette
from gui.components.stats_modal import StatsModal

@dataclass
class Student:
    id: int
    name: str
    grade: str
    avatar_color_bg: str
    avatar_color_text: str

class AppState:
    def __init__(self):
        self.students = [
            Student(1, "Alonso Fernández, Javier", "6.5", "#dbeafe", "#1d4ed8"),
            Student(2, "Gómez Ruiz, Elena", "", "#f3e8ff", "#7e22ce"),
            Student(3, "Martínez López, Carlos", "", "#ffedd5", "#c2410c"),
            Student(4, "Zaragoza Vega, Lucía", "9.0", Theme.brand.c100, Theme.brand.c700)
        ]
        self.is_dark_mode = False
        self.is_sound_enabled = True
        self.is_listening = False

def main(page: ft.Page):
    # Configuración de página principal
    page.title = "Clickedu - Minimalismo Zen"
    page.padding = 0
    page.theme_mode = ft.ThemeMode.LIGHT
    page.bgcolor = Theme.get_bg_color(page)
    
    page.fonts = {
        "Inter": "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
    }
    page.theme = ft.Theme(font_family="Inter")
    
    # Estado Global Simulable
    state = AppState()

    # --------------- Referencias Globales ---------------
    def on_student_click(student):
        toast.show(f"Alumno clickeado: {student.name}", type="info")

    def on_stats_click(e):
        stats_modal.show()

    def on_cmd_click(e):
        cmd_palette.show()

    def on_mic_down():
        if state.is_listening: return
        state.is_listening = True
        dock.set_listening(True)
        # Activar blur en el grid
        grid.update_blur_state(True)
        page.update()

    def on_mic_up():
        if not state.is_listening: return
        state.is_listening = False
        dock.set_listening(False)
        # Desactivar blur
        grid.update_blur_state(False)
        page.update()

        # Simula dar nota
        ungraded = [s for s in state.students if not s.grade]
        if ungraded:
            target = random.choice(ungraded)
            target.grade = f"{random.uniform(5, 10):.1f}"
            
            # Notifica la UI
            card = grid.cards_dict.get(target.id)
            if card: card.trigger_flash(target.grade)
            
            toast.show(f"Nota asignada: {target.grade}", type="success")
            
            # Simula TTS
            if state.is_sound_enabled:
                pass # Play local feedback 

    def toggle_theme():
        state.is_dark_mode = not state.is_dark_mode
        page.theme_mode = ft.ThemeMode.DARK if state.is_dark_mode else ft.ThemeMode.LIGHT
        page.bgcolor = Theme.get_bg_color(page)
        
        # Propagar actualización de tema a todos los componentes
        header.update_theme()
        grid.update_theme()
        dock.update_theme()
        cmd_palette.update_theme()
        stats_modal.update_theme()
        page.update()
        
        toast.show("Modo oscuro activado" if state.is_dark_mode else "Modo claro activado")

    def on_cmd_action(action_id):
        if action_id == "toggle_theme":
            toggle_theme()
        elif action_id == "toggle_sound":
            state.is_sound_enabled = not state.is_sound_enabled
            toast.show("Voz activada" if state.is_sound_enabled else "Voz silenciada")
        elif action_id == "upload_cloud":
            toast.show("Sincronizando...", type="cloud")
            import threading
            threading.Timer(2.0, lambda: toast.show("Notas subidas con éxito", type="success")).start()

    # Teclado global
    def on_keyboard(e: ft.KeyboardEvent):
        # Spacebar Mic
        if e.key == " " and not cmd_palette.visible:
            if not state.is_listening: on_mic_down()
            else: on_mic_up() # Trigger release manual si soltamos. En flet KeyboardEvent no distingue raw up/down bien, esto es un tap.
        # Ctrl + K
        if e.key == "K" and (e.ctrl or e.meta):
            if cmd_palette.visible: cmd_palette.hide()
            else: cmd_palette.show()
        # Esc
        if e.key == "Escape":
            if cmd_palette.visible: cmd_palette.hide()
            if stats_modal.visible: stats_modal.hide()

    page.on_keyboard_event = on_keyboard

    # --------------- Construcción de Componentes ---------------
    header = Header(page, state, on_cmd_click, on_stats_click)
    grid = StudentGrid(page, state, on_student_click)
    dock = MicrophoneDock(page, state, on_mic_down, on_mic_up)
    toast = NotificationToast(page)
    cmd_palette = CommandPalette(page, state, on_cmd_action)
    stats_modal = StatsModal(page)

    # Layout de la estructura principal centrada `max-w-5xl px-4`
    main_column = ft.Column(
        controls=[
            ft.Container(content=header, width=1024, padding=ft.padding.symmetric(horizontal=16), alignment=ft.alignment.top_center),
            ft.Container(content=grid, width=1024, expand=True, alignment=ft.alignment.top_center)
        ],
        horizontal_alignment=ft.CrossAxisAlignment.CENTER,
        expand=True
    )

    # Stack permite z-index absoluto
    base_stack = ft.Stack(
        controls=[
            main_column,
            # Floating Dock
            ft.Container(
                content=dock,
                bottom=32,
                left=0,
                right=0,
                alignment=ft.alignment.center
            ),
            # Capas Modales Top Level Z-index
            toast,
            cmd_palette,
            stats_modal
        ],
        expand=True
    )

    page.add(base_stack)
    page.update()

if __name__ == "__main__":
    ft.app(target=main, assets_dir="../assets")
