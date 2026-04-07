import threading
import pyaudio
import speech_recognition as sr
import json
import re
from rapidfuzz import process, fuzz, utils
import webrtcvad

class VoiceProcessor:
    def __init__(self):
        print("Cargando motor de voz (Google Speech Recognition con PTT)...")
        self.recognizer = sr.Recognizer()
        self.pa = pyaudio.PyAudio()
        self.stream = None
        self.frames = []
        self.is_recording = False
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
            frames_per_buffer=480 # 30ms a 16000Hz
        )
        self.record_thread = threading.Thread(target=self._record_loop)
        self.record_thread.start()

    def _normalize_phonetics(self, text):
        if not text: return ""
        text = text.lower()
        replacements = {
            'á': 'a', 'é': 'e', 'í': 'i', 'ó': 'o', 'ú': 'u',
            'v': 'b', 'h': '', 'y': 'i', 'll': 'i',
            'k': 'c', 'z': 's'
        }
        for k, v in replacements.items():
            text = text.replace(k, v)
        text = re.sub(r'c([ei])', r's\1', text)
        text = re.sub(r'qu([ei])', r'c\1', text)
        text = re.sub(r'g([ei])', r'j\1', text)
        return text.strip()

    def _record_loop(self):
        while self.is_recording:
            try:
                # Bloques para PyAudio
                data = self.stream.read(480, exception_on_overflow=False)
                self.frames.append(data)
            except Exception:
                pass

    def stop_recording(self):
        if not self.is_recording:
            return None
            
        self.is_recording = False
        if hasattr(self, 'record_thread'):
            self.record_thread.join()
            
        if self.stream:
            self.stream.stop_stream()
            self.stream.close()
            self.stream = None
            
        raw_data = b''.join(self.frames)
        if not raw_data:
            return None
            
        return sr.AudioData(raw_data, 16000, 2)

    def process_audio(self, audio_data, students_list):
        try:
            texto = self.recognizer.recognize_google(audio_data, language="es-ES")
            print(f"Texto reconocido (Google PTT): {texto}")
            if not texto.strip():
                return "ERROR"
            return self._extract_info(texto, students_list)
        except sr.UnknownValueError:
            print("Google no pudo entender el audio.")
            return "ERROR"
        except sr.RequestError as e:
            print(f"Error de red con Google: {e}")
            return "ERROR"
            
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
        
        clean_text = clean_text.replace('.', '').replace(',', '').strip()
        
        if not clean_text:
            return None, None, 0, None

        # 3. Matching ultrarrápido fonético con rapidfuzz
        phonetic_students = {s['ID']: self._normalize_phonetics(s['Nombre']) for s in students_list}
        clean_phonetic = self._normalize_phonetics(clean_text)
        
        result = process.extractOne(clean_phonetic, phonetic_students, scorer=fuzz.token_set_ratio, processor=utils.default_process)
        
        if not result: return None, None, 0, None
        
        best_phonetic_match, score, student_id = result
        best_match_original = next(s['Nombre'] for s in students_list if s['ID'] == student_id)
        
        print(f"Texto fonético VAD: '{clean_phonetic}' | Match fonético: '{best_phonetic_match}' ({best_match_original}) con certidumbre {score}")
        
        # Umbral bajado al 45% porque la limpieza exhaustiva fonética evita falsos positivos
        if score >= 45:
            return student_id, grade, score, best_match_original
                    
        return None, None, score, best_match_original
