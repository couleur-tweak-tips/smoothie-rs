# vi: ft=python
# I try to respect PEP 8 where I can, please do as well 👍

import vapoursynth as vs
from vapoursynth import core

import os
from os import path  # filepath handling
import sys

path_dirname = path.dirname(__file__).replace('\\\\?\\', '')  # .replace because windows

if path_dirname not in sys.path:  # Sometimes isn't by default
    sys.path.append(path_dirname)

# in order of (optional) use
from scripts import havsfunc
from scripts import blending
from scripts import adjust
from scripts import filldrops
from scripts.consts import YES, NO

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


def verb(*msg, **kwargs):
    if rc['miscellaneous']['always verbose'].lower() in YES:
        print("VERB:", *msg, file=sys.stderr, **kwargs)


def eprint(*msg, **kwargs):
    print("ERR:", *msg, file=sys.stderr, **kwargs)


verb(f"CONTEXT: Using vapoursynth bindings from {vs.__file__}")
verb(f"CONTEXT: __file__: {__file__}")
# verb(rc)

_, input_ext = path.splitext(input_video)

base_file_name = path.basename(input_video)
temp_folder = os.environ['TEMP']
cache_file = path.join(temp_folder, f"{base_file_name}-ffms2_index")

if input_ext == '.avi':

    clip: vs.VideoNode = core.avisource.AVISource(input_video)
else:


    eprint(f"cache_file: {cache_file}")

    clip: vs.VideoNode = core.ffms2.Source(
        source=input_video,
        cachefile=cache_file,
    )

    #clip: vs.VideoNode = core.lsmas.LWLibavSource(
    #    # * Docs and plugin came from https://github.com/AkarinVS/L-SMASH-Works/tree/master/VapourSynth
    #
    #    source=input_video.replace("\\\\?\\", "").strip('"'),
    #    # \\?\ caused by Rust's .canonicalize(), not supported by lsmas
    #
    #    cache=int(rc['miscellaneous']['source indexing'].lower() in YES),
    #    # Smoothie comes bundled with the cachedir-tmp variant of this plugin
    #    # * Create the index file (.lwi) to the same directory as the source file if set to 1.
    #    # * The index file avoids parsing all frames in the source file at the next or later access.
    #    # * Parsing all frames is very important for frame accurate seek.
    #
    #    prefer_hw=3
    #    # * 0: Use default software decoder.
    #    # * 1: Use NVIDIA CUVID acceleration for supported codec, otherwise use default software decoder.
    #    # * 2: Use Intel Quick Sync Video acceleration for supported codec, otherwise use default software decoder.
    #    # * 3: Try hardware decoder in the order of CUVID->QSV. If none is available then use default software decoder.
    #
    #)
# source plugin

# haven't bothered adding an arg to this 😴
# this is for nerdy-ahhh comparisons
cap_fps = 0
if cap_fps != 0:
    eprint(f"CAPPING FRAME-RATE TO {cap_fps}")
    clip = havsfunc.ChangeFPS(clip, cap_fps, 1)


def resolve_mask_path(folder_path: str, file_name: str):

    if file_name in NO:
        eprint(f"GOT: {folder_path} AND {file_name}")
        raise ValueError(
            "If masking is enabled you MUST provide `[artifact masking] file name:`"
        )

    mask_path = path.join(folder_path, file_name)

    if not path.exists(mask_path):

        default_mask_dir = path.join(path.dirname(path.dirname(__file__)), "artifact masks")

        mask_path = path.join(default_mask_dir, file_name)

        if not path.exists(mask_path):
            eprint(f"EXPECTED: {mask_path}")
            raise ValueError(
                "Could not resolve your mask's input path, \n"
                "The path of the mask you specify must be absolute \n"
                "(get it by doing shift-right click the image -> Copy As Path)"
            )
    return mask_path


if (am := rc['artifact masking'])['enabled'].lower() in YES:
    MASK_PATH = resolve_mask_path(
        folder_path=am['folder path'],
        file_name=am['file name']
    )
else:
    MASK_PATH = None
# mask path resolve


if (in_scale := float(rc['timescale']['in'])) != 1.0:
    clip = core.std.AssumeFPS(clip, fpsnum=(clip.fps * (1 / in_scale)))
# input timescale


if (dt := rc['miscellaneous']['dedup threshold']).lower() not in NO:

    # eprint("balls!")

    if (dt := rc['miscellaneous']['dedup threshold']).lower() in YES:
        dt = 0.001
    clip = filldrops.FillDrops(
        clip,
        thresh=float(dt)
    )


# deduplication


def Masking(
        to_mask: vs.VideoNode,
        original_clip: vs.VideoNode,
):
    if MASK_PATH is None:
        return to_mask

    if to_mask.fps != original_clip.fps:
        original_clip = havsfunc.ChangeFPS(original_clip, fpsnum=to_mask.fps.numerator, fpsden=to_mask.fps.denominator)

    og_clip_format = clip.format

    # technically a source as well, but it's 1 image, we chill
    mask_src = core.ffms2.Source(MASK_PATH)

    if to_mask.format.name == 'RGBS':
        mask_src = core.resize.Bicubic(clip=mask_src, format=vs.RGBS)

    if rc['artifact masking']['feathering'] in YES:
        mask_src = mask_src.std.Minimum().std.BoxBlur(vradius=6, hradius=6, vpasses=2, hpasses=2)

    return core.std.MaskedMerge(clipa=original_clip, clipb=to_mask, mask=mask_src, first_plane=True)


# mask func


