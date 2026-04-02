import flet as ft

def LucideIcon(name: str, size: float = 24, color: str = None) -> ft.Image:
    """Helper to render downloaded Lucide SVG icons using Flet Image recoloring."""
    return ft.Image(
        src=f"icons/{name}.svg",
        color=color,
        width=size,
        height=size,
        fit=ft.ImageFit.CONTAIN
    )
