from os import path

IMAGE_EXTS = ('.png', '.jpg', '.jpeg', '.webp', '.tiff', '.bmp', '.jxl', '.avif')
VIDEO_EXTS = ('.mp4', '.mkv', '.mov', '.mkv', '.wmv', '.webm', '.ts')

# These are not really constants but w/e, they workie 😋
SRCDIR = path.dirname(__file__)
SMDIR = path.dirname(SRCDIR)  # That's ../../ if you look at it relative from Smoothie/src/main.py
MASKDIR = path.join(SMDIR, "masks")  # Except this one