if (pi := rc['pre-interp'])['enabled'].lower() in YES:

    model_path = pi['model'].strip('"')

    if not path.isdir(model_path):

        default_model_dir = path.join(path.dirname(path.dirname(__file__)), "rife models")

        relative_model_path = path.join(default_model_dir, model_path)

        if path.isfile(relative_model_path):
            raise NotADirectoryError("You need to specify the model's directory, not a file")

        if not path.isdir(relative_model_path):
            eprint(f"\n\ndefault_model_dir does not exist, expected: {relative_model_path}")
            raise NotADirectoryError("Could not find RIFE model directory..")

        model_path = relative_model_path

    og_format = clip.format
    og_matrix = clip.get_frame(0).props._Matrix
    clip = core.resize.Bicubic(clip=clip, format=vs.RGBS)

    factor = pi['factor'].strip('x')

    if pi['masking'] in YES:
        og_clip = clip

    # * Plugin from https://github.com/styler00dollar/VapourSynth-RIFE-ncnn-Vulkan#usage
    clip = core.rife.RIFE(
        clip=clip,
        factor_num=(pi['factor']).strip('x'),
        # multiplier=str(pi['factor']).strip('x'),
        model_path=model_path,
        gpu_id=0,
        gpu_thread=1,
        tta=False,
        uhd=False,  # tune for 1440p+ content
        sc=False  # scene change
    )

    verb('Masking pre-interp')
    if pi['masking'] in YES:
        clip = Masking(
            to_mask=clip,
            original_clip=og_clip
        )

    clip = core.resize.Bicubic(clip=clip, format=og_format, matrix=og_matrix)
    # eprint(f"og_matrix:{og_matrix}")
    # eprint(f"pre-interp\n{clip}")
# pre-interp


if (ip := rc['interpolation'])['enabled'].lower() in YES:

    if ip['area'].lower() in NO:
        area = None
    else:
        area = int(ip['area'])

    if ip['fps'].endswith("x") or ip['fps'].startswith("x"):
        ip['fps'] = int(ip['fps'].strip('x')) * clip.fps

    # * Adjusted InterFrame from havsfunc,support 10bit with new svp
    # * Source: https://github.com/xyx98/my-vapoursynth-script/blob/master/xvs.py

    if ip['masking'] in YES:
        og_clip = clip
        new_fps = int(ip['fps'])

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

    if ip['masking'] in YES:
        clip = Masking(
            to_mask=clip,
            original_clip=og_clip
        )
# interpolation


if (out := float(rc['timescale']['out'])) != 1:  # Output timescale, done after interpolation

    clip = core.std.AssumeFPS(clip, fpsnum=int(clip.fps * out))


# output timescale


# my mv.FlowBlur helper
def FlowBlur(
        clip: vs.VideoNode,
        flb: dict
) -> vs.VideoNode | None:
    if flb['masking'].lower() not in NO:
        original = clip  # makes a "backup"

    if flb['amount'].lower() in NO:
        return clip
    verb(flb)
    verb(f"Flowblurring @ {flb['intensity']} intensity")

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
        blur=int(rc['flowblur']['amount'])
    )

    if flb['masking'] not in YES:
        return clip

    verb('Masking Flowblur')
    return Masking(
        
        to_mask=clip,
        original_clip=original
    )


# def FlowBlur


if rc['flowblur']['do blending'] == 'before' and rc['flowblur']['enabled'].lower() in YES:
    clip = FlowBlur(clip, rc['flowblur'])
# flowblur (before)


if (fbd := rc['frame blending'])['enabled'].lower() in YES:

    if fbd['weighting'].lower() in NO:
        verb("No weighting mode passed: defaulting to equal")
        fbd['weighting'] = 'equal'

    if fbd['fps'].lower() not in NO and float(fbd['fps']) >= ((clip.fps_num / clip.fps_den) + 1):

        verb('Input video FPS is lower or equal compared to frame blending FPS, skipping frame blending')
    else:
        weights = blending.parse_weights2(clip, fbd)

        clip = blending.FrameBlend(
            clip,
            fbd,
            rc['miscellaneous']['always verbose'].lower() in YES,
            weights=weights)

        fps = round(clip.fps_num / clip.fps_den)

        clip = havsfunc.ChangeFPS(clip, int(fbd['fps']))
        verb(
            f'Blending {fps} down to {fbd["fps"]} @ {fbd["intensity"]} intensity ({len(weights)} blur-frames) ' +
            f"using '{fbd['weighting']}' => " + blending.format_vec(weights)
        )
# frame blending


if rc['flowblur']['do blending'] == 'after' and rc['flowblur']['enabled'].lower() in YES:
    clip = FlowBlur(clip, rc['flowblur'])
# flowblur (after)


if (cg := rc['color grading'])['enabled'].lower() in YES:
    clip = adjust.Tweak(
        clip,
        hue=float(cg['hue']),
        cont=float(cg['contrast']),
        sat=float(cg['saturation']),
        bright=float(cg['brightness']),
        coring=cg['coring'] in YES
    )
# color grading


if (lt := rc['lut'])['enabled'].lower() in YES:

    if (rc['lut'])['path'].lower() in NO:
        eprint("LUT filter was enabled, but no path was passed, skipping.. ")
    else:
        eprint(f"Adding a LUT with an opacity of {lt['opacity']}%, path: {lt['path']}")

        og_format = clip.format
        # enforcing matrix? absolutely
        lut_clip = core.resize.Bicubic(clip, format=vs.RGB48)
        lut_clip = lut_clip.timecube.Cube(cube=lt['path'].strip('"'))
        lut_clip = core.resize.Bicubic(lut_clip, format=og_format, matrix=1)

        clip = core.std.Merge(clip, lut_clip, weight=float(lt['opacity']))

clip.set_output()
