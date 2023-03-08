> **Warning**
>
> WIP, do not expect a working program 👍

## smoothie-rs

Temporary repository to write the code for Smoothie's future form: developed in rust 🦀.
Thanks to yalter and anima for inspiration (though I'll try to copy as less code as possible).

### Roadmap

- [X] Input processing/validation
- [X] ``rc`` and `dir` "void" args
- [X] --rerun (`last_args.txt`)
- [X] recipe parsing with defaults backup
- [ ] Option to pass a .vpy script instead of the internal api calls used (python runtime required)
- [ ] Progress bar 
- [ ] Refactor VEGAS Pre-Renderer script & [suckless-cut](https://github.com/couleur-tweak-tips/suckless-cut) to support sm-rs' ``--json``


Using rustsynth would slim down Smoothie's bundle to just:

- 🗜 `smoothie-0.69.zip`
    - 📂``Smoothie/``
        - 📂``models/`` - rife models
        - 📂``masks/`` - artifact masking for flowblur (and maybe interp soon)
        - 📝 ``recipe.ini``
        - 📂``bin/`` - let me know a more fit name since there are also text files in here
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
                - ⚙ ``libmvtools.dll`` - for flowblur
                - ⚙ ``svpflow1_vs.dll`` & ``svpflow2_vs.dll`` - frame interpolation
                - ⚙ ``akarin.dll`` - for lexpr

Else in that ``/bin/`` folder there would be a whole portable Python environment

## The future of this repository

The replacement process will go like so:

1. ctt/smoothie - new branch called "python-old" created (thus creating an archive of how was smoothie before)
2. ctt/smoothie - 1 new commit: "removed all python code 🐍❌"
3. ctt/smoothie - 1 new commit: "copied over all rust code 🦀🦀"
4. ctt/smoothie-rs - archive repository and link ctt/smoothie in README
