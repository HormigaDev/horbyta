# Para ejecutar este script es necesario instalar como dependencia de desarrollo
# Pillow, opencv-python y numpy
# pip install Pillow opencv-python numpy

import os
import sys
import cv2
import numpy as np
from PIL import Image
from typing import List, Tuple, Optional

SCRIPT_DIR: str = os.path.dirname(os.path.abspath(__file__))
INPUT_IMAGE_NAME: str = os.path.abspath(os.path.join(SCRIPT_DIR, '..', '..', 'frontend', 'public', 'logo.png'))
OUTPUT_DIR: str = os.path.abspath(os.path.join(SCRIPT_DIR, '..', '..', "frontend", "public", "icons"))

PNG_SIZES: List[Tuple[str, int]] = [
    ("apple-icon-120x120.png", 120),
    ("apple-icon-152x152.png", 152),
    ("apple-icon-167x167.png", 167),
    ("apple-icon-180x180.png", 180),
    ("favicon-16x16.png", 16),
    ("favicon-32x32.png", 32),
    ("favicon-96x96.png", 96),
    ("favicon-128x128.png", 128),
    ("icon-128x128.png", 128),
    ("icon-192x192.png", 192),
    ("icon-256x256.png", 256),
    ("icon-384x384.png", 384),
    ("icon-512x512.png", 512),
    ("ms-icon-144x144.png", 144)
]

def ensure_output_dir() -> None:
    """Verifica si el directorio de salida existe, si no, lo crea."""
    if not os.path.exists(OUTPUT_DIR):
        print(f"Creandodirectorio de salida: {OUTPUT_DIR}")
        os.makedirs(OUTPUT_DIR, exist_ok=True)

def generate_pngs(img: Image.Image) -> None:
    """Redimensiona y guarda las imágenes PNG."""
    print(f"Generando archivos.png en {OUTPUT_DIR}...")
    for filename, size in PNG_SIZES:
        output_path: str = os.path.join(OUTPUT_DIR, filename)

        # LANCZOS para mantener la mejor calidad posible al reducir
        resized_img: Image.Image = img.resize((size, size), Image.Resampling.LANCZOS)
        resized_img.save(output_path, format="PNG")
        print(f"✅ Guardado: {filename}")

def generate_safari_svg(image_path: str, output_name: str = "safari-pinned-tab.svg") -> None:
    """
    Genera un SVG monocromático trazando los contornos del canal alfa.
    """
    output_path: str = os.path.join(OUTPUT_DIR, output_name)
    print(f"\nGenerando {output_name} en: {OUTPUT_DIR}...")

    img_cv: Optional[np.ndarray] = cv2.imread(image_path, cv2.IMREAD_UNCHANGED)

    if img_cv is None:
        print(f"Error: No se pudo leer la imagen {image_path} con OpenCV.")
        return
    
    if img_cv.shape[2] == 4:
        mask: np.ndarray = img_cv[:, :, 3]
    else:
        gray: np.ndarray = cv2.cvtColor(img_cv, cv2.COLOR_BGR2GRAY)
        _, mask = cv2.threshold(gray, 127, 255, cv2.THRESH_BINARY_INV)

    _, thresh = cv2.threshold(mask, 127, 255, cv2.THRESH_BINARY)
    contours, _ = cv2.findContours(thresh, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

    height: int
    width: int
    height, width = mask.shape

    with open(output_path, 'w') as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}">\n')
        f.write(f'  <g fill="black">\n')

        for contour in contours:
            if len(contour) < 3:
                continue

            path_points: List[str] = []
            for point in contour:
                x: int = point[0][0]
                y: int = point[0][1]
                path_points.append(f"{x},{y}")

            path_data: str = "M " + " L ".join(path_points) + " Z"
            f.write(f'    <path d="{path_data}" />\n')

        f.write('  </g>\n')
        f.write('</svg>\n')

    print(f"✅ Guardado: {output_name}")

def main() -> None:
    input_path: str = os.path.join(SCRIPT_DIR, INPUT_IMAGE_NAME)

    if not os.path.exists(input_path):
        print(f"Error: No se encontró la imagen '{INPUT_IMAGE_NAME}' en {SCRIPT_DIR}.")
        sys.exit(1)

    ensure_output_dir()

    try:
        img: Image.Image = Image.open(input_path)
    except Exception as e:
        print(f"Error abriendo la imagen: {e}")
        sys.exit(1)

    generate_pngs(img)

    generate_safari_svg(input_path)

    print("\n¡Proceso finalizado! Los iconos han sido actualizados en la carpeta frontend.")

main()