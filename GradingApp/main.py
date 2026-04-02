from gui.webview_app import WebViewApp
from core.updater import Updater

def main():
    app = WebViewApp()
    
    try:
        import pyi_splash
        pyi_splash.close()
    except Exception:
        pass
        
    # Comprobador de actualizaciones en segundo plano
    # Por ahora lo mantenemos igual, pero WebViewApp necesitará manejar show_update_banner
    updater = Updater("v1.3.3")
    # updater.check_for_updates(app) # Desactivado temporalmente hasta adaptar la UI de update
    
    app.run()

if __name__ == "__main__":
    main()
