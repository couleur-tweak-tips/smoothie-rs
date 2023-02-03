### Quick clone & compile

You need in path:
* ``git``
* ``cargo`` (see [rustup](https://rustup.rs))
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

### Guidelines

- Do a little `cargo fmt` before commits
- If you're suggesting a new feature try to follow this design (if applicable)
    * Optional so it never gets in the ways of people who do not want to use it
    * If user opt-ins but does not provide anything helpful for X feature, prompt the user
    * Do not prompt the user like in point 2 if he provided "arguments"/info
