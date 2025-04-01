<p href="">
    <img src="https://ctt.cx/assets/images/video/smoothie/smoothie-gui.webp" width="220"  align="right">
</p>

<h1 align="center">
    <!-- yup if i put a line break they're not actually centered =( -->
    <img src="https://raw.githubusercontent.com/couleur-tweak-tips/CTT/refs/heads/main/overrides/.icons/custom/smoothie.svg" width=100 /> Smoothie
</h1>
<p align="center">
    add motion blur to videos, with granular configuration
</p>
<p align="center">
    </a>
        <a href="https://ctt.cx/smoothie">
        <img src="https://img.shields.io/badge/Documentation-526CFE?logo=MaterialForMkDocs&logoColor=white" alt="License" />
    </a>
    <a href="https://discord.com/channels/774315187183288411/1051234238835474502">
        <img src="https://img.shields.io/badge/HOF%20render%20tests-white?logo=discord" alt=".gg/CTT render tests" />
    </a>
    <a href="https://www.youtube.com/playlist?list=PLrsLsEZL_o4M_yTqZGwN5cM5ZxJTqkWkZ">
        <img src="https://img.shields.io/badge/Demo%20Playlist-FF0000?logo=youtube" alt="Demo Playlist" />
    </a>
    <a href="https://github.com/couleur-tweak-tips/SmoothieInstaller/releases/latest/download/SmoothieInstaller.exe">
        <img src="https://img.shields.io/badge/Download%20Installer-8A2BE2" alt="Download" />
    </a>
    <a href="https://github.com/couleur-tweak-tips/smoothie-rs/releases/latest/download/smoothie-rs-nightly.zip">
        <img src="https://img.shields.io/badge/Download%20Portable%20zip-8A2BE2" alt="Download" />
    </a>
    <a href="https://github.com/couleur-tweak-tips/smoothie-rs/blob/master/LICENSE">
        <img src="https://img.shields.io/github/license/couleur-tweak-tips/smoothie-rs.svg" alt="License" />
</p>

smoothie-rs is a rewrite of [smoothie](https://github.com/couleur-tweak-tips/smoothie) in rust, find the documentation over on [ctt.cx/smoothie](https://ctt.cx/video/smoothie)

Thanks to tekno, yalter and anima for inspiration

## What is smoothie for?

Smoothie can be used to apply motion blur to video-game footage (or anything really, tho it was designed in mind for fast-paced FPS games), 

it has features similar to [blur's](https://ctt.cx/smoothievsblur),  [VEGAS Pro's frame sampling (frame blending)](https://ctt.cx/recipe#frame-blending), [RSMB](https://ctt.cx/recipe#flowblur), [Flowframes](https://ctt.cx/recipe#pre-interp), and basic video editing capabilities ([cutting](https://github.com/couleur-tweak-tips/suckless-cut), [basic color grading](https://ctt.cx/recipe#color-grading),[LUT](https://ctt.cx/recipe#LUT)..) <!-- [and scaling](https://github.com/user-attachments/assets/4c547387-d39f-44d2-93de-4c88f28cc6c0) -->

It acts as an all-in-one filter chain, you can individually toggle and configure each component however you like via the [recipe](https://ctt.cx/recipe).

## Installer

[Get the latest installer.exe here](https://github.com/couleur-tweak-tips/SmoothieInstaller/releases/latest/download/SmoothieInstaller.exe)

https://github.com/user-attachments/assets/ccf22785-0751-4989-9fa4-dec653d7679a

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


