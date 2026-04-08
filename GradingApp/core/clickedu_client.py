import os
import re
import requests
from urllib.parse import urljoin, urlparse, parse_qs
from bs4 import BeautifulSoup

class ClickeduClient:
    BASE_URL = "https://calasanz-pamplona-escolapiosemaus.clickedu.eu"
    LOGIN_PATH = "/user.php?action=doLogin"
    LOGIN_PAGE = "/user.php?action=login"
    CONTROL_PATH = "/user.php?action=controlArxiuPas"
    SUBJECTS_PATH = "/go/Index.php?ctr=Subjects/Subjects"

    def __init__(self):
        self.session = requests.Session()
        self.is_authenticated = False

    def extract_hidden_fields(self, html):
        """Extrae campos hidden y botones de un HTML usando BeautifulSoup"""
        soup = BeautifulSoup(html, "html.parser")
        fields = {}
        for inp in soup.find_all("input"):
            name = inp.get("name")
            type_attr = (inp.get("type") or "").lower()
            if not name or type_attr == "file": continue
            fields[name] = inp.get("value", "")
        for btn in soup.find_all("button"):
            name = btn.get("name")
            if name: fields[name] = btn.get("value", "")
        return fields

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

        # Extraemos los campos necesarios dinámicamente del HTML de respuesta del login
        # (incluye id_usuari, tokens de cookies, etc.)
        fields = self.extract_hidden_fields(resp_login.text)
        
        if 'id_usuari' not in fields:
            # Si no está el id_usuari en el HTML, es probable que las credenciales fueran incorrectas
            # o el servidor haya respondido con un error distinto.
            if "Incorrecto" in resp_login.text or "usrKO" in resp_login.url:
                 raise Exception("Usuario o contraseña incorrectos.")
            raise Exception("No se pudo detectar el ID de usuario en la respuesta de Clickedu.")

        filename = os.path.basename(security_file_path)

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

    def get_subjects(self):
        url = urljoin(self.BASE_URL, self.SUBJECTS_PATH)
        headers = {"Referer": self.BASE_URL, "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0"}
        resp = self.session.get(url, headers=headers, timeout=30)
        soup = BeautifulSoup(resp.text, 'html.parser')
        
        lista_materias = []
        bloques = soup.find_all('div', class_='bloc-assignatura')
        
        for bloque in bloques:
            enlace = bloque.find('a')
            if not enlace: continue
            parsed_url = urlparse(enlace.get('href', ''))
            id_asignatura = parse_qs(parsed_url.query).get('id', [None])[0]
            if not id_asignatura: continue
                
            div_title = bloque.find('div', class_='title')
            div_subtitle = bloque.find('div', class_='sub-title')
            titulo = div_title.text.strip() if div_title else "Desconocido"
            curso = div_subtitle.text.strip() if div_subtitle else "Desconocido"
            
            lista_materias.append({"id_asignatura": id_asignatura, "nombre": titulo, "curso": curso})
            
        return lista_materias

    def get_evaluations(self, id_asignatura):
        """Extrae las evaluaciones disponibles (1ª, 2ª, 3ª) de la asignatura"""
        url = urljoin(self.BASE_URL, f"/go/Index.php?ctr=Subject/Items&do=Index&id={id_asignatura}")
        headers = {"Referer": self.BASE_URL, "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0"}
        resp = self.session.get(url, headers=headers, timeout=30)
        soup = BeautifulSoup(resp.text, 'html.parser')
        
        evaluaciones = []
        select_trm = soup.find('select', {'name': 'trm'})
        if select_trm:
            for option in select_trm.find_all('option'):
                val = option.get('value')
                text = option.text.strip()
                if val and val != 'all':
                    evaluaciones.append({"id": val, "nombre": text})
        return evaluaciones

    def get_evaluation_items(self, id_asignatura):
        """Extrae todos los ítems de la asignatura con su evaluación asociada"""
        url = urljoin(self.BASE_URL, f"/go/Index.php?ctr=Subject/Items&do=Index&id={id_asignatura}")
        headers = {"Referer": self.BASE_URL, "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0"}
        resp = self.session.get(url, headers=headers, timeout=30)
        soup = BeautifulSoup(resp.text, 'html.parser')
        
        items_encontrados = []
        
        for a in soup.find_all('a', class_='popupEvent'):
            url_popup = a.get('url', '')
            if 'ctr=Subject/Item/Item&do=See' in url_popup:
                parsed_url = urlparse(url_popup)
                id_item = parse_qs(parsed_url.query).get('it', [None])[0]
                if not id_item: continue
                
                nombre_item = a.get('popuptitle', 'Desconocido').strip()
                id_evaluacion = "Desconocida"
                celda = a.find_parent('td')
                if celda:
                    edit_link = celda.find('a', href=lambda h: h and 'do=Edit' in h)
                    if edit_link:
                        edit_parsed = urlparse(edit_link['href'])
                        id_evaluacion = parse_qs(edit_parsed.query).get('trm', ['Desconocida'])[0]

                if not any(i['id_item'] == id_item for i in items_encontrados):
                    items_encontrados.append({
                        "id_item": id_item,
                        "id_evaluacion": id_evaluacion,
                        "nombre": nombre_item
                    })
        return items_encontrados

    def subir_notas_excel(self, id_asignatura, id_evaluacion, id_criterio, ruta_excel):
        url_post = urljoin(self.BASE_URL, f"/assignatures/importar_notes_items.php?accio=carregar_plantilla&assig={id_asignatura}&id_aval={id_evaluacion}")
        datos_formulario = {"id_criteri_avaluacio": str(id_criterio), "BotoGenerar": "Generar"}
        
        try:
            with open(ruta_excel, "rb") as f:
                archivos = {"v_arxiu_importar": (os.path.basename(ruta_excel), f, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")}
                resp_post = self.session.post(url_post, data=datos_formulario, files=archivos, allow_redirects=True, timeout=30)
                
            if resp_post.status_code != 200:
                raise Exception(f"Error al subir el Excel. HTTP Status: {resp_post.status_code}")
        except FileNotFoundError:
            raise FileNotFoundError(f"No se encuentra el archivo: {ruta_excel}")
        except Exception as e:
            raise Exception(f"Error en la petición de subida: {str(e)}")

        url_lista = urljoin(self.BASE_URL, f"/assignatures/importar_notes_items.php?assig={id_asignatura}&id_aval={id_evaluacion}")
        resp_lista = self.session.get(url_lista, timeout=30)

        soup = BeautifulSoup(resp_lista.text, 'html.parser')
        
        enlaces_importar = []
        for a in soup.find_all('a'):
            href = a.get('href', '')
            if 'accio=importar_notes' in href:
                enlaces_importar.append(href)
                
        if not enlaces_importar:
            raise Exception("No se encontró el enlace para importar en el servidor tras la carga. Formato de Excel inválido.")
            
        enlace_importar = enlaces_importar[-1]
        id_archivo = parse_qs(urlparse(enlace_importar).query).get('id', [None])[0]
            
        url_importar = urljoin(self.BASE_URL, f"/assignatures/{enlace_importar}")
        resp_importar = self.session.get(url_importar, allow_redirects=True, timeout=30)
        
        if resp_importar.status_code != 200:
            raise Exception("Fallo en la URL final de importación de Clickedu.")
            
        url_eliminar = urljoin(self.BASE_URL, f"/assignatures/importar_notes_items.php?accio=esborrar_arxiu&assig={id_asignatura}&id_aval={id_evaluacion}&id={id_archivo}")
        self.session.get(url_eliminar, allow_redirects=True, timeout=30)
            
        return True
