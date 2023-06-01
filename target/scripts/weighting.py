"""
All weighting functions are of the following basic form:
    Args:
        frames: `int` | number of frames to generate weights for
    Returns:
        `list[float]`: `[w1, w2, ..., wN]` | weights for each frame
Reference:
    https://github.com/siveroo/HFR-Resampler
"""

import math
import warnings as w
from numbers import Number
from typing import Iterable

_wizardry_enabled = False


def enable_wizardry():
    global _wizardry_enabled
    _wizardry_enabled = True


def normalize(weights: Iterable[Number]):
    """
    Normalize a list of numbers to sum up to 1
    """

    if min(weights) < 0 and not _wizardry_enabled:
        abs_min = abs(min(weights))
        weights = [w + abs_min + 1 for w in weights] # remove negative weights

    tot = sum(weights)
    return [w / tot for w in weights]


def scale_range(n: int, start: Number, end: Number):
    """
    Returns a list of `n` numbers from `start` to `end`
    >>> res = scale_range(5, 0, 1)
    >>> assert res[0] == 0 and res[-1] == 1
    >>> assert len(res) == 5
    """
    if n <= 1: return [start] * n
    return [(x * (end - start) / (n - 1)) + start for x in range(n)]


def ascending(frames: int):
    """
    Linear ascending curve
    """
    return normalize(range(1, frames + 1))


def descending(frames: int):
    """
    Linear descending curve
    """
    return normalize(range(frames, 0, -1))


def equal(frames: int):
    """
    Flat curve
    """
    return [1 / frames] * frames


def gaussian(frames: int, apex: Number = 2, std_dev: Number = 1, bound: tuple[Number, Number] = (0, 2)):
    """
    Args:
        bound: `[a, b]` | x axis vector from `a` to `b`
        apex: `μ`       | the position of the center of the peak, relative to x axis vector
        std_dev: `σ`    | width of the "bell", higher == broader/flatter
    Reference:
        https://en.wikipedia.org/wiki/Gaussian_function
    """
    _warn_bound(bound, "gaussian")

    r = scale_range(frames, bound[0], bound[1]) # x axis vector

    val = [1 / (math.sqrt(2 * math.pi) * std_dev) # normalization
           * math.exp(-((x - apex) / std_dev) ** 2 / 2) # gaussian function
           for x in r]

    return normalize(val)


def gaussian_sym(frames: int, std_dev: Number = 1, bound: tuple[Number, Number] = (0, 2)):
    """
    Same as `gaussian()` but symmetric;
    the peak (apex) will always be at the center of the curve
    """
    _warn_bound(bound, "gaussian_sym")

    max_abs = max(map(abs, bound[:2]))
    r = scale_range(frames, -max_abs, max_abs)

    val = [1 / (math.sqrt(2 * math.pi) * std_dev)
           * math.exp(-(x / std_dev) ** 2 / 2)
           for x in r]

    return normalize(val)


def pyramid(frames: int):
    """
    Symmetric pyramid function
    """
    half = (frames - 1) / 2
    val = [half - abs(x - half) + 1 for x in range(frames)]

    return normalize(val)


def func_eval(func: str, nums: list[float]):
    """
    Run an operation on a sequence of numbers
    Names allowed in `func`:
        - Everything in the `math` module
        - `x`: the current number (frame) in the sequence
        - `frames` (`len(nums)`): number of elements in the sequence (blended frames)
        - The following built-in functions: `sum`, `abs`, `max`, `min`, `len`, `pow`, `map`, `range`, `round`
    """

    # math functions + math related builtins
    namespace = {k: v for k, v in math.__dict__.items() if not k.startswith("_")}
    namespace |= {
        'frames': len(nums), # total number of items (frames)
        'x': None, # iterator for nums
        '__builtins__': {
            'sum': sum,
            'abs': abs,
            'max': max,
            'min': min,
            'len': len,
            'pow': pow,
            'map': map,
            'range': range,
            'round': round
        }
    }
    # only allow functions specified in namespace
    return eval(f"[({func}) for x in {nums}]", namespace)


def custom(frames: int, func: str = "", bound: tuple[Number, Number] = (0, 1)):
    """
    Arbitrary custom weighting function
    """
    _warn_bound(bound, "custom")

    r = scale_range(frames, bound[0], bound[1])
    val = func_eval(func, r)

    return normalize(val)


def divide(frames: int, weights: list[float]):
    """
    Stretch the given array (weights) to a specific length (frames)
    Example: `frames = 10; weights = [1, 2]`
    Result: `val == [1, 1, 1, 1, 1, 2, 2, 2, 2, 2]`, then normalize it to
    `[0.0667, 0.0667, 0.0667, 0.0667, 0.0667, 0.1333, 0.1333, 0.1333, 0.1333, 0.1333]`
    """
    r = scale_range(frames, 0, len(weights) - 0.1)
    val = [weights[int(r[x])] for x in range(frames)]

    return normalize(val)


def _warn_bound(bound: tuple, func_name: str):
    if len(bound) < 2:
        raise ValueError(f"{func_name}: bound must be a sequence of length 2, got {bound}")
    elif len(bound) > 2:
        w.warn(f"{func_name}: bound was given as a sequence of length {len(bound)}, "
               f"only the first two values will be used (got {bound}))",
               RuntimeWarning)


__all__ = [ascending, descending, equal, gaussian, gaussian_sym, pyramid, custom, divide, enable_wizardry]
