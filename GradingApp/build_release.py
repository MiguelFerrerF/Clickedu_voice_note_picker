import os
import shutil
import subprocess
import json
import urllib.request
import urllib.error
import getpass
import re

# Configuración
REPO_OWNER = "MiguelFerrerF"
REPO_NAME = "Clickedu_voice_note_picker"
ISCC_PATH = r"C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
INNO_SCRIPT = "InnoSetup_Script.iss"
SPEC_FILE = "GradingApp.spec"
OUTPUT_INSTALLER = r"Inno_Output\Instalador_GestorNotas.exe"

def extract_version():
    with open(INNO_SCRIPT, "r", encoding="utf-8") as f:
        for line in f:
            if line.startswith("AppVersion="):
                return line.split("=")[1].strip()
    return "1.0.0"

def extract_release_notes(version):
    changelog_path = os.path.join(os.path.dirname(os.getcwd()), "CHANGELOG.md")
    if not os.path.exists(changelog_path):
        return f"Release v{version}"
        
    notes = []
    capture = False
    with open(changelog_path, "r", encoding="utf-8") as f:
        for line in f:
            if line.startswith(f"## 🚀 Novedades en v{version}"):
                capture = True
                continue
            if capture:
                if line.startswith("## ") or line.startswith("---"):
                    break
                if line.strip():
                    notes.append(line.strip())
                    
    if not notes:
        return f"Release v{version}"
    return "\n".join(notes)

def run_step(step_name, cmd_args):
    print(f"\n--- Ejecutando: {step_name} ---")
    try:
        subprocess.run(cmd_args, check=True)
        print(f"✅ {step_name} completado con éxito.")
    except subprocess.CalledProcessError as e:
        print(f"❌ Error durante {step_name}. Deteniendo el script.")
        exit(1)

def build_app():
    if os.path.exists("build"):
        shutil.rmtree("build")
    if os.path.exists("dist"):
        shutil.rmtree("dist")
    
    run_step("PyInstaller (Compilación del código fuente)", ["pyinstaller", SPEC_FILE, "--noconfirm"])

def build_installer():
    if not os.path.exists(ISCC_PATH):
        print(f"❌ No se encontró Inno Setup en la ruta: {ISCC_PATH}")
        print("Asegúrate de tener instalado Inno Setup 6.")
        exit(1)
        
    run_step("Inno Setup (Creación del instalador .exe)", [ISCC_PATH, INNO_SCRIPT])

def run_git_commands(version):
    repo_root = os.path.dirname(os.getcwd())
    print("\n--- Subiendo código a GitHub ---")
    
    # Add
    run_step("Git Add", ["git", "-C", repo_root, "add", "."])
    
    # Commit (puede fallar si no hay cambios, lo cual es manejable)
    try:
        print(f"\n--- Ejecutando: Git Commit ---")
        subprocess.run(["git", "-C", repo_root, "commit", "-m", f"Lanzamiento de la versión v{version}"], check=True, stdout=subprocess.DEVNULL)
        print("✅ Git Commit completado.")
    except subprocess.CalledProcessError:
        print("⚠️ No había cambios nuevos qué confirmar en Git o falló el commit.")
        
    # Push
    run_step("Git Push", ["git", "-C", repo_root, "push"])

def create_github_release(version, notes, token):
    print("\n--- Conectando con GitHub API ---")
    
    # 1. Crear la Release
    url_release = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases"
    headers = {
        "Accept": "application/vnd.github+json",
        "Authorization": f"Bearer {token}",
        "X-GitHub-Api-Version": "2022-11-28",
        "User-Agent": "Build-Release-Script"
    }
    
    data = {
        "tag_name": f"v{version}",
        "name": f"Release v{version}",
        "body": notes,
        "draft": False,
        "prerelease": False
    }
    
    req = urllib.request.Request(url_release, headers=headers, data=json.dumps(data).encode("utf-8"), method="POST")
    
    try:
        with urllib.request.urlopen(req) as response:
            res_data = json.loads(response.read().decode())
            upload_url = res_data["upload_url"].split("{")[0]
            print(f"✅ Release v{version} creada exitosamente.")
    except urllib.error.HTTPError as e:
        print(f"❌ Error al crear release: {e.code} - {e.read().decode()}")
        exit(1)
        
    # 2. Subir el Archivo (Asset)
    print("\n--- Subiendo Instalador a GitHub ---")
    if not os.path.exists(OUTPUT_INSTALLER):
        print("❌ No se encontró el instalador generado.")
        exit(1)
        
    file_size = os.path.getsize(OUTPUT_INSTALLER)
    # Binary upload header specific for assets
    headers["Content-Type"] = "application/octet-stream"
    headers["Content-Length"] = str(file_size)
    
    upload_url = f"{upload_url}?name=Instalador_GestorNotas.exe"
    
    print("Subiendo archivo... por favor espera.")
    with open(OUTPUT_INSTALLER, "rb") as f:
        req_upload = urllib.request.Request(upload_url, headers=headers, data=f, method="POST")
        try:
            with urllib.request.urlopen(req_upload) as response:
                print(f"✅ Instalador subido correctamente a GitHub!")
        except urllib.error.HTTPError as e:
            print(f"❌ Error al subir el instalador: {e.code} - {e.read().decode()}")

def main():
    print("===========================================")
    print("🚀 SCRIPT DE COMPILACIÓN Y DEPLOY AUTOMÁTICO")
    print("===========================================")
    
    version = extract_version()
    print(f"Versión detectada en InnoSetup: v{version}")
    
    confirm = input("¿Deseas iniciar todo el proceso? (s/n): ")
    if confirm.lower() != 's':
        print("Cancelado.")
        return
        
    print("\n[Fase 1/4] Extrayendo Notas de la Versión...")
    notes = extract_release_notes(version)
    print("-------------------------------------------")
    print(notes)
    print("-------------------------------------------")
    
    token_file = ".github_token"
    token = ""
    if os.path.exists(token_file):
        with open(token_file, "r") as f:
            token = f.read().strip()
            if token:
                print(f"✅ Token cargado desde '{token_file}'")
    
    if not token:
        token = getpass.getpass("\nIntroduce tu GitHub Personal Access Token (PAT): ")
        if token.strip():
            save = input("¿Deseas guardar este token para futuros lanzamientos? (s/n): ")
            if save.lower() == 's':
                with open(token_file, "w") as f:
                    f.write(token.strip())
                print(f"✅ Token guardado en '{token_file}' (asegúrate de que esté en gitignore)")

    if not token.strip():
        print("Token vacío. Se compilará localmente pero se omitirá Git y Release.")
        upload = False
    else:
        upload = True
        
    print("\n[Fase 2/4] Construyendo aplicación...")
    build_app()
    build_installer()
    
    if upload:
        print("\n[Fase 3/4] Guardando en Git...")
        run_git_commands(version)
        print("\n[Fase 4/4] Publicando Release Oficial...")
        create_github_release(version, notes, token)
    else:
        print("\n[Fase 3/4] Git Omitido.")
        print("\n[Fase 4/4] Release Omitida.")
        print("Puedes subir el archivo manualmente situado en 'Inno_Output\\Instalador_GestorNotas.exe'.")

    print("\n🎉 ¡PROCESO FINALIZADO!")

if __name__ == "__main__":
    main()
