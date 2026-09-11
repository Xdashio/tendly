# Development Guide

This document provides instructions for setting up, developing, testing, and building the Tendly desktop application.

## 1. Prerequisites

To build and develop Tendly locally, ensure the following tools are installed:

- **Node.js**: v20 or later (Node 24 recommended).
- **Rust**: Stable toolchain (1.77.2 or later). Install via [rustup](https://rustup.rs).
- **Linux GUI Libraries** (for Linux desktop execution):
  - Ubuntu/Debian: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `build-essential`, `libssl-dev`
  - Fedora: `webkit2gtk4.1-devel`, `gtk3-devel`, `libappindicator-gtk3-devel`, `librsvg2-devel`
  - Arch Linux: `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `librsvg`

---

## 2. Project Architecture Overview

Tendly uses **Architecture B** (direct in-process desktop model):

- **Frontend (`src/`)**: Svelte 5 + Vite + Tailwind CSS.
- **IPC Boundary (`src/lib/api.ts`)**: Type-safe Tauri IPC bindings communicating with Rust core.
- **Backend (`src-tauri/`)**: Rust application core.
  - `src-tauri/src/core/`: Configuration, error handling, structured privacy-safe logging.
  - `src-tauri/src/domain/`: Canonical activity models, classification rules, session states.
  - `src-tauri/src/storage/`: SQLite database manager with WAL mode and reproducible migrations.
  - `src-tauri/src/capture/`: Interfaces and manager for in-process watchers.
  - `src-tauri/src/commands/`: Tauri IPC command handlers.

---

## 3. Development Commands

### Install Dependencies

```bash
npm install
```

### Frontend Development (Browser Preview)

Runs Vite in browser mode (IPC calls return preview fixtures):

```bash
npm run dev
```

### Desktop Application Development (Native Tauri Container)

Runs the desktop application with hot-reloading for both frontend and backend:

```bash
npm run tauri dev
```

---

## 4. Verification and Testing

### Frontend Type Checking

```bash
npm run check
```

### Frontend Unit Tests (Vitest)

```bash
npm test
```

### Frontend Production Build

```bash
npm run build
```

### Rust Unit & Integration Tests

```bash
cd src-tauri && cargo test
```

### Rust Formatting & Linting

```bash
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy -- -D warnings
```

---

## 5. Privacy and Security Policies

1. **Zero Raw Activity Logging**: Default logs must never record window titles, URLs, keystrokes, or personal data.
2. **Local Storage Only**: Database is stored at `~/.local/share/tendly/tendly.db` on Linux with `0600` permissions.
3. **No Open HTTP Ports**: The core desktop application does not bind an HTTP port on `127.0.0.1`. All UI-to-core communication uses Tauri IPC.
4. **No Unsanitized Errors**: Internal database or filesystem paths must not be returned directly across the IPC boundary.
