<!-- Modified by itay7891 (2026) under Apache License 2.0 section 4(b). See HARDENING.md. -->

# Security and privacy

## Mods have your access

A mod is trusted JavaScript running inside Herdr. It can run programs with your environment, fetch URLs and read files.
The built-in file API limits writes to the app home, but a mod can run a program to write elsewhere. That is not a security boundary.
Mods can also run programs on connected machines.

There is no permission prompt for each program or network request. Read a mod before installing it, and treat it like a shell script you would run yourself.
Layouts can load mods. Check those references too.

## What stays on disk

The default app home is `~/.config/herdr-app`, including on macOS and Windows (under your user home).
`XDG_CONFIG_HOME` changes the config root. `HERDR_APP_HOME` selects an app home directly and takes precedence.

This folder holds your settings, keys, layouts, mods, generated agent guide and API types.
It also holds mod data, shared state, window positions, this screen's focus and zoom, update notes and cached media, and telemetry while enabled.
Mods may store sensitive data here. Herdr does not encrypt this folder.
The [installed guide's file map](../defaults/AGENTS.md#the-files) explains the files.

Spaces, tabs and splits are stored in each machine's core document, not in the app home.
Terminals and their output belong to herdr-core. This fork expects a core you build from [itay7891/herdr](https://github.com/itay7891/herdr).
Removing the app home does not stop the core or erase its data.

The app exposes a local control socket in its home so agents can inspect and drive the window.
Treat access to that home as access to the window and its commands.
Remote connections use SSH through the core.

## Updates and signing

This build does not check for updates, download them, or swap in a staged bundle.
`app.check_updates`, `app.update`, and `app.update_core` do not contact a release server.
The app also does not run the core installer or `hc update`.
See [HARDENING.md](../HARDENING.md).

Upstream release builds check a release service and download an update. That client is not used here: the helpers that would call curl return an error unless the crate is built for its unit tests.

## Telemetry

Usage telemetry and feedback upload are compiled out. The app does not collect a daily report, does not keep an install id for one, and does not post to a telemetry server.
`HERDR_TELEMETRY_URL` and `HERDR_FEEDBACK_URL` do not turn sending back on.
The `telemetry` field in settings.json is accepted and ignored.
`telemetry()` tells mods that sending is off because it was compiled out.

## Mods

Mods are JavaScript. While they run they can execute programs, fetch any URL, and read files under the home directory. Writes through the file API stay in the app home; a program the mod runs does not.
Set `"mods": false` in settings.json to stop the window from loading them, or start the app with `HERDR_MODS=0`, which wins over the file.
The sidebar, tabs, menus, and settings screen are mods. Turning them off leaves the terminals without that chrome.

[Back to Herdr](../README.md)
