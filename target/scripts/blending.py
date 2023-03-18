import vapoursynth as vs
from vapoursynth import core
import sys

# you'll probably need to remove "from scripts" if you use that somewhere else
from scripts import weighting
from scripts import havsfunc

import logging
logging.basicConfig(level=logging.DEBUG)
import ast


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


def _parse_weights(orig) -> tuple:
    if not orig:
        raise ValueError('no weights given')

    if isinstance(orig, list):
        return 'divide', {'weights': orig}

    else:
        if orig[0] == '[' and orig[-1] == ']':
            weights = [j.trim() for j in orig.stripr('[').stripl(']').split(',')]

        orig = orig.replace(' ', '')
        orig = orig.split('|')
        func_name = orig[0]

        if not hasattr(weighting, func_name):
            raise ValueError(f'Invalid weighting function: "{func_name}"')

        if len(orig) == 1:
            return func_name, {}

        else:
            params = {}
            for pair in orig[1].split(';'):
                param, value = pair.split('=')
                if param in ('std', 'std_dev', 'stddev'):
                    param = 'std_dev'
                # custom func is a string that literal_eval can't parse
                if not (func_name == 'custom' and param == 'func'):
                    try:
                        value = literal_eval(value)
                    except ValueError as v:
                        raise ValueError(f'weighting: invalid value "{value}" '
                                         f'for parameter "{param}"') from v

                params[param] = value

            return func_name, {**params}


def parse_literal(lit: str, opt: str):
    try:
        return ast.literal_eval(lit)
    except ValueError as v:
        raise ValueError(f'weighting: invalid value "{lit}" '
                         f'for option "{opt}"') from v


def parse_weights2(clip: vs.VideoNode, fbd: dict[str, str]) -> list[float]:

    n_weights = round((clip.fps_num / clip.fps_den) / int(fbd['fps']) * float(fbd['intensity']))
    orig = fbd['weighting']

    if n_weights > 0:
        if n_weights % 2 == 0:  # If number is not odd (requires odd number of frames)
            n_weights += 1

    if not orig:
        raise ValueError('No weights given')

    if orig == 'vegas':
        return weighting.vegas(
            input_fps=round(clip.fps_num / clip.fps_den),
            out_fps=int(fbd['fps']),
            blur_amt=float(fbd['intensity'])
        )

    if orig[0] == '[' and orig[-1] == ']':
        return weighting.divide(n_weights, parse_literal(orig, 'weights'))

    to_parse = orig.replace(' ', '').split('|', 1)  # only split once
    func_name = to_parse.pop(0)

    if not hasattr(weighting, func_name):
        raise ValueError(f'Invalid weighting function: "{func_name}"')

    fn = getattr(weighting, func_name)

    if not to_parse:  # no parameters given
        return fn(frames=n_weights)

    params = {'frames': n_weights}

    for pair in to_parse[0].split(';'):
        param, value = pair.split('=')
        # custom func is a string that literal_eval can't parse
        if func_name != 'custom' and param != 'func':
            value = parse_literal(value, param)

        params[param] = value

    return fn(**params)


def format_vec(v: list[float]):
    rounded = [round(x, 3) for x in v]

    if len(rounded) <= 3:
        return str(rounded)

    return str(rounded[0:2])[:-1] + ', ..., ' + str(rounded[-3:-1])[1:]


def _test_weights():
    import random

    tests = (  # add more here
        '[0.1, 0.2, 0.3, 0.5]',
        '[1, 2, 3, 4, 5]',
        'gaussian | apex = 3; std_dev = 2.2',
        'gaussian_sym',
        'custom | func = x**2'
    )

    fps_vals = [60 * i for i in range(4, 12)]

    for t in tests:
        fps = random.choice(fps_vals)
        ofps = random.choice([120, 60, 30])
        intensity = random.choice([1, 1.2, 1.3, 1.5, 1.8, 2, 3])

        fbd = {'intensity': intensity, 'fps': ofps, 'weighting': t}
        vals = parse_weights(vs.VideoNode(fps), fbd)

        print(f'{fps} -> {ofps} @ {intensity} using "{t}" =>', format_vec(vals), end='\n\n')


def FrameBlend(clip: vs.VideoNode, fbd: dict, is_verbose: bool) -> vs.VideoNode:

    # duplicated code I know... let me know a better way to bring them into scope.
    def verb(msg):
        if is_verbose:
            print(logging.debug(f'VERB: {msg}'))

    YES: list = ['on', 'True', 'true', 'yes', 'y', '1', 'yeah', 'yea', 'yep', 'sure', 'positive', True]
    NO: list = ['off', 'False', 'false', 'no', 'n', 'nah', 'nope', 'negative', 'negatory', '0', 'null', '', ' ', '  ',
                '\t', 'none', None, False]

    weights = parse_weights2(clip, fbd)

    fps = round(clip.fps_num / clip.fps_den)

    logging.debug(
        f'{fps} {fbd["fps"]} @ {fbd["intensity"]} ({len(weights)} blur-frames) using "{fbd["weighting"]}" => ' + format_vec(
            weights))

    if fbd["bright blend"] not in NO:
        og_format = clip.format
        og_matrix = 1#clip.get_frame(0).props._Matrix
        clip = core.resize.Bicubic(clip=clip, format=vs.RGB48, transfer_in_s="709", transfer_s="linear", matrix_in_s="709")

    clip = average(clip=clip, weights=weights)

    if fbd["bright blend"] in YES:

        clip = core.resize.Bicubic(clip=clip, format=og_format, matrix=og_matrix, transfer_s="709", transfer_in_s="linear", matrix_s="709")

    clip = havsfunc.ChangeFPS(clip, int(fbd['fps']))

    return clip