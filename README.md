> **Warning**
>
> WIP, do not expect a working program 🚧

## smoothie-rs

Temporary repository for Smoothie's rewrite in Rust 🦀.
Thanks to [YaLTeR](https://github.com/YaLTeR/vapoursynth-rs) and [animafps](https://github.com/animafps/rustsynth) for inspiration. (I'll try to copy as little code as possible).

### Roadmap

- [X] Input processing/validation
- [X] ``rc`` and `dir` "void" args
- [X] --rerun (`last_args.txt`)
- [X] Recipe parsing with defaults backup
- [ ] Option to pass a .vpy script instead of the internal API calls used (Python runtime required)
- [ ] Progress bar
- [ ] Refactor VEGAS pre-renderer script & [suckless-cut](https://github.com/couleur-tweak-tips/suckless-cut) to support sm-rs' ``--json``


Using rustsynth would slim Smoothie's bundle down to just:

- 🗜 `smoothie-0.01.zip`
    - 📂``Smoothie/``
        - 📂``models/`` - RIFE models
        - 📂``masks/`` - artifact masking for flowblur (and maybe interp soon too)
        - 📝 ``recipe.ini``
        - 📂``bin/`` - suggest a more fitting name, as this also contains text files
            - 📚 ``defaults.ini`` - read-only version of the recipe used as a fallback
            - 🧋 ``Smoothie.exe`` - passes all arguments to sm and add `-cui`
            - 💾 ``sm.exe`` - consider this to be the "core", would be the biggest file out of the two
            - 🗒 ``last_args.txt`` - not really a binary but I prefer it tucked in bin
            - ⚙ `VapourSynth.dll` - no idea if anything else is needed, I haven't looked into last goal yet
            - ⚙ `msvcp140.dll`, `vcruntime140.dll`, `vcruntime140_1.dll` - dependencies for Rust & VapourSynth
            - 📝 `portable.vs` - tells VapourSynth that it's a portable env, you can remove it so it's used globally
            - 📂``/vapoursynth64/plugins/``
                - ⚙ ``libvslsmashsource.dll`` - inputs videos
                - ⚙ ``RemapFramesVapoursynth.dll`` - for `-padding`
                - ⚙ ``RIFE.dll`` - low fps interpolation, uses /models/
                - ⚙ ``libmvtools.dll`` - flowblur
                - ⚙ ``svpflow1_vs.dll`` & ``svpflow2_vs.dll`` - frame interpolation
                - ⚙ ``akarin.dll`` - lexpr

If there were anything else in ``/bin/``, it would be a portable Python environment

## The future of this repository

The replacement process will go as follows:

1. Create new branch named "python-old" in [smoothie](https://github.com/couleur-tweak-tips/smoothie) (this will be used for archiving the the previous python versions)
2. Remove all the Python code from [smoothie](https://github.com/couleur-tweak-tips/smoothie/)
3. Copy over all of the Rust code from [smoothie-rs](https://github.com/couleur-tweak-tips/smoothie-rs) to [smoothie](https://github.com/couleur-tweak-tips/smoothie)
4. Archive this repository and link [smoothie](https://github.com/couleur-tweak-tips/smoothie) in the README
