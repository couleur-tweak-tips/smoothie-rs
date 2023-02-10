$ErrorActionPreference = "Stop"

function Get-Release{
    param(
        $Repo, # Username or organization/Repository
        $Pattern # Wildcard pattern
    )
    Write-Host "Getting $Pattern from $Repo"
    $Latest = (Invoke-RestMethod https://api.github.com/repos/$Repo/releases/latest -ErrorAction Stop).assets.browser_download_url |
            Where-Object {$_ -Like "*$Pattern"}

    if ($Latest.Count -gt 1){
        $Latest
        throw "Multiple patterns found"
    }
    return $Latest
}

mkdir ./smoothie-rs-artifact/bin/vapoursynth64/

Push-Location ./smoothie-rs-artifact/bin/vapoursynth64/

$Deps = @{
    'svp.7z'     = 'https://github.com/bjaan/smoothvideo/blob/main/SVPflow_LastGoodVersions.7z?raw=true'
    'vs.zip'     = @{ Repo = "AmusementClub/vapoursynth-classic"; Pattern = "release-x64.zip"}
    'akexpr.7z'  = "https://github.com/AkarinVS/vapoursynth-plugin/releases/download/v0.96/akarin-release-lexpr-amd64-v0.96b.7z"#@{ Repo = "AkarinVS/vapoursynth-plugin"; Pattern = "akarin-release-lexpr-amd64-v*.7z"}
    'lsmash.zip' = "https://github.com/AkarinVS/L-SMASH-Works/releases/download/vA.3k/release-x86_64-cachedir-tmp.zip"
    'mvtools.7z' = @{ Repo = "dubhater/vapoursynth-mvtools"; Pattern = "vapoursynth-mvtools-v*-win64.7z"}
    'remap.zip'  = @{ Repo = "Irrational-Encoding-Wizardry/Vapoursynth-RemapFrames"; Pattern = "Vapoursynth-RemapFrames-v*-x64.zip"}
    # 'vsfbd.dll'= @{Repo = "couleurm/vs-frameblender" ; Pattern="vs-frameblender-*.dll"}
    'rife.7z'    = @{Repo = "HomeOfVapourSynthEvolution/VapourSynth-RIFE-ncnn-Vulkan"; Pattern="RIFE-r*-win64.7z"}
}
ForEach ($Dep in [Array]$Deps.Keys){
#    Wait-Debugger
    $Uri = if ($Deps.$Dep -is [String]){
        $Deps.$Dep
    } else {
        Get-Release -Pattern $Deps.$Dep.Pattern -Repo $Deps.$Dep.Repo
    }

    Write-Warning "Downloading $Dep to $Uri"
    curl -s -o $Dep -L $Uri
    $File = Get-Item $Dep
    Set-Variable -Name $File.BaseName -Value $File.FullName
}
"Unzipping SVPFlow"
7z e -y $svp -r svpflow1_vs.dll svpflow2_vs.dll . | Out-Null
$akexpr, $lsmash, $mvtools, $rife, $remap | ForEach-Object {
    "Unzipping $($_ | Split-Path)"
    7z x $_ | Out-Null
}

Pop-Location

7z x $vs -ovapoursynth

$env:VAPOURSYNTH_LIB_DIR=(Get-Item ./vapoursynth/sdk/lib64/).FullName

cargo build --release
cp ./target/release/smoothie-rs.exe ./smoothie-rs-artifact/bin/
cp ./target/*.ini ./smoothie-rs-artifact/

mv ./vapoursynth/vapoursynth64/* ./smoothie-rs-artifact/bin/vapoursynth64/

'msvcp140.dll','vcruntime140_1.dll', 'vcruntime140.dll','VapourSynth.dll', 'portable.vs' | ForEach-Object {
    7z e $vs -osmoothie-rs-artifact/bin/ $PSItem
}

Remove-Item $akexpr, $lsmash, $mvtools, $rife, $remap, $svp, $vs

set-content ./smoothie-rs-artifact/launch.cmd -value '@echo off & cd /D "%~dp0" & .\bin\smoothie-rs.exe --tui & pause'