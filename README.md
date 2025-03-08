## smoothie-rs

rewrite of [Smoothie](https://github.com/couleur-tweak-tips/smoothie) in rust, find more info and documentation Smoothie is on [ctt.cx/smoothie](https://ctt.cx/video/smoothie)

Thanks to yalter and anima for inspiration

## installation

## Installer

[Get the latest installer.exe here](https://github.com/couleur-tweak-tips/SmoothieInstaller/releases/latest/download/SmoothieInstaller.exe)

## Portable

It's as simple as extracting the [latest nightly release zip](https://github.com/couleur-tweak-tips/smoothie-rs/releases/latest/download/smoothie-rs-nightly.zip) to a folder and running `launch.bat`

Here is also a short YouTube tutorial with extra tips (Send To & Acquiring RIFE models):

[![thumbnail of smrs installation tutorial youtube video](https://img.youtube.com/vi/RfPDgoMuSWg/maxresdefault.jpg)](https://www.youtube.com/watch?v=RfPDgoMuSWg)

<details><summary>about the now-removed README's roadmap</summary>

Me and [anima](https://github.com/animafps) once considered [developping Smoothie-RS in such a way that it directly made use of VapourSynth as a library via a Rust wrapper](https://github.com/couleur-tweak-tips/smoothie-rs/pull/24) (and [occasionally ditching Python completely](https://github.com/couleur-tweak-tips/smoothie-rs/tree/db8181f7975b057c804b1c1b6fe365de0a7dc13e#roadmap)), but the only benefits I see are:
* Slightly smaller package (50MB doesn't matter for much people nowadays)
* Faster startup times because it wouldn't have to go through VSPipe
* It could also allow more fancy ways to output (e.g render a VSNode once but pipe it to two processes at once?)

And developping it would mean:
* Compiling would be much more complex since it'd be OS-based to link VapourSynth's library
* All of the logic in the easy python code would have to be rewritten in Rust / C 

I don't consider it worth working on nowadays
</details>
