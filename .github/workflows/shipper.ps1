$ErrorActionPreference = "Stop"

mkdir ./smoothie-rs/bin/

curl -L https://github.com/couleurm/VSBundler/releases/latest/download/VapourSynth.7z -o"vapoursynth.7z"
7z x vapoursynth.7z -osmoothie-rs/bin/
mv smoothie-rs/bin/VapourSynth/* smoothie-rs/bin/
rm smoothie-rs/bin/VapourSynth/

$env:VAPOURSYNTH_LIB_DIR=(Get-Item ./smoothie-rs/bin/VapourSynth/sdk/lib64/).FullName
cargo build --release

cp ./target/release/smoothie-rs.exe ./smoothie-rs/bin/
cp ./target/scripts/*.py ./smoothie-rs/bin/VapourSynth
cp ./target/*.ini ./smoothie-rs/

set-content ./smoothie-rs/launch.cmd -value '@echo off & cd /D "%~dp0" & .\bin\smoothie-rs.exe --tui & pause'