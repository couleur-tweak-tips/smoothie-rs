from os import path

IMAGE_EXTS = ('.png', '.jpg', '.jpeg', '.webp', '.tiff', '.bmp', '.jxl', '.avif')
VIDEO_EXTS = ('.mp4', '.mkv', '.mov', '.mkv', '.wmv', '.webm', '.ts')

YES = ['on', 'True', 'true', 'yes', 'y', '1', 'yeah', 'yea', 'yep', 'sure', 'positive', True]
NO = [
    'off', 'False', 'false', 'no', 'n', 'nah', 'nope', 'negative', 'negatory',
    '0', '0.0', 'null', '', ' ', '  ', '\t', 'none', None, False
]

# These are not really constants but w/e, they workie 😋
SRCDIR = path.dirname(__file__)
SMDIR = path.dirname(SRCDIR)  # That's ../../ if you look at it relative to Smoothie/src/main.py
MASKDIR = path.join(SMDIR, "masks")  # Except this one
