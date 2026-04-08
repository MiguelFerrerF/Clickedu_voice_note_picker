import os
import re
import requests
from urllib.parse import urljoin

class ClickeduClient:
    BASE_URL = "https://calasanz-pamplona-escolapiosemaus.clickedu.eu"
    LOGIN_PATH = "/user.php?action=doLogin"
    LOGIN_PAGE = "/user.php?action=login"
    CONTROL_PATH = "/user.php?action=controlArxiuPas"

    def __init__(self):
        self.session = requests.Session()
        self.is_authenticated = False

    def login(self, username, password, security_file_path):
        """
        Inicia sesión validando usuario/contraseña y el archivo de seguridad.
        Lanza excepciones en caso de que alguna parte del proceso falle.
        """
        # --- PASO 1: USERNAME & PASSWORD ---
        headers_login = {
            "Content-Type": "application/x-www-form-urlencoded",
            "Origin": self.BASE_URL,
            "Referer": urljoin(self.BASE_URL, self.LOGIN_PAGE),
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0"
        }

        data_login = {
            "username": username,
            "password": password,
            "button": "Inicia"
        }

        try:
            resp_login = self.session.post(
                urljoin(self.BASE_URL, self.LOGIN_PATH),
                headers=headers_login,
                data=data_login,
                allow_redirects=True,
                timeout=30
            )
            # Verificamos si realmente redirigió y entró al modo controlArxiuPas o similar, 
            # pero por ahora seguimos estrictamente el script del usuario.
        except Exception as e:
            raise Exception(f"Fallo de conexión al intentar el login: {e}")

        # --- PASO 2: SECURITY FILE ---
        if not os.path.exists(security_file_path):
            raise FileNotFoundError(f"No existe el archivo de seguridad: {security_file_path}")

        # Extraemos el ID del usuario directamente del nombre del archivo
        filename = os.path.basename(security_file_path)
        match = re.search(r'\d+', filename)
        if not match:
            raise ValueError(f"No se pudo encontrar un ID numérico en el nombre del archivo: {filename}.\nPor favor, no renombre el archivo de seguridad bajado de Clickedu.")
        
        id_usuario_dinamico = match.group()

        fields = {
            'id_usuari': id_usuario_dinamico,
            'MAX_FILE_SIZE': '100000',
            'button': 'Aceptar',
            'cookiescheckdumb': '1',
            'cookiescheckanalytics': '1'
        }

        headers_file = {
            "Origin": self.BASE_URL,
            "Referer": urljoin(self.BASE_URL, self.LOGIN_PATH),
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0"
        }

        try:
            with open(security_file_path, "rb") as f:
                files = {
                    "userfile": (filename, f, "application/octet-stream")
                }

                resp_file = self.session.post(
                    urljoin(self.BASE_URL, self.CONTROL_PATH),
                    headers=headers_file,
                    data=fields,
                    files=files,
                    allow_redirects=True,
                    timeout=30
                )
        except Exception as e:
            raise Exception(f"Fallo de red al enviar archivo de seguridad: {e}")

        if "Archivo de seguridad INCORRECTO" in resp_file.text or "usrKO" in resp_file.url:
            self.is_authenticated = False
            raise Exception("El servidor ha rechazado el usuario, contraseña o archivo de seguridad indicando que son INCORRECTOS.")

        self.is_authenticated = True
        return True
