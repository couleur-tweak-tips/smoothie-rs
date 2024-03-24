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
import warnings
from numbers import Number
from typing import Iterable


__all__ = ["ascending", "descending", "equal", "gaussian", "gaussian_sym", "pyramid", "divide", "enable_wizardry"]

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


def gaussian(frames: int, mean: Number = 2, std_dev: Number = 1, bound: tuple[Number, Number] = (0, 2)):
    """
    Args:
        bound: `[a, b]` | range for the x-axis, from `a` to `b` (inclusive)
        mean: `μ`       | the position of the center of the peak, relative to x-axis
        std_dev: `σ`    | width of the "bell", higher <=> broader/flatter
    Reference:
        https://en.wikipedia.org/wiki/Gaussian_function
    """
    _warn_bound(bound, "gaussian")

    x_axis = scale_range(frames, bound[0], bound[1])

    val = [math.exp(-(x - mean) ** 2 / (2 * std_dev ** 2))
           for x in x_axis]

    return normalize(val)


def gaussian_sym(frames: int, std_dev: Number = 1, bound: tuple[Number, Number] = (0, 2)):
    """
    Same as `gaussian()` but symmetric;
    the peak (mean) will always be at the center of the curve
    """
    _warn_bound(bound, "gaussian_sym")

    max_abs = max(map(abs, bound[:2]))
    return gaussian(frames, mean=0, std_dev=std_dev, bound=(-max_abs, max_abs))


def pyramid(frames: int):
    """
    Symmetric pyramid function
    """
    half = (frames - 1) / 2
    val = [half - abs(x - half) + 1 for x in range(frames)]

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
        warnings.warn(f"{func_name}: bound was given as a sequence of length {len(bound)}, "
                      f"only the first two values will be used (got {bound}))",
                      RuntimeWarning)
