
you need in path:
* ``git``
* ``cargo`` (see [rustup](https://rustup.sh))
and to run Smoothie have FFmpeg in path

In a PowerShell prompt:
```PowerShell
cd $env:TEMP

git clone https://github.com/couleur-tweak-tips/smoothie-rs
cd smoothie-rs
cargo build --release

explorer /select,.\target\release\smoothie-rs.exe
```
> First & last lines are for Windows PowerShell
