import flet as ft
from dataclasses import dataclass

@dataclass
class BrandColors:
    c50 = "#eef9f9"
    c100 = "#d4eff0"
    c200 = "#aee0e2"
    c300 = "#7bcad0"
    c400 = "#4cb0b7"
    c500 = "#1a939a" # Primario
    c600 = "#16797f"
    c700 = "#136167"
    c800 = "#114f54"
    c900 = "#0e4247"

@dataclass
class SlateColors:
    c50 = "#f9fafb" # Light Base
    c100 = "#f3f4f6"
    c200 = "#e5e7eb"
    c300 = "#d1d5db"
    c400 = "#9ca3af"
    c500 = "#6b7280"
    c600 = "#4b5563"
    c700 = "#3f3f46"
    c800 = "#383838" # Dark Card
    c900 = "#2d2d2d" # Dark Base

class Theme:
    brand = BrandColors()
    slate = SlateColors()
    
    # Fuentes base
    font_family = "Inter"

    # Constantes genéricas
    border_radius_sm = 4
    border_radius_md = 6
    border_radius_lg = 8
    border_radius_xl = 12
    border_radius_2xl = 16
    border_radius_3xl = 24
    border_radius_full = 9999

    # Sombras Tailwind Flet
    shadow_sm = ft.BoxShadow(blur_radius=2, spread_radius=0, color="#0D000000", offset=ft.Offset(0, 1))
    shadow_md = ft.BoxShadow(blur_radius=6, spread_radius=-1, color="#1A000000", offset=ft.Offset(0, 4))
    shadow_lg = ft.BoxShadow(blur_radius=15, spread_radius=-3, color="#1A000000", offset=ft.Offset(0, 10))
    shadow_2xl = ft.BoxShadow(blur_radius=50, spread_radius=-12, color="#40000000", offset=ft.Offset(0, 25))

    @staticmethod
    def is_dark(page: ft.Page) -> bool:
        return page.theme_mode == ft.ThemeMode.DARK

    @staticmethod
    def get_bg_color(page: ft.Page) -> str:
        return Theme.slate.c900 if Theme.is_dark(page) else Theme.slate.c50

    @staticmethod
    def get_card_bg(page: ft.Page) -> str:
        return Theme.slate.c800 if Theme.is_dark(page) else ft.Colors.WHITE

    @staticmethod
    def get_text_main(page: ft.Page) -> str:
        return ft.Colors.WHITE if Theme.is_dark(page) else Theme.slate.c900

    @staticmethod
    def get_text_secondary(page: ft.Page) -> str:
        return Theme.slate.c400 if Theme.is_dark(page) else Theme.slate.c500

    @staticmethod
    def get_border_color(page: ft.Page) -> str:
        return Theme.slate.c700 if Theme.is_dark(page) else Theme.slate.c100
