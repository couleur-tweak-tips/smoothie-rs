import ast
import logging
import sys
import warnings

# you'll probably need to remove "from scripts" if you use that somewhere else
from scripts import weighting
from scripts.consts import NO, YES

import vapoursynth as vs
from vapoursynth import core

logging.basicConfig(level=logging.DEBUG)


# thanks to eoe
def average(clip: vs.VideoNode, weights: list[float], divisor: float | None = None):
    def get_offset_clip(offset: int) -> vs.VideoNode:
        if offset > 0:
            return clip[0] * offset + clip[:-offset]
        elif offset < 0:
            return clip[-offset:] + clip[-1] * (-offset)
        else:
            return clip

    diameter = len(weights)
    radius = diameter // 2

    if divisor is None:
        divisor = sum(weights)

    assert diameter % 2 == 1, "An odd number of weights is required."

    clips = [get_offset_clip(offset) for offset in range(-radius, radius + 1)]

    expr = ""
    # expr_vars = "xyzabcdefghijklmnopqrstuvw"
    expr_vars = []
    for i in range(0, 1024): expr_vars += [f"src{i}"]

    for var, weight in zip(expr_vars[:diameter], weights):
        expr += f"{var} {weight} * "

    expr += "+ " * (diameter - 1)
    expr += f"{divisor} /" if divisor != 1 else ""
    # https://github.com/AkarinVS/vapoursynth-plugin
    clip = core.akarin.Expr(clips, expr)

    return clip


def vegas_weights(input_fps: int, out_fps: int, blur_amt: float = 1.0) -> list[float]:

    blend_factor = int(input_fps / out_fps * blur_amt)
    n_weights = blend_factor + (1 - (blend_factor % 2)) # + 1 if even

    if blend_factor % 2 == 0:
        weights = [1] + [2] * (n_weights - 2) + [1] # - 2 for first and last
    else:
        weights = [1] * n_weights

    return weighting.normalize(weights)


def parse_literal(lit: str, opt: str):
    try:
        return ast.literal_eval(lit)
    except ValueError as v:
        raise ValueError(f'Invalid value "{lit}" '
                         f'for option "{opt}"') from v


def parse_weights2(clip: vs.VideoNode, fbd: dict[str, str]) -> list[float]:

    input_fps = round(clip.fps_num / clip.fps_den)

    n_weights = round(input_fps / int(fbd['fps']) * float(fbd['intensity']))
    orig = fbd['weighting']

    if n_weights <= 1:
        return [1.0]

    if n_weights % 2 == 0:  # If number is not odd (requires odd number of frames)
        n_weights += 1

    if not orig:
        raise ValueError('No weights given')

    to_parse = [x for x in orig.replace(' ', '').split(';') if x]
    func_name = to_parse.pop(0)

    if func_name == 'vegas':
        return vegas_weights(
            input_fps=input_fps,
            out_fps=int(fbd['fps']),
            blur_amt=float(fbd['intensity'])
        )

    params = {'frames': n_weights}

    if func_name[0] == '[' and func_name[-1] == ']':
        fn = weighting.divide
        params['weights'] = parse_literal(func_name, 'weights')

    else:
        if not hasattr(weighting, func_name):
            raise ValueError(f'Invalid weighting function: "{func_name}"')

        fn = getattr(weighting, func_name)

    if not to_parse:  # no extra parameters given
        return fn(**params)

    for pair in to_parse:
        if '=' not in pair:
            raise ValueError(f'Options must be of the form "name=value", not "{pair}"')

        param, value = pair.split('=')

        if param == 'frames':
            raise ValueError('Cannot set option "frames" manually')

        if param == 'wizardry' and value in YES:
            weighting.enable_wizardry()
            continue

        if params.get(param) is not None:
            warnings.warn(f'Option "{param}" is set multiple times')

        params[param] = parse_literal(value, param)

    return fn(**params)


def format_vec(v: list[float]):
    rounded = [f'{x:.2f}' for x in v]
    if len(rounded) > 4:
        rounded = rounded[:2] + ['...'] + rounded[-2:]
    return f"[{', '.join(rounded)}]"


def _test_weights():
    import random

    tests = (  # add more here
        '[0.1, 0.2, 0.3, 0.5]',
        '[1, 2, 3, 4, 5]; wizardry = yes',
        '[1, 2, 3, 4, 5]',
        'gaussian; apex = 3; std_dev = 2.2',
        'gaussian_sym',
    )

    fps_vals = [60 * i for i in range(4, 12)]

    for fn in tests:
        fps = random.choice(fps_vals)
        ofps = random.choice([120, 60, 30])
        intensity = random.choice([1, 1.2, 1.3, 1.5, 1.8, 2, 3])

        fbd = {'intensity': intensity, 'fps': ofps, 'weighting': fn}
        node = core.std.BlankClip(fpsnum=fps, fpsden=1, length=1)
        vals = parse_weights2(node, fbd)

        print(f'{fps} -> {ofps} @ {intensity} using "{fn}" =>', format_vec(vals), end='\n\n', file=sys.stderr)


def FrameBlend(clip: vs.VideoNode, fbd: dict, is_verbose: bool, weights: list[float]) -> vs.VideoNode:

    def verb(msg):
        if is_verbose:
            logging.debug(f'VERB: {msg}')

    if fbd["bright blend"] not in NO:
        og_format = clip.format
        og_matrix = 1#clip.get_frame(0).props._Matrix
        clip = core.resize.Bicubic(clip=clip, format=vs.RGB48, transfer_in_s="709", transfer_s="linear", matrix_in_s="709")

    clip = average(clip=clip, weights=weights)

    if fbd["bright blend"] in YES:

        clip = core.resize.Bicubic(clip=clip, format=og_format, matrix=og_matrix, transfer_s="709", transfer_in_s="linear", matrix_s="709")

    return clip
