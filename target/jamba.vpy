# vi: ft=python

import vapoursynth as vs

core = vs.core

from os import path  # filepath handling
from sys import path as envpath  # Python's own PATH

if f"{path.dirname(__file__)}/scripts" not in envpath:  # Sometimes isn't by default
    envpath.append(f"{path.dirname(__file__)}/scripts")  # From script directory

# in order of (optional) use
import havsfunc
import blending
import weighting
import adjust
import filldrops

# import scripts  # Import every .py script modules from /plugins/

import json  # for .loads

for var in ['input_video', 'recipe']:
    if var not in vars():
        raise NameError(
            "You need to pass in both input_video and recipe variables via VSPipe's --arg"
        )

try:  # I have no clue why it returns a str the first time, that works though :shrug:
    recipe = json.loads(recipe)
    rc = json.loads(recipe)['data']

except json.decoder.JSONDecodeError:
    print(f"\n\nRecipe: {recipe}\n")
    raise Exception("Failed to parse the passed recipe as a JSON string")

YES: list = ['on', 'True', 'true', 'yes', 'y', '1', True]
NO: list = ['off', 'False', 'false', 'no', 'n', '0', 'null', '', ' ', '  ', '\t', 'none', None, False]

import logging
logging.basicConfig(level=logging.DEBUG)


def verb(msg):
    if rc['miscellaneous']['always verbose'].lower() in YES:
        print(logging.debug(f'VERB: {msg}'))


_, input_ext = path.splitext(input_video)

if input_ext == '.avi':

    clip: vs.VideoNode = core.avisource.AVISource(input_video)
else:

    clip: vs.VideoNode = core.lsmas.LWLibavSource(
        # * Docs and plugin came from https://github.com/AkarinVS/L-SMASH-Works/tree/master/VapourSynth

        source=input_video.replace("\\\\?\\", ""),
        # \\?\ caused by Rust's .canonicalize(), not supported by lsmas

        cache=int(rc['miscellaneous']['source indexing'].lower() in YES),
        # Smoothie comes bundled with the cachedir-tmp variant of this plugin
        # * Create the index file (.lwi) to the same directory as the source file if set to 1.
        # * The index file avoids parsing all frames in the source file at the next or later access.
        # * Parsing all frames is very important for frame accurate seek.

        prefer_hw=3
        # * 0: Use default software decoder.
        # * 1: Use NVIDIA CUVID acceleration for supported codec, otherwise use default software decoder.
        # * 2: Use Intel Quick Sync Video acceleration for supported codec, otherwise use default software decoder.
        # * 3: Try hardware decoder in the order of CUVID->QSV. If none is available then use default software decoder.

    )
# source


if (in_scale := float(rc['timescale']['in'])) != 1.0:
    clip = core.std.AssumeFPS(clip, fpsnum=(clip.fps * (1 / in_scale)))
# input timescale


if (dt := rc['miscellaneous']['dedup threshold']).lower() not in NO:
    clip = filldrops.FillDrops(
        clip,
        thresh=float(dt)
    )
# deduplication


if (pi := rc['pre-interp'])['enabled'].lower() in YES:

    # Color conversion snippet taken from Flowframes:
    # Source: https://github.com/n00mkrad/flowframes/blob/main/Code/Os/VapourSynthUtils.cs
    # Support: https://www.patreon.com/n00mkrad

    # I'm sure some vs wizards can overhaul this to be much shorter, feel free to PR

    cMatrix = '709'

    try:
        m = clip.get_frame(0).props._Matrix

        if m == 0:
            cMatrix = 'rgb'
        elif m == 4:
            cMatrix = 'fcc'
        elif m == 5:
            cMatrix = '470bg'
        elif m == 6:
            cMatrix = '170m'
        elif m == 7:
            cMatrix = '240m'
        elif m == 8:
            cMatrix = 'ycgco'
        elif m == 9:
            cMatrix = '2020ncl'
        elif m == 10:
            cMatrix = '2020cl'
        elif m == 12:
            cMatrix = 'chromancl'
        elif m == 13:
            cMatrix = 'chromacl'
        elif m == 14:
            cMatrix = 'ictcp'

    except:
        cMatrix = '709'

    colRange = 'limited'

    try:
        if clip.get_frame(0).props._ColorRange == 0: colRange = 'full'
    except:
        colRange = 'limited'

    if clip.format.color_family == vs.YUV:
        clip = core.resize.Bicubic(clip=clip, format=vs.RGBS, matrix_in_s=cMatrix, range_s=colRange)

    if clip.format.color_family == vs.RGB:
        clip = core.resize.Bicubic(
            clip=clip,
            format=vs.RGBS
        )

    if path.exists(pi['model']):
        if path.isfile(pi['model']):
            raise NotADirectoryError("You need to specify the model's directory, not file")
        model_path = pi['model']
    else:
        model_path = f"{rc['runtime']['smDir']}/models/{pi['model']}"

    if not path.exists(model_path):
        raise FileNotFoundError(f"Model directory not found: [{model_path}]")

    # * Plugin from https://github.com/styler00dollar/VapourSynth-RIFE-ncnn-Vulkan#usage
    clip = core.rife.RIFE(
        clip=clip,
        factor_num=str(pi['factor']).strip('x'),
        model_path=model_path,
        gpu_id=0,
        gpu_thread=1,
        tta=False,
        uhd=False,  # tune for 1440p+ content
        sc=False  # scene change
    )

    clip = vs.core.resize.Bicubic(clip, format=vs.YUV420P8, matrix_s=cMatrix)
