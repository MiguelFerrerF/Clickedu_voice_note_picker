import threading
import pyaudio
import speech_recognition as sr
import json
import re
from rapidfuzz import process
import webrtcvad

class VoiceProcessor:
    def __init__(self):
        print("Cargando motor de voz (Google Speech Recognition con PTT)...")
        self.recognizer = sr.Recognizer()
        self.pa = pyaudio.PyAudio()
        self.stream = None
        self.frames = []
        self.is_recording = False
        
        # WebRTC VAD en nivel 3 (Agresivo: Escarta automáticamente silencios y ruido de fondo)
        self.vad = webrtcvad.Vad(3)

    def start_recording(self):
        if self.is_recording:
            return
            
        self.is_recording = True
        self.frames = []
        self.stream = self.pa.open(
            format=pyaudio.paInt16,
            channels=1,
            rate=16000,
            input=True,
            frames_per_buffer=480 # 30ms a 16000Hz (requerido por webrtcvad)
        )
        self.record_thread = threading.Thread(target=self._record_loop)
        self.record_thread.start()

    def _record_loop(self):
        while self.is_recording:
            try:
                # Extraer bloques exactos de 30ms para WebRTC VAD
                data = self.stream.read(480, exception_on_overflow=False)
                if self.vad.is_speech(data, 16000):
                    self.frames.append(data)
            except Exception:
                pass

    def stop_recording_and_process(self, students_list):
        if not self.is_recording:
            return None, None
            
        self.is_recording = False
        if hasattr(self, 'record_thread'):
            self.record_thread.join()
            
        if self.stream:
            self.stream.stop_stream()
            self.stream.close()
            self.stream = None
            
        raw_data = b''.join(self.frames)
        if not raw_data:
            return None, None, 0, None
            
        audio_data = sr.AudioData(raw_data, 16000, 2)
        try:
            texto = self.recognizer.recognize_google(audio_data, language="es-ES")
            print(f"Texto reconocido (Google PTT): {texto}")
            if not texto.strip():
                return None, None, 0, None
            return self._extract_info(texto, students_list)
        except sr.UnknownValueError:
            print("Google no pudo entender el audio.")
            return None, None, 0, None
        except sr.RequestError as e:
            print(f"Error de red con Google: {e}")
            return None, None, 0, None
            
        return self._extract_info(texto, students_list)
            
    def _extract_info(self, text, students_list):
        text = text.lower()
        
        word_to_num = {
            "cero": "0", "uno": "1", "un": "1", "dos": "2", "tres": "3",
            "cuatro": "4", "cinco": "5", "seis": "6", "siete": "7",
            "ocho": "8", "nueve": "9", "diez": "10"
        }
        for word, num in word_to_num.items():
            text = re.sub(rf'\b{word}\b', num, text)
            
        text = re.sub(r'\bcon\b', '.', text)
        text = re.sub(r'\bcoma\b', '.', text)
        text = text.replace("y medio", ".5")
        text = text.replace(",", ".")
        text = re.sub(r'(\d+)\s*\.\s*(\d+)', r'\1.\2', text)
        
        # Regex to find numbers
        numbers = re.findall(r'\d+(?:\.\d+)?', text)
        
        # 1. Quitar el número del final para dejar solo el nombre crudo
        grade = None
        if numbers:
            grade = float(numbers[-1])
            text_without_number = re.sub(r'\d+(?:\.\d+)?\s*(?:puntos|nota)?', '', text).strip()
        else:
            text_without_number = text
            print("No se encontró ningún número claro en el audio.")
            return None, None, 0, None

        if not text_without_number:
            return None, None, 0, None
            
        # 2. Eliminar palabras de relleno (stop words) para purificar el nombre
        stop_words = {"ponle", "pon", "a", "nota", "para", "el", "la", "los", "las", "un", "una", "unos", "unas", "al", "del", "de", "en", "por", "evaluar", "calificar", "tiene", "sacó", "saco", "que", "se", "llama", "alumno", "estudiante"}
        words = text_without_number.split()
        filtered_words = [w for w in words if w not in stop_words]
        clean_text = " ".join(filtered_words)
        
        if not clean_text:
            return None, None, 0, None

        # 3. Matching ultrarrápido con rapidfuzz
        student_names = [s['Nombre'] for s in students_list]
        result = process.extractOne(clean_text, student_names)
        
        if not result: return None, None, 0, None
        
        best_match, score, _ = result
        print(f"Texto limpio VAD: '{clean_text}' | Match: '{best_match}' con certidumbre {score}")
        
        if score > 60:
            for s in students_list:
                if s['Nombre'] == best_match:
                    return s['ID'], grade, score, best_match
                    
        return None, None, score, best_match
