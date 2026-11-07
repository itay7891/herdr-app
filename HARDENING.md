<!-- Modified by itay7891 (2026) under Apache License 2.0 section 4(b). -->

# Hardened herdr-app

This fork of [herdr-app](https://github.com/herdrdev/herdr-app) is for personal use. It keeps the Apache License 2.0 (`LICENSE`, `NOTICE`, `THIRD-PARTY-NOTICES`). Changed files start with a notice that they were modified, as required by Apache License 2.0 section 4(b).

The desktop app is a window. Terminals and agents run in herdr-core (`hc`). This document covers the app. A separate checkout of [itay7891/herdr](https://github.com/itay7891/herdr) is the core to build locally.

## What changed

- **Telemetry is compiled out.** The app does not collect usage counts, write a report, start a telemetry thread, or contact a telemetry server. There is no endpoint in the binary. `HERDR_TELEMETRY_URL`, `DO_NOT_TRACK`, and `"telemetry": false` are not how sending is stopped: there is no send path. `telemetry()` reports `sends: false` and `off_because: "compiled out"`. The `telemetry` key in `settings.json` is still accepted so older files load.
- **Feedback upload is compiled out.** The form can still draft a message and keep screenshots under the app home. `send` returns an error and does not run curl. `HERDR_FEEDBACK_URL` does nothing.
- **No crash reporter.** This tree has no Sentry, crashpad, or minidump upload.
- **App self-update is off.** The background checker does not start. A staged bundle is not swapped in on launch or quit. `app.check_updates`, `app.update`, and `app.update_core` do not download anything. `HERDR_APP_RELEASE` and `HERDR_RELEASES_URL` do not turn checks back on.
- **Release HTTP is not in the app binary.** The functions that would shell out to curl for a manifest, tarball, or changelog media return an error unless the crate is compiled for unit tests (`cfg(test)`). The test build can still talk to a localhost listener. The ignored macOS signature test downloads a release only if you run it with `--ignored`.
- **herdr-core is not downloaded.** The setup screen does not run `curl | sh` or the PowerShell installer. The app does not run `hc update`. On macOS it does not fetch the core bundle installer.
- **Local core is the way in.** Set `HERDR_CORE` to the binary you built. When that variable is set, it is the only candidate: a missing or unsuitable file is not replaced with an `hc` from `PATH` or Homebrew. Otherwise the app uses `settings.json` `"core"`, then `PATH`, then the usual install directories (`herdr-core` or `hc`). A candidate counts only when `--version` prints a line starting with `hc `.
- **Mods can be turned off.** `"mods": false` in `settings.json`, or `HERDR_MODS=0` (also `false`, `off`, `no`). The environment variable wins over the file, so an agent that can edit `~/.config/herdr-app` cannot turn mods back on without changing the process environment. Default is on, because the sidebar, tab bar, menus, and settings screen are mods. With mods off, terminals still attach and mod JavaScript is not loaded. The host grant in that mode has no exec, no network, and no file roots.

### What a running mod can do

Mods are trusted JavaScript (gpui-shell), not a sandbox, while they are enabled:

- `exec(cmd, args)` runs any local program with your environment, or a program on another machine through `hc exec` (`on`).
- `hc(args)` runs the same `hc` binary the app found.
- `fetch` (the shell's HTTP client) may contact any host. WebSocket and TCP follow the same unrestricted network grant.
- `readTextFile` reads text the user can read, locally or on a machine the core can reach. `writeTextFile` is limited to the app home. That limit is not a security boundary: `exec` can write anywhere the user can.
- `watchFolder` watches a local directory.
- Clipboard read and write are granted.
- Layouts name which mods load. A layout edit can point at a new mod.

Shipped mods are rewritten from the binary on each start. Edits to them are moved aside. Mods you add, and clones, stay and hot-reload.

## Build and run on macOS Apple silicon (M4)

These steps use this fork and [itay7891/herdr](https://github.com/itay7891/herdr). They do not download Herdr release zips.

### 1. Tools

- macOS on Apple silicon.
- Xcode Command Line Tools: `xcode-select --install`
- Rust **1.95.0** (the version CI uses):

  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup toolchain install 1.95.0
  rustup default 1.95.0
  ```

- Zig **0.16.0** on `PATH` (the ghostty-vt static library is built with it). Install from [ziglang.org](https://ziglang.org/download/) or your package manager, then `zig version`.
- `just` is optional. The commands below use `cargo` directly.

The rustup installer is a network install of the compiler. It is not part of the app.

### 2. Build herdr-core from the fork

```sh
git clone https://github.com/itay7891/herdr.git "$HOME/src/herdr"
cd "$HOME/src/herdr"
git checkout master
cargo build --release
```

That checkout's package name is `herdr`, so Cargo writes `target/release/herdr`. The desktop app accepts a binary only when `./target/release/herdr --version` prints a line starting with `hc ` (for example `hc 0.1.13`). If the core fork's binary does not, the app will refuse it. After that fork emits a compatible `hc` / `herdr-core` binary, point `HERDR_CORE` at that file. Do not copy a binary from `releases.herdr.dev` or `herdr.dev`.

Check:

```sh
"$HOME/src/herdr/target/release/herdr" --version
```

### 3. Build the app

```sh
git clone https://github.com/itay7891/herdr-app.git "$HOME/src/herdr-app"
cd "$HOME/src/herdr-app"
git submodule update --init --depth 1
cargo build --release
```

Submodules are part of this repository (Ghostty's libvt and the GPUI sources under `vendor/`). `cargo build` compiles them; it does not download a Herdr release.

### 4. Run

```sh
export HERDR_CORE="$HOME/src/herdr/target/release/herdr"
# Optional: keep this window off your everyday config.
# export HERDR_APP_HOME="$HOME/.config/herdr-app-hardened"
# Optional: do not run mods (the chrome is mods; terminals remain).
# export HERDR_MODS=0
"$HOME/src/herdr-app/target/release/herdr-app"
```

Or, without the environment variable, put this in `~/.config/herdr-app/settings.json` (the app creates the file on first launch if it is missing):

```json
{
  "core": "/Users/you/src/herdr/target/release/herdr",
  "mods": true
}
```

`HERDR_CORE` overrides `"core"`. Quit and reopen after changing either one; the core path is read at start.

A Dock-friendly bundle, still built from this tree (ad-hoc signature, no download):

```sh
bash scripts/install-macos
```

That copies `target/release/herdr-app` to `~/Applications/Herdr.app` and links `~/.local/bin/herdr-app`. Quit and reopen to use it. LaunchServices does not pass `HERDR_CORE` unless you set it for the GUI session (for example in `~/.MacOSX/environment.plist` is obsolete; a practical approach is `"core"` in settings.json, which the app reads itself).

### 5. Tests on a Linux or macOS dev machine

From this checkout, with Rust 1.95 and Zig 0.16:

```sh
cargo test --locked
```

CI on Linux also needs the packages listed in `.github/workflows/ci.yml` (pkg-config, clang, X11/Wayland, Vulkan, fontconfig). macOS CI uses the platform SDK.

## Remaining outbound network

What the **application binary** can still cause:

| When | What leaves the machine |
|---|---|
| You click a URL in terminal text | macOS `open` (or `xdg-open` / `ShellExecute` elsewhere) hands the URL to the system. Not automatic. |
| Mods are enabled (the default) | A mod's `fetch` may HTTP any host. `exec` may run `curl`, `ssh`, or anything else. Remote `exec` / `readTextFile` ask the local core to run a command on a machine you added; SSH is the core's. |
| The app starts the core | It runs `<HERDR_CORE> connect` and speaks JSON-RPC on that process's stdin/stdout. Whatever the core binary does on its own (sockets, SSH, its own updater) is outside this repo. This app does not run `hc update`. |
| Agents use `herdr-app do` | A Unix socket under the app home. Local only. |

What this build does **not** do:

- POST usage to `telemetry.herdr.dev`
- POST feedback to `telemetry.herdr.dev`
- GET `releases.herdr.dev` app or core manifests, changelogs, or tarballs
- GET `media.herdr.dev` images
- Pipe `https://releases.herdr.dev/core/install.sh` into a shell
- Pipe `https://releases.herdr.dev/app/install.sh` into a shell from the setup window

Still on disk, **not invoked by the app**:

- `distribution/install.sh` and `distribution/install.ps1` download release artifacts if you run them. Don't.
- `scripts/notes-media.ts` is release tooling for changelog clips.

Other local commands the app runs that are not outbound by themselves: `git` (to learn ignored directories), `codesign` on macOS when installing a CLI symlink, and the core binary. A terminal's own `git push` or package install happens inside the core, as the program you ran.

## Risk that remains

- The core process is a separate program. If it still phones home, this app cannot stop that. Build the core from the fork and set `HERDR_CORE` so a Homebrew or `~/.local/bin` copy is not used by accident.
- With mods on, anything that can edit `~/.config/herdr-app/mods` can run code as you. `"mods": false` in a file those same agents can edit is not a boundary. `HERDR_MODS=0` is, until something can change the environment of the app process.
- The app home holds settings, layouts, mod source, and a control socket. Access to that directory is access to the window.
- Clicking links opens them.
- Unit tests compiled with `cfg(test)` still contain the curl helpers. The `herdr-app` binary you run is not built that way.