# pre-interp


if (ip := rc['interpolation'])['enabled'].lower() in YES:

    if ip['area'].lower() in NO:
        area = None
    else:
        area = int(ip['area'])

    # * Adjusted InterFrame from havsfunc,support 10bit with new svp
    # * Source: https://github.com/xyx98/my-vapoursynth-script/blob/master/xvs.py

    clip = havsfunc.InterFrame(
        Input=clip,
        GPU=ip['use gpu'].lower() in YES,
        Preset=ip['speed'],
        Tuning=ip['tuning'],
        NewNum=float(ip['fps']),
        NewDen=1,
        OverrideAlgo=int(ip['algorithm']),
        OverrideArea=area
    )
# interpolation


if (out := float(rc['timescale']['out'])) != 1:  # Output timescale, done after interpolation

    clip = core.std.AssumeFPS(clip, fpsnum=int(clip.fps * out))
# output timescale


if (fbd := rc['frame blending'])['enabled'].lower() in YES:

    if fbd['weighting'].lower() in NO:
        verb("No weighting mode passed: defaulting to equal")
        fbd['weighting'] = 'equal'

    if float(fbd['fps']) >= (clip.fps_den / clip.fps_num) + 1:
        verb('Input video FPS is lower or equal compared to frame blending FPS, skipping frame blending')
    else:
        clip = blending.FrameBlend(clip, fbd, rc['miscellaneous']['always verbose'].lower() in YES)
# frame blending


def Masking(
        to_mask: vs.VideoNode,
        original_clip: vs.VideoNode,
        mask_path: str):

    rmask = core.ffms2.Source(mask_path)
    featherMask = rmask.std.Minimum().std.BoxBlur(vradius=6, hradius=6, vpasses=2, hpasses=2)

    return core.std.MaskedMerge(clipa=to_mask, clipb=original_clip, mask=featherMask, first_plane=True)
# mask func


if (flb := rc['flowblur'])['enabled'].lower() in YES and rc['flowblur']['enabled'].lower() not in NO:
    pass

    if flb['mask'].lower() not in NO:
        original = clip  # makes a "backup"

    # to be honest with you, I got no heckin idea how to configure this
    # mess with it and suggest what looks better 👍
    super_clip = core.mv.Super(clip, 16, 16, rfilter=3)
    backwards_vectors = core.mv.Analyse(super_clip, isb=True, blksize=16, plevel=2, dct=5)
    forward_vectors = core.mv.Analyse(super_clip, blksize=16, plevel=2, dct=5)
    clip = core.mv.FlowBlur(
        clip,
        super_clip,
        backwards_vectors,
        forward_vectors,
        blur=rc['flowblur']['amount']
    )

    if (mask := flb['mask']).lower() not in NO:

        mask = mask.strip('"')

        if not path.exists(mask):
            raise ValueError(
                "The mask you specify must be absolute"
                " (get it by doing shift-right click the image -> Copy As Path)"
            )

        clip = Masking(
            to_mask=clip,
            original_clip=original,
            mask_path=mask
        )
# flowblur


clip.set_output()
