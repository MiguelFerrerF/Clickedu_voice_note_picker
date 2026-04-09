import os

class AudioEngine:
    def __init__(self):
        self.is_recording = False

    def start_recording(self):
        """Inicia la captura de audio."""
        self.is_recording = True
        print("Grabación iniciada...")

    def stop_recording_and_process(self):
        """Detiene la grabación y procesa el audio a texto."""
        self.is_recording = False
        print("Grabación detenida. Procesando...")
        # Lógica de SpeechRecognition / Whisper aquí
        return "Texto reconocido"
