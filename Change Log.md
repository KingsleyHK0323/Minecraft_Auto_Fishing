# Change Log

All notable changes to the Minecraft Auto Fishing tool are documented here.

## Unreleased

## 1.0.3 — 2026-11-18 (Windows)

- Set the Windows installer publisher to Kingsley.
- Upgrading from 1.0.2 has not been verified. The NSIS publisher change also changes the auxiliary install-location registry key from `HKCU\Software\github\Minecraft Auto Fishing` to `HKCU\Software\Kingsley\Minecraft Auto Fishing`, so a custom install path may not be restored automatically. Check the destination before continuing. If uninstalling 1.0.2 first, leave **Delete application data** unchecked to preserve settings and logs. MSI in-place upgrade has not been verified either.

## 1.0.2 — 2026-11-17 (Windows)

- The Windows installer upgrade from 1.0.1 has not been verified; the identifier migration has not been tested through an installed upgrade.
- Updated the Bedrock resource pack author display name to Kingsley, migrated the application identifier to KingsleyHK0323 with a one-time, non-destructive copy of existing settings and log files, and updated the funding links to Kingsley's accounts.
- Updated active website links to the current GitHub Pages address.
- Updated release status and clarified that the in-game test result for menu detection is unconfirmed.
- Updated the app's release update check to use the KingsleyHK0323 repository.
- Licensed project-owned code and resource packs under PolyForm Noncommercial 1.0.0; third-party components retain their respective licenses. Copyright Kingsley.

## 1.0.1 — 2026-10-17 (Windows)

- The Bite Caption resource pack now supports Minecraft: Java Edition 1.21 and later, and Bedrock Edition 1.21 and later.
- Releases ship the resource packs in one zip (`resource-packs.zip`, Bedrock and Java) instead of a separate Bedrock `.mcpack` download; the app can still install them from its Setup tab.

## 1.0.0 — 2026-10-11 (Windows)

First release of the Windows app. Download it from [GitHub Releases](https://github.com/KingsleyHK0323/Minecraft_Auto_Fishing/releases/latest). macOS and Linux are not supported yet. The installer is not code-signed yet, so Windows SmartScreen or Microsoft Defender may warn you.

- Fishing pauses while a game menu (pause, inventory, chat) is open, so clicks never land in a menu, for example splitting item stacks in the Java inventory (not verified in game yet) — 2026-10-11

## 0.1.0-alpha.2 — 2026-10-10 (test pre-release, Windows)

Second test build of the Windows app, for testing only. Download it from [GitHub Releases](https://github.com/KingsleyHK0323/Minecraft_Auto_Fishing/releases). The installer is not code-signed yet, so Windows SmartScreen or Microsoft Defender may warn you.

- Bedrock Edition: the first cast after you return to the game (for example after closing the pause menu with Esc) should no longer be missed (not verified in game yet). Bedrock ignores the first mouse button event after its window gets the focus back, so the app first sends a mouse release that does nothing in the game, and it waits half a second after F8 — 2026-10-10
- The Windows installer shows the app icon instead of the default installer icon — 2026-10-10
- Settings and log files moved out of the install folder, to `%APPDATA%\io.github.spacesquare640.minecraftautofishing` and `%LOCALAPPDATA%\io.github.spacesquare640.minecraftautofishing\logs`. Uninstalling with "Delete the application data" now removes them. Your settings are carried over; old log files in `%LOCALAPPDATA%\Minecraft Auto Fishing\logs` are no longer used and can be deleted — 2026-10-10

## 0.1.0-alpha.1 — 2026-10-10 (test pre-release, Windows)

First test build of the Windows app, for testing only. Download it from [GitHub Releases](https://github.com/KingsleyHK0323/Minecraft_Auto_Fishing/releases). The installer is not code-signed yet, so Windows SmartScreen may warn you (More info › Run anyway).

- First Windows app
  - Bite detection: Java Edition reads the bite subtitle in your game language; Bedrock Edition uses the new Bite Caption resource pack.
  - F8 starts and stops fishing; the app pauses when Minecraft is not the active window.
  - Setup tab installs the Bite Caption resource pack for Java or Bedrock.
  - Optional log files (off by default, kept 7 days) and an update check at startup (can be turned off).
  - If no bite subtitle shows up for several waits in a row, the app suggests what to check (pack, subtitles, casting distance).
  - Bedrock Edition: clicks hold the mouse button briefly, so the game does not miss them (being tested).
  - Pressing F8 after a stop always casts first (reel in before you start).
  - Turning on log files also saves the events of the current session.

## Earlier

- Licensed under GPL-3.0 (`LICENSE` added) — 2026-10-08

<!-- Updated by: OpenAI-Codex -->
