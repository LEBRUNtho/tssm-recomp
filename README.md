# tssm-recomp

A static recompilation of **The SpongeBob SquarePants Movie** (GameCube, USA, GGVE78 revision 1)
to native Windows (and Android), written in Rust. This fork adds iPhone and iPad, macOS as a
development target, and the European French and Dutch version (GGVX78).

**This repository contains no game code or data.** You need your own disc image, made from a
disc you own. The builder reads the game's PowerPC code from it, translates every function to
Rust, and compiles a native executable on your computer.

## Features

- Runs natively: no emulator. The GPU's commands are drawn with wgpu (Vulkan).
- Any frame rate, set in the game's own settings screen (60 to 500), with the game running at
  the same speed; the driving levels' car is stepped at a fixed rate so it handles the same.
- Field of view setting (60 to 110), widescreen and ultrawide.
- Keyboard and mouse (mouse look, clickable menus) and gamepads.
- With a texture pack, sharper text and sky flowers too: upscaled with smooth edges.
- Texture packs in Dolphin's format: put the zip next to the game and it asks whether to use it
  (or put it in the `textures` folder).

## Building

1. Download this repository (Code, Download ZIP) and unzip it, keeping the folders together.
2. Double-click `build.bat` and choose your disc image (`.iso` or `.gcm`).
3. If Rust or the Visual Studio C++ build tools are missing, it offers to download and install
   them (answer `Y`).
4. It offers the Xbox version's textures, a separate download
   ([GameBanana](https://gamebanana.com/mods/648590)): answer `Y` and choose the downloaded zip.
5. When it is done, start **The SpongeBob Movie Game.exe**, which appears next to `build.bat`.

From a terminal: `powershell -ExecutionPolicy Bypass -File tools\build.ps1 -Iso <image>`
(`-Dev` for a quicker-to-compile build, `-Force` to redo every step, `-Yes` to install missing
tools without asking). The first build takes a while; later ones are quick.

The Windows builder accepts GGVE78 revision 1 only.

## iPhone and iPad

`tools/ios.sh` builds the app on a Mac, from the American disc (GGVE78 revision 1) or the
European French and Dutch one (GGVX78 revision 0):

1. Install Xcode and [Rust](https://rustup.rs/), then `rustup target add aarch64-apple-ios`.
2. `tools/ios.sh <disc image .iso or .gcm> ipa`
3. Install `build/ios/SpongeBobMovie-<version>.ipa` with a sideloading tool (AltStore,
   Sideloadly, Feather...), or see the top of the script for signing with your own certificate.

The app carries your disc's files, so it is for your own devices only: do not share it.

- Touch controls on screen (the eye button hides them), or a controller (MFi, PlayStation,
  Xbox, Switch Pro); no jailbreak and no JIT.
- The European version plays in the phone's language, French or Dutch.
- Saves, settings and the log are in the Files app, under the app's name.
- Other disc image formats (.rvz, .ciso...) have to be converted to .iso first, for example
  with Dolphin or [nod](https://github.com/encounter/nod).

The port's knowledge of the game's code is by symbol (`config/addresses.txt`), which the
recompiler looks up in each version's ELF: that is what lets one host build against both.

## Layout

| Path | What it is |
|---|---|
| `crates/recomp` | The recompiler (PowerPC ELF to Rust) and the disc extractor |
| `crates/runtime` | Guest memory and CPU state the generated code runs on |
| `crates/game` | Where the generated code goes (`src/gen`, written by the build, not in the repo) |
| `crates/tssm` | The host: graphics, audio, input, memory card, disc, and the port's changes |
| `crates/android` | The Android app |
| `tools/ios.sh`, `tools/ios` | The iPhone and iPad builder |
| `config` | Controls, window and frame rate settings, the list of replaceable SDK functions, and the game symbols the host uses |
| `tools` | The builder (`build.ps1`) and the package maker (`package.ps1`) |

`CHANGELOG.md` records what has changed and how it was checked.

## Legal

This project is not affiliated with or endorsed by Nickelodeon, THQ, Heavy Iron Studios or
Nintendo. SpongeBob SquarePants is a trademark of Viacom International Inc. No copyrighted game
code or assets are distributed here; you must supply your own legally obtained copy of the game.
