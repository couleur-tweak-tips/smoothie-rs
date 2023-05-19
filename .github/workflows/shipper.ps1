$ErrorActionPreference = "Stop"
Get-Command 7z, curl, cargo -CommandType Application

mkdir ./smoothie-rs/bin/

$curl = (Get-Command -Name curl -CommandType Application).Source | Select-Object -First 1

& $curl -L https://github.com/couleurm/VSBundler/releases/latest/download/VapourSynth.7z -o"vapoursynth.7z"
7z x vapoursynth.7z -osmoothie-rs/bin/
mv smoothie-rs/bin/VapourSynth/* smoothie-rs/bin/
rm smoothie-rs/bin/VapourSynth/

$env:VAPOURSYNTH_LIB_DIR=(Get-Item ./smoothie-rs/bin/sdk/lib64/).FullName
cargo build --release

cp ./target/release/smoothie-rs.exe ./smoothie-rs/bin/
mkdir ./smoothie-rs/bin/scripts/
cp ./target/scripts/* ./smoothie-rs/bin/scripts/
cp ./target/jamba.vpy ./smoothie-rs/
cp ./target/*.ini ./smoothie-rs/

set-content ./smoothie-rs/launch.cmd -value '@echo off & cd /D "%~dp0" & .\bin\smoothie-rs.exe --tui & pause'

7z a smoothie-rs-nightly.zip ./smoothie-rs