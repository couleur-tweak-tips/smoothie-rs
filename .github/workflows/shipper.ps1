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
if (-not(test-path ./smoothie-rs/bin/scripts/)){
    mkdir ./smoothie-rs/bin/scripts/
}
cp ./target/scripts/* ./smoothie-rs/bin/scripts/
cp ./target/jamba.vpy ./smoothie-rs/
cp ./target/*.ini ./smoothie-rs/

set-content ./smoothie-rs/launch.cmd -value @'
@echo off
title smoothie-rs
cd /D "%~dp0"
.\bin\smoothie-rs.exe --tui
timeout 1 > nul
'@

set-content ./smoothie-rs/makeShortcuts.cmd -value @'
@echo off
title make smoothie-rs shortcuts
:: https://github.com/couleur-tweak-tips/TweakList/blob/master/modules/Installers/Invoke-SmoothieRsPost.ps1
PowerShell "iex(irm tl.ctt.cx); Invoke-SmoothieRsPost -DIR \"%~dp0\""
echo Shortcuts done
echo.
set /p choice="Add smoothie-rs to PATH? (convenient use from command line) [Y/N]: "
if /i "%choice%"=="Y" (
    setx PATH "%PATH%;%~dp0bin"
	echo Done, do not forget to adjust it if you move smoothie's folder in the future
    echo.
)
echo Feel free to delete or hide this batchfile
timeout 5 > nul
'@

7z a smoothie-rs-nightly.zip ./smoothie-rs