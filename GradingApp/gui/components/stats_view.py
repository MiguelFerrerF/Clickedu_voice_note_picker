import customtkinter as ctk

class StatsView(ctk.CTkScrollableFrame):
    def __init__(self, master, stats, df_last_column, **kwargs):
        super().__init__(master, fg_color="transparent", **kwargs)
        
        self.grid_columnconfigure((0, 1), weight=1)
        
        header = ctk.CTkLabel(self, text="Resumen de Rendimiento de la Clase", font=ctk.CTkFont(size=24, weight="bold"))
        header.grid(row=0, column=0, columnspan=2, pady=(0, 20), sticky="w")
        
        # --- CÁLCULO DE MÉTRICAS ---
        # Eficiencia
        ts = stats['grading_timestamps']
        total_time_str = "0s"
        speed_str = "0s"
        mins_saved = "0"
        burst_mode = 0
        
        if len(ts) >= 2:
            total_sec = ts[-1] - ts[0]
            m, s = divmod(int(total_sec), 60)
            total_time_str = f"{m}m {s}s"
            
            avg_sec = total_sec / (len(ts) - 1)
            speed_str = f"{avg_sec:.1f}s"
            
            clickedu_estimated_sec = len(ts) * 15
            saved_sec = max(0, clickedu_estimated_sec - total_sec)
            mins_saved = f"{int(saved_sec // 60)}"
            
            current_burst = 1
            for i in range(1, len(ts)):
                if ts[i] - ts[i-1] <= 10:
                    current_burst += 1
                    burst_mode = max(burst_mode, current_burst)
                else:
                    current_burst = 1
        
        # Rendimiento Escolar
        raw_grades = []
        if df_last_column is not None:
            for val in df_last_column.dropna():
                try:
                    raw_grades.append(float(val))
                except ValueError:
                    pass
        
        mean_grade = sum(raw_grades)/len(raw_grades) if raw_grades else 0
        sorted_grades = sorted(raw_grades)
        median_grade = sorted_grades[len(sorted_grades)//2] if raw_grades else 0
        
        aprobados = sum(1 for g in raw_grades if g >= 5.0)
        success_rate = (aprobados / len(raw_grades) * 100) if raw_grades else 0
        
        # Curiosidades
        mic_total = stats['voice_attempts']
        mic_success = stats['voice_successes']
        mic_accuracy = (mic_success / mic_total * 100) if mic_total > 0 else 0
        
        # Distribución para Campana
        dist = {"Insuficiente (<5)": 0, "Suficiente (5-6)": 0, "Bien (6-7)": 0, "Notable (7-9)": 0, "Sobresaliente (9-10)": 0}
        for g in raw_grades:
            if g < 5: dist["Insuficiente (<5)"] += 1
            elif g < 6: dist["Suficiente (5-6)"] += 1
            elif g < 7: dist["Bien (6-7)"] += 1
            elif g < 9: dist["Notable (7-9)"] += 1
            else: dist["Sobresaliente (9-10)"] += 1
        max_dist = max(dist.values()) if raw_grades else 1

        from collections import Counter
        repeated_grade = Counter(raw_grades).most_common(1)[0][0] if raw_grades else "N/A"
        unpronounceable = stats['lowest_match']
        
        # --- CONSTRUCCIÓN DE UI VISUAL ---
        card_metrics = ctk.CTkFrame(self, fg_color=("gray85", "gray16"), corner_radius=15)
        card_metrics.grid(row=1, column=0, padx=15, pady=15, sticky="nsew")
        ctk.CTkLabel(card_metrics, text="⚡ Eficiencia", font=ctk.CTkFont(size=20, weight="bold")).pack(pady=(15, 10))
        
        ctk.CTkLabel(card_metrics, text=speed_str, font=ctk.CTkFont(size=44, weight="bold"), text_color="#3498DB").pack(pady=(10,0))
        ctk.CTkLabel(card_metrics, text="Segundos por nota", font=ctk.CTkFont(size=12, slant="italic")).pack()
        
        ctk.CTkLabel(card_metrics, text=f"{burst_mode} 🚀", font=ctk.CTkFont(size=32, weight="bold"), text_color="#E67E22").pack(pady=(15,0))
        ctk.CTkLabel(card_metrics, text="Modo Ráfaga máxima", font=ctk.CTkFont(size=12, slant="italic")).pack()
        
        ctk.CTkLabel(card_metrics, text=f"{mins_saved} min", font=ctk.CTkFont(size=32, weight="bold"), text_color="#2ECC71").pack(pady=(15,0))
        ctk.CTkLabel(card_metrics, text="Ahorro estimado total", font=ctk.CTkFont(size=12, slant="italic")).pack(pady=(0, 20))

        card_school = ctk.CTkFrame(self, fg_color=("gray85", "gray16"), corner_radius=15)
        card_school.grid(row=1, column=1, padx=15, pady=15, sticky="nsew")
        ctk.CTkLabel(card_school, text="🎓 Examen", font=ctk.CTkFont(size=20, weight="bold")).pack(pady=(15, 10))
        
        means_frame = ctk.CTkFrame(card_school, fg_color="transparent")
        means_frame.pack(fill="x", pady=10)
        means_frame.grid_columnconfigure((0,1), weight=1)
        
        ctk.CTkLabel(means_frame, text=f"{mean_grade:.2f}", font=ctk.CTkFont(size=38, weight="bold"), text_color="#F1C40F").grid(row=0, column=0)
        ctk.CTkLabel(means_frame, text="Media", font=ctk.CTkFont(size=12)).grid(row=1, column=0)
        
        ctk.CTkLabel(means_frame, text=f"{median_grade:.2f}", font=ctk.CTkFont(size=38, weight="bold"), text_color="#9B59B6").grid(row=0, column=1)
        ctk.CTkLabel(means_frame, text="Mediana", font=ctk.CTkFont(size=12)).grid(row=1, column=1)
        
        ctk.CTkLabel(card_school, text=f"Éxito: {success_rate:.1f}% Aprobados", font=ctk.CTkFont(weight="bold")).pack(pady=(10, 5))
        pass_bar = ctk.CTkProgressBar(card_school, height=12, progress_color="#2ECC71", fg_color="#E74C3C")
        pass_bar.pack(fill="x", padx=30, pady=(0, 15))
        pass_bar.set(success_rate / 100)
        
        dist_frame = ctk.CTkFrame(card_school, fg_color="transparent")
        dist_frame.pack(fill="x", padx=20, pady=(0,20))
        colors = {"Insuficiente (<5)": "#E74C3C", "Suficiente (5-6)": "#E67E22", "Bien (6-7)": "#F1C40F", "Notable (7-9)": "#3498DB", "Sobresaliente (9-10)": "#2ECC71"}
        for i, (k, v) in enumerate(dist.items()):
            ctk.CTkLabel(dist_frame, text=k, font=ctk.CTkFont(size=11), width=110, anchor="w").grid(row=i, column=0, padx=(0,10))
            bar = ctk.CTkProgressBar(dist_frame, height=8, progress_color=colors[k], fg_color="gray30")
            bar.grid(row=i, column=1, sticky="ew")
            bar.set(v / max_dist if max_dist > 0 else 0)
            ctk.CTkLabel(dist_frame, text=str(v), font=ctk.CTkFont(size=11, weight="bold")).grid(row=i, column=2, padx=(10,0))
        dist_frame.grid_columnconfigure(1, weight=1)

        card_fun = ctk.CTkFrame(self, fg_color=("gray85", "gray16"), corner_radius=15)
        card_fun.grid(row=2, column=0, columnspan=2, padx=15, pady=15, sticky="nsew")
        ctk.CTkLabel(card_fun, text="🎙️ Voz", font=ctk.CTkFont(size=20, weight="bold")).pack(pady=(15, 10))
        
        mic_frame = ctk.CTkFrame(card_fun, fg_color="transparent")
        mic_frame.pack(fill="x", pady=10)
        mic_frame.grid_columnconfigure((0,1), weight=1)
        
        ctk.CTkLabel(mic_frame, text=f"{mic_accuracy:.1f}%", font=ctk.CTkFont(size=38, weight="bold"), text_color="#1ABC9C").grid(row=0, column=0)
        ctk.CTkLabel(mic_frame, text=f"Tasa Aciertos ({mic_success}/{mic_total})", font=ctk.CTkFont(size=12)).grid(row=1, column=0)
        
        unpron_name = unpronounceable['name'] if unpronounceable['name'] else "Ninguno"
        unpron_score = unpronounceable['score']
        ctk.CTkLabel(mic_frame, text=f"🗣️ {unpron_name}", font=ctk.CTkFont(size=24, weight="bold", slant="italic"), text_color="#e74c3c").grid(row=0, column=1)
        ctk.CTkLabel(mic_frame, text=f"Alumno Difícil ({unpron_score:.1f}% match)", font=ctk.CTkFont(size=12)).grid(row=1, column=1)
        
        ctk.CTkLabel(card_fun, text=f"Moda (Nota + repetida): {repeated_grade}", font=ctk.CTkFont(weight="bold", size=14)).pack(pady=(10, 20))
