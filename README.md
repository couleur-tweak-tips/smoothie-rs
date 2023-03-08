> **Warning**
>
> WIP, do not expect a working program 👍

## smoothie-rs

Temporary repository for Smoothie's future form: developed in Rust 🦀.
Thanks to yalter and anima for inspiration (though I'll try to copy as little code as possible).

### Roadmap

- [X] Input processing/validation
- [X] ``rc`` and `dir` "void" args
- [X] --rerun (`last_args.txt`)
- [X] Recipe parsing with defaults backup
- [ ] Option to pass a .vpy script instead of the internal API calls used (Python runtime required)
- [ ] Progress bar
- [ ] Refactor VEGAS pre-renderer script & [suckless-cut](https://github.com/couleur-tweak-tips/suckless-cut) to support sm-rs' ``--json``


Using rustsynth would slim Smoothie's bundle down to just:

- 🗜 `smoothie-0.69.zip`
    - 📂``Smoothie/``
        - 📂``models/`` - RIFE models
        - 📂``masks/`` - artifact masking for flowblur (and maybe interp soon too)
        - 📝 ``recipe.ini``
        - 📂``bin/`` - let me know a more fitting name since there are also text files in here
            - 📚 ``defaults.ini`` - read-only version of the recipe, used as a fallback
            - 🧋 ``Smoothie.exe`` - passes all arguments to sm and add `-cui`
            - 💾 ``sm.exe`` - consider this to be the "core", would be the biggest file out of the two
            - 🗒 ``last_args.txt`` - not really a binary but I prefer it tucked in bin
            - ⚙ `VapourSynth.dll` - no idea if anything else is needed, haven't looked into last goal yet
            - ⚙ `msvcp140.dll`, `vcruntime140.dll`, `vcruntime140_1.dll` - dependencies for Rust & VapourSynth
            - 📝 `portable.vs` - tells VapourSynth it's a portable env, remove it for it to use global
            - 📂``/vapoursynth64/plugins/``
                - ⚙ ``libvslsmashsource.dll`` - to input videos
                - ⚙ ``RemapFramesVapoursynth.dll`` - for `-padding`
                - ⚙ ``RIFE.dll`` - for low fps interpolation, uses /models/
                - ⚙ ``libmvtools.dll`` - flowblur
                - ⚙ ``svpflow1_vs.dll`` & ``svpflow2_vs.dll`` - frame interpolation
                - ⚙ ``akarin.dll`` - for lexpr

Else in that ``/bin/`` folder there would be a whole portable Python environment

## The future of this repository

The replacement process will go as follows:

1. new branch called "python-old" created in ctt/smoothie (for archiving the the previous version of smoothie)
2. Remove all the Python code from ctt/smoothie
3. Copy over all of the Rust code from ctt/smoothie-rs to ctt/smoothie
4. Archive this repository and link ctt/smoothie in the readme
