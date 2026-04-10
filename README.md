# Tuya Desk Controller

[![Release](https://img.shields.io/github/v/release/dorlanpabon/my_home_smart)](https://github.com/dorlanpabon/my_home_smart/releases)
[![License](https://img.shields.io/github/license/dorlanpabon/my_home_smart)](https://github.com/dorlanpabon/my_home_smart/blob/main/LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-stable-000000)](https://www.rust-lang.org/)

Desktop controller for Tuya and Tuya Smart wall switches, built with `Tauri + Rust + Vite + TypeScript`.

This project focuses on a real gap in the Tuya ecosystem: fast desktop control for multi-gang light switches without depending on the mobile app UI. It uses the official Tuya Cloud API, models each switch channel explicitly, and is structured to grow into local automation and API integrations.

## Release

- Latest tagged release: [`v0.1.0`](https://github.com/dorlanpabon/my_home_smart/releases/tag/v0.1.0)
- Windows installer: [`Tuya Desk Controller_0.1.0_x64-setup.exe`](https://github.com/dorlanpabon/my_home_smart/releases/tag/v0.1.0)

## Why this project exists

Most Tuya desktop workflows are either:

- mobile-first and awkward on PC
- tied to generic device dashboards
- not explicit about multi-gang wall switches
- hard to extend into local automations

Tuya Desk Controller solves that by treating a device as a parent entity with one or more independent channels such as `switch`, `switch_led`, or `switch_1` through `switch_4`.

## Highlights

- Official Tuya Cloud integration, no scraping or reverse engineering
- Explicit multi-gang support for 1, 2, 3, and 4 channel switches
- Per-channel ON/OFF control
- Local credential and preference storage
- Clean desktop UI optimized for fast control
- Alias support for devices and individual channels
- Action log for recent operations
- Modular backend prepared for `automation`, `scheduler`, and `local_api`

## What makes it different

- It is built around wall-switch channels as first-class entities, not just generic device cards.
- It keeps Tuya credentials and request signing in the Rust backend instead of leaking cloud logic into the frontend.
- It is intentionally structured so the same backend can later power a desktop app, local scheduler, and local HTTP API.

## MVP scope

The current release supports:

- saving Tuya Cloud credentials and region settings
- testing connection from the Rust backend
- obtaining and refreshing Tuya access tokens
- listing linked devices
- loading device functions, status, and capabilities
- inferring channel count and controlability
- toggling channels independently
- refreshing device state
- handling common failures such as invalid credentials, expired tokens, offline devices, and partial Tuya responses

## Tech stack

- `Tauri v2`
- `Rust`
- `Vite`
- `TypeScript`
- lightweight vanilla frontend architecture

## Project structure

```text
src/
  components/
  pages/
  services/
  stores/
  styles/
  types/
  utils/

src-tauri/
  src/
    commands/
    config/
    errors/
    future/
    models/
    services/
  capabilities/
  icons/

docs/
  architecture.md
```

## Documentation

- Architecture notes: [docs/architecture.md](/D:/xampp/htdocs/my_home_smart/docs/architecture.md)
- Release artifacts: [releases](/D:/xampp/htdocs/my_home_smart/releases)

## Domain model

The app is built around a normalized domain model instead of rendering raw Tuya payloads directly.

- `Device`
  - `id`
  - `name`
  - `online`
  - `category`
  - `productId`
  - `inferredType`
  - `gangCount`
  - `channels`
  - `raw`
  - `metadata`
- `DeviceChannel`
  - `code`
  - `displayName`
  - `index`
  - `currentState`
  - `controllable`
- `TuyaFunction`
- `TuyaStatus`
- `AppConfig`
- `DeviceAlias`
- `ChannelAlias`

## How channel detection works

Channel inference is centralized in a reusable function:

- `infer_device_channels(...)`

Detection priority:

1. `switch_1`, `switch_2`, `switch_3`, `switch_4`
2. `switch`
3. `switch_led`
4. fallback to boolean switch-like codes discovered in functions, status, or capabilities

Rules:

- if `switch_1` and `switch_2` exist, the device is treated as 2-gang
- if `switch_1` to `switch_3` exist, it is treated as 3-gang
- if `switch_1` to `switch_4` exist, it is treated as 4-gang
- if only `switch` or `switch_led` exists, it is treated as 1 channel
- if a code appears in `status` but not in functions or capabilities, it is shown as detected but not controllable

## Architecture

The core design decision is to keep Tuya-specific logic in Rust and keep the frontend bound only to normalized app models.

Current flow:

- `frontend -> Tauri commands -> Tuya service -> Tuya Cloud`

This keeps:

- secrets out of the frontend
- signature and token handling in one place
- device normalization reusable across future transports

Detailed notes are in [docs/architecture.md](/D:/xampp/htdocs/my_home_smart/docs/architecture.md).

## Local persistence

The app stores data in the application data directory:

- `config.json`
  - Tuya credentials and base URL
- `metadata.json`
  - device aliases, channel aliases, UI preferences
- `actions.jsonl`
  - lightweight local action history

## Requirements

- `Node.js 22+`
- `npm 10+`
- `Rust stable`
- Windows with the MSVC toolchain

Validated locally with `rustc 1.94.1`.

## Getting started

```bash
npm install
npm run tauri dev
```

Build a release:

```bash
npm run tauri build
```

Useful verification commands:

```bash
npm run test
npm run build
cd src-tauri && cargo test
```

## Configuration

Typical values for Tuya Western America:

- `Base URL`: `https://openapi.tuyaus.com`
- `Region label`: `Western America Data Center`

You will also need:

- `Client ID`
- `Client Secret`

If the settings screen is opened later and `Client Secret` is left blank, the app preserves the stored secret instead of overwriting it.

## Roadmap

Near term:

- better refresh behavior and background polling
- more robust endpoint fallback handling
- more fixtures from real Tuya device payloads
- improved offline/degraded mode

Planned architecture extensions:

- `automation`
- `scheduler`
- `local_api`

Longer term:

- local HTTP endpoints for external integrations
- scheduled actions
- simple scenes and rules
- Home Assistant and Node-RED friendly local bridges

## Why the architecture matters

The current backend is intentionally shaped so it can evolve without rewriting the MVP:

- today: `frontend -> Tauri commands -> TuyaService`
- later: `frontend -> commands -> ApplicationService -> TuyaService`
- later: `local_api -> ApplicationService -> TuyaService`
- later: `scheduler -> ApplicationService -> TuyaService`

That avoids duplicating:

- token handling
- request signing
- Tuya error mapping
- channel inference
- device normalization

## Status

This is a real, functional MVP aimed at Tuya wall-switch control from Windows. It is not a UI mockup and not a generic template scaffold.

The strongest current use case is multi-gang switch control with a clean desktop workflow and a backend foundation ready for automation features.
