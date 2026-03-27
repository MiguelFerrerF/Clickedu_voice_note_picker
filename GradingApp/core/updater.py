import os
import requests
import threading
import subprocess
import tkinter.messagebox as messagebox
import customtkinter as ctk

class Updater:
    def __init__(self, current_version):
        # Current app version
        self.current_version = current_version
        self.api_url = "https://api.github.com/repos/MiguelFerrerF/Clickedu_voice_note_picker/releases/latest"

    def check_for_updates(self, parent_window):
        # Lanzar comprobación en segundo plano para no bloquear (congelar) el arranque
        def run_check():
            try:
                response = requests.get(self.api_url, timeout=5)
                if response.status_code == 200:
                    data = response.json()
                    latest_version = data.get("tag_name", "")
                    
                    if latest_version and latest_version != self.current_version:
                        # Extraer versión numérica (ej: 'v1.2.0' -> [1, 2, 0])
                        v_curr = [int(x) for x in self.current_version.replace('v', '').split('.')]
                        v_lat = [int(x) for x in latest_version.replace('v', '').split('.')]
                        
                        # Si es numéricamente superior:
                        if v_lat > v_curr:
                            download_url = None
                            file_size = 0
                            
                            for asset in data.get('assets', []):
                                if asset['name'] == 'Instalador_GestorNotas.exe':
                                    download_url = asset['browser_download_url']
                                    file_size = asset['size']
                                    break
                            
                            if download_url:
                                parent_window.after(1000, lambda: parent_window.show_update_banner(latest_version, download_url, file_size, self))
            except Exception as e:
                print(f"Búsqueda de actualizaciones fallida temporalmente: {e}")

        threading.Thread(target=run_check, daemon=True).start()

    def show_update_popup(self, root, latest_version, download_url, file_size):
        respuesta = messagebox.askyesno(
            "Actualización disponible",
            f"Hay una nueva versión de la aplicación ({latest_version}).\n\n¿Quieres descargarla e instalarla ahora?",
            parent=root
        )
        
        if respuesta:
            self.show_download_progress(root, download_url, file_size)

    def show_download_progress(self, root, download_url, file_size):
        # Usar Toplevel nativo de CTk para encajar con el diseño
        progress_window = ctk.CTkToplevel(root)
        progress_window.title("Actualizando...")
        progress_window.geometry("350x120")
        progress_window.resizable(False, False)
        progress_window.attributes('-topmost', True) # Mantener arriba
        
        ctk.CTkLabel(progress_window, text="Descargando la nueva versión, por favor espera...", font=("Inter", 12)).pack(pady=15)
        
        # Barra de progreso nativa moderna (ya no necesitamos ttk)
        progress_bar = ctk.CTkProgressBar(progress_window, orientation="horizontal", width=300)
        progress_bar.pack(pady=5)
        progress_bar.set(0) # Iniciar vacía
        
        hilo_descarga = threading.Thread(
            target=self.download_and_install, 
            args=(download_url, file_size, progress_bar, root, progress_window)
        )
        hilo_descarga.start()

    def download_and_install_inline(self, download_url, file_size, progress_bar, root):
        temp_dir = os.environ.get('TEMP', os.path.expanduser('~'))
        installer_path = os.path.join(temp_dir, 'Instalador_GestorNotas_Actualizado.exe')
        
        try:
            respuesta = requests.get(download_url, stream=True)
            respuesta.raise_for_status()
            
            descargado = 0
            
            with open(installer_path, 'wb') as archivo:
                for chunk in respuesta.iter_content(chunk_size=1024 * 256):
                    if not chunk: continue
                    archivo.write(chunk)
                    descargado += len(chunk)
                    
                    if file_size > 0:
                        progreso = descargado / file_size
                        root.after(0, progress_bar.set, progreso)
            
            print("Descarga completada. Ejecutando instalador en segundo plano...")
            subprocess.Popen([installer_path], shell=True)
            root.after(500, lambda: os._exit(0))
            
        except Exception as e:
            root.after(0, lambda: messagebox.showerror("Error", f"Error de descarga:\n{e}", parent=root))
            root.after(100, lambda: root.hide_update_banner() if hasattr(root, 'hide_update_banner') else None)
