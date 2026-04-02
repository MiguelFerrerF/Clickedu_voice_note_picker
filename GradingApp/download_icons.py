import os
import urllib.request

icons = [
    "graduation-cap", "search", "command", "bar-chart-2", "download", 
    "mic", "info", "x", "zap", "alert-circle", "moon", "sun", 
    "volume-2", "volume-x", "cloud-upload", "check-circle-2"
]

os.makedirs("assets/icons", exist_ok=True)
base_url = "https://unpkg.com/lucide-static@0.344.0/icons/{}.svg"

for icon in icons:
    path = os.path.join("assets", "icons", f"{icon}.svg")
    print(f"Downloading {icon}.svg...")
    try:
        urllib.request.urlretrieve(base_url.format(icon), path)
    except Exception as e:
        print(f"Failed: {e}")
print("Done!")
