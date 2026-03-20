import customtkinter as ctk
from gui.app_window import AppWindow

def main():
    ctk.set_appearance_mode("System")  # Modes: system (default), light, dark
    ctk.set_default_color_theme("blue")  # Themes: blue (default), dark-blue, green
    
    app = AppWindow()
    app.mainloop()

if __name__ == "__main__":
    main()
