r"""La prueba de voz de H3a con Google, **solo como referencia** (D2).

Es el motor de la v1: la API web no oficial de `SpeechRecognition`, que puede
cerrarse cualquier día y que manda la voz fuera. En la prueba solo hay
nombres inventados; en la app, la voz no sale del ordenador.

    py -m venv %TEMP%\voz-google
    %TEMP%\voz-google\Scripts\pip install SpeechRecognition
    %TEMP%\voz-google\Scripts\python scripts\probar-voz\google.py [carpeta]

Deja `<carpeta>\resultados\google.json`; luego
`cargo run --release --example probar_voz -- --informe` lo compara.
"""

import ctypes
import json
import sys
import time
from pathlib import Path

import speech_recognition as sr


def documentos() -> Path:
    """La carpeta Documentos de verdad (puede estar en OneDrive)."""
    ruta = ctypes.create_unicode_buffer(260)
    ctypes.windll.shell32.SHGetFolderPathW(None, 5, None, 0, ruta)  # CSIDL_PERSONAL
    return Path(ruta.value)


def main() -> None:
    carpeta = Path(sys.argv[1]) if len(sys.argv) > 1 else documentos() / "Gestor de Notas" / "prueba-voz"
    grabaciones = sorted(carpeta.glob("[0-9][0-9].wav"))
    if not grabaciones:
        sys.exit(f"No hay grabaciones en {carpeta}")
    resultados = carpeta / "resultados"
    resultados.mkdir(exist_ok=True)

    r = sr.Recognizer()
    frases = []
    for g in grabaciones:
        with sr.AudioFile(str(g)) as fuente:
            audio = r.record(fuente)
        inicio = time.perf_counter()
        try:
            texto = r.recognize_google(audio, language="es-ES")
        except sr.UnknownValueError:
            texto = ""
        except sr.RequestError as e:
            texto = f"[error: {e}]"
        ms = round((time.perf_counter() - inicio) * 1000)
        frases.append({"n": int(g.stem), "texto": texto, "ms": ms})
        print(f"\r  google: {len(frases)}/{len(grabaciones)}", end="", flush=True)
    print()
    (resultados / "google.json").write_text(
        json.dumps({"motor": "google", "frases": frases}, ensure_ascii=False, indent=2), encoding="utf-8"
    )


if __name__ == "__main__":
    main()
