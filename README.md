> **Warning**
>
> As with a lot of projects, this one is also a WIP, expect broken code 👍

## smoothie-rs

Temporary repository hosting the code for Smoothie's future form: developped in rust 🦀.
Thanks to yalter and anima for inspiration (though I'll be copying as less code as possible).

### Development goals

- [X] Basic clap functionality/looping/queue
- [ ] Get VSPipe working with a temporary smoothie-style json string in --arg, until last goal is met
- [ ] Get Smoothie to a working state
- [ ] Add my extra boilerplate debugging args
- [ ] Merge `-cui`, `-input` and `-json` (determine which case scenario)
- [ ] Cleaner code rewrite so there's no nim/c code laying around just for win32 window manipulations
- [ ] Get rid of Python runtime and call VapourSynth's API directly via DLL

This would slim down Smoothie's bundle just to:

- 🗜 `smoothie-0.69.zip`

    - 📂``Smoothie/``
        - 📂``models/`` - rife models
        - 📂``masks/`` - artifact masking for flowblur (and maybe interp soon)
        - 📝 ``recipe.ini``
        - 📂``bin/`` - let me know a more fit name since there's also text files in here
            - 🧋 ``Smoothie.exe`` - this passes all arguments to sm and add `-cui`
            - 💾 ``sm.exe`` - consider that the "core", will be the biggest file out of the two
            - 🗒 ``last_args.txt`` - not really a binary but i prefer it tucked in bin
            - 🗒 ``last_script.vpy``- 
            - ⚙ `VapourSynth.dll` - no idea if anything else is needed, haven't looked into last goal yet
            - ⚙ `msvcp140.dll`, `vcruntime140.dll`, `vcruntime140_1.dll` - dependencies for Rust & VapourSynth
            - 📝 `portable.vs` - tells VapourSynth it's a portable env, remove it for it to use global
            - 📂``vapoursynth64/plugins/``
                - ⚙ ``libvslsmashsource.dll`` - to input videos
                - ⚙ ``RemapFramesVapoursynth.dll`` - for `-padding`
                - ⚙ ``RIFE.dll`` - for low fps interpolation, uses /models/
                - ⚙ ``libmvtools.dll`` - frame motion estimation
                - ⚙ ``svpflow1_vs.dll`` - ' '
                - ⚙ ``svpflow2_vs.dll`` - '
                - ⚙ ``akarin.dll`` - used just for it's expr plugin

## The future of this repository

The replacement process will go like so:

1. ctt/smoothie - new branch called "python-old" created (thus creating an easy to access archive of sm-py)
1. ctt/smoothie - 1 new commit: "removed all python code"
1. ctt/smoothie - 1 new commit: "copied over all rust code"
1. ctt/smoothie-rs - archive repository and link ctt/smoothie in README
