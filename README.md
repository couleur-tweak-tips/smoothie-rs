## smoothie-rs

Rewrite of [Smoothie](https://github.com/couleur-tweak-tips/smoothie) in Rust, for now you can find more info and documentation about Smoothie in [the old python repo](https://github.com/couleur-tweak-tips/smoothie)

Thanks to yalter and anima for inspiration

## installation

It's as simple as extracting the [latest nightly release zip](https://github.com/couleur-tweak-tips/smoothie-rs/releases/latest/download/smoothie-rs-nightly.zip) to a folder and running `launch.bat`

There is also a short YouTube tutorial with extra tips (Send To & Acquiring RIFE models):

[![thumbnail of smrs installation tutorial youtube video](https://img.youtube.com/vi/RfPDgoMuSWg/maxresdefault.jpg)](https://www.youtube.com/watch?v=RfPDgoMuSWg)

<details><summary>about the now-removed README's roadmap</summary>

Me and [anima](https://github.com/animafps) once considered [developping Smoothie-RS in such a way that it directly made use of VapourSynth as a library via a Rust wrapper (and occasionally ditching Python completely)](https://github.com/couleur-tweak-tips/smoothie-rs/pull/24), but the only benefits I see are:
* Slightly smaller package (50MB doesn't matter for much people nowadays)
* Faster startup times because it wouldn't have to go through VSPipe
* It could also allow more fancy ways to output (e.g render a VSNode once but pipe it to two processes at once?)

And the challenges would be:
* Compiling would be much more complex since it'd be OS-based to link VapourSynth's library
* All of the logic in the easy python code would need to be rewritten in Rust / C 

I don't consider it worth working on nowadays
</details>
