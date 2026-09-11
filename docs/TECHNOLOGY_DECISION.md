# Technology Decision: Desktop Application Framework

## 1. Context

Tendly is a privacy-first, local-first AI-assisted time-awareness desktop application. As a utility that runs continuously in the background, it imposes severe constraints on system resources (idle RAM, CPU usage), requires deep integration with OS-level features (system tray, notifications, autostart, window tracking), and necessitates a local, embedded database for secure telemetry storage. The choice of desktop framework is critical to ensure high performance, low footprint, and robust cross-platform capabilities without compromising user privacy or licensing requirements (MPL-2.0).

## 2. Evaluation Criteria

The following 12 factors were evaluated to determine the best framework:
1. **Idle RAM Footprint**: Must be low (< 50MB preferred) since the app is always on.
2. **System Tray Integration**: Native support for tray icons and background execution.
3. **Packaging & Distribution**: Built-in, reliable packaging for Windows (MSI/NSIS), macOS (DMG/App), and Linux (AppImage/deb).
4. **Local Database (SQLite)**: First-class, zero-friction support for in-process SQLite.
5. **Notifications**: Native system notifications.
6. **Cross-Platform Support**: Windows, macOS, Linux (X11 & Wayland).
7. **License Compatibility**: Must allow for an MPL-2.0 open-source release without forcing restrictive copyleft (like GPLv3) on the entire codebase.
8. **Charting & UI Ecosystem**: Rich data visualization capabilities, essential for time-awareness reporting.
9. **Autostart Capabilities**: Reliable configuration for launching on boot.
10. **OS Permissions & Window Tracking**: Access to low-level APIs for tracking active windows and idle time.
11. **Community & Ecosystem**: Active maintenance, documentation, and plugins.
12. **Developer Experience (DX)**: Tooling, hot-reloading, and ease of cross-compilation.

## 3. Framework-by-Framework Analysis

### 1. Tauri 2.x (Rust + Web)
- **What it is**: A framework for building tiny, blazing fast binaries for all major desktop platforms using Rust for the backend and a webview for the frontend.
- **Strengths**: Extremely low resource usage (15-30 MB idle RAM). Direct access to native OS APIs via Rust. Uses OS-native webviews (WebView2, WebKit), avoiding bundled browser engine bloat. First-party plugins for Tray, Autostart, Notifications, and SQL. Outstanding web charting ecosystem.
- **Weaknesses**: Requires knowledge of both Rust (backend) and web technologies (frontend). Webviews can sometimes have platform-specific quirks.
- **Show-stoppers**: None.
- **Verdict**: **Recommended (9.6/10)**.

### 2. Wails v3 (Go + Web)
- **What it is**: The Go alternative to Tauri, allowing developers to write Go backends and web frontends.
- **Strengths**: Excellent Go developer experience. Low memory footprint (~25-45 MB). Full access to the web charting ecosystem.
- **Weaknesses**: Wails v3 is currently in beta. CGO cross-compilation can introduce significant friction, especially when dealing with native C-libraries like SQLite. Smaller plugin ecosystem compared to Tauri.
- **Show-stoppers**: Beta status and cross-compilation friction for core features.
- **Verdict**: **Strong Contender (8.2/10)**, but Tauri is more mature.

### 3. Electron (Node + Web)
- **What it is**: The industry standard for web-to-desktop apps, bundling Node.js and Chromium.
- **Strengths**: Ubiquitous, massive ecosystem. Identical rendering across platforms due to bundled Chromium.
- **Weaknesses**: High resource usage. An idle Electron app consumes 140-250+ MB of RAM just sitting in the tray. Heavy final binary size.
- **Show-stoppers**: Unacceptable idle RAM footprint for an always-on background utility.
- **Verdict**: **Rejected (7.1/10)**.

### 4. Flutter Desktop (Dart)
- **What it is**: Google's UI toolkit for building natively compiled applications from a single Dart codebase.
- **Strengths**: Consistent UI rendering (uses its own Skia/Impeller engine). Good performance (~60-85 MB RAM).
- **Weaknesses**: Charting libraries are decent but lag far behind the web ecosystem (e.g., ECharts). Non-standard UI paradigms compared to native apps or web apps.
- **Show-stoppers**: System tray, autostart, and notifications all rely heavily on single-maintainer, community-driven plugins, which are high-risk for a core background utility.
- **Verdict**: **Rejected (6.5/10)**.

### 5. Qt 6 (PySide6/C++)
- **What it is**: A comprehensive C++ framework for cross-platform UI development, with bindings for Python (PySide6/PyQt).
- **Strengths**: Extremely mature, powerful native UI components, native OS integration.
- **Weaknesses**: Packaging Python apps (e.g., PyInstaller) is notoriously difficult and brittle. The C++ developer experience is heavy.
- **Show-stoppers**: **GPLv3 traps**. QtCharts and QtGraphs are GPLv3-only under the open-source license. Using them forces the entire application to be GPLv3, violating Tendly's MPL-2.0 decision. PyQt is also GPLv3-only.
- **Verdict**: **Rejected (5.4/10)**.

### 6. GTK4 (Rust/Python)
- **What it is**: The primary toolkit for the GNOME desktop environment.
- **Strengths**: Native Linux look and feel, low memory usage (~30 MB).
- **Weaknesses**: Poor cross-platform support (Windows/macOS versions are second-class citizens). Minimal charting ecosystem.
- **Show-stoppers**: The GNOME Human Interface Guidelines (HIG) actively oppose system tray icons, and GTK4 completely removed native system tray support. For an app that lives in the tray, this is a fatal flaw.
- **Verdict**: **Rejected (3.8/10)**.

## 4. Comparison Matrix

| Rank | Framework | Score | Idle RAM | Tray | License | Charting | Autostart/OS APIs |
|:---:|:---|:---:|:---:|:---:|:---:|:---:|:---:|
| **1** | **Tauri 2.x (Rust + Web)** | **9.6/10** | ~15-30 MB | Native | MIT/Apache (Compatible) | Web 10/10 | Excellent (Rust) |
| 2 | Wails v3 (Go + Web) | 8.2/10 | ~25-45 MB | v3 beta | MIT (Compatible) | Web 10/10 | Good (CGO friction) |
| 3 | Electron (Node + Web) | 7.1/10 | ~140-250+ MB | Native | MIT (Compatible) | Web 10/10 | Excellent (Node) |
| 4 | Flutter Desktop (Dart) | 6.5/10 | ~60-85 MB | Plugin | BSD-3 (Compatible) | Custom 7.5/10 | Risky (3rd-party) |
| 5 | Qt 6 (PySide6/C++) | 5.4/10 | ~25-85 MB | Native | **GPLv3 traps** | GPLv3 4/10 | Good |
| 6 | GTK4 (Rust/Python) | 3.8/10 | ~30 MB | **Removed** | LGPL (Compatible) | None 2/10 | Poor outside Linux |

## 5. ActivityWatch Case Study

The migration path of **ActivityWatch** is highly relevant to Tendly. ActivityWatch, a prominent open-source time tracker, initially built its desktop watcher and UI using Python and Qt. 
- **The Problems**: The Python/Qt approach resulted in a ~300MB+ memory footprint, zombie watcher processes, and "PyInstaller nightmares" for packaging. macOS notarization was particularly torturous.
- **The Solution**: ActivityWatch transitioned to a Rust-based watcher and a web/Tauri-based architecture. 
- **The Result**: Idle RAM usage dropped from ~300MB to ~35MB, packaging became significantly more robust, and cross-platform consistency improved dramatically. 
Tendly avoids the Python/Qt pitfall entirely by learning from this exact scenario and adopting Rust/Tauri from day one.

## 6. Decision: Tauri 2.x

Tauri 2.x is chosen as the desktop framework for Tendly. 

**Justification**:
- **Resource Efficiency**: Meets the strict requirement for <50MB idle RAM, ensuring the app remains invisible to system performance while tracking context in the background.
- **Native APIs**: Tauri 2.x offers a first-class `tauri::tray::TrayIconBuilder` for native tray support, alongside robust official plugins for autostart (`tauri-plugin-autostart`) and notifications (`tauri-plugin-notification`).
- **Data Visualization**: Leverages the unparalleled web ecosystem for charting (e.g., ECharts), crucial for the dashboard.
- **Packaging**: First-class, built-in tooling for AppImage, deb, MSI, NSIS, and DMG generation. No external scripts or PyInstaller hacks required.
- **Licensing Compatibility**: Built primarily on MIT/Apache-2.0 licensed crates, fully compatible with Tendly's MPL-2.0 license.

## 7. Frontend Decision: Svelte 5

While React is popular, **Svelte 5** is chosen for the Tauri frontend.

**Justification**:
- **Zero Virtual DOM**: Svelte compiles to highly efficient vanilla JavaScript, resulting in native DOM reactivity. This minimizes CPU spikes when rendering complex timelines or updating the dashboard.
- **Bundle Size**: Produces significantly smaller bundles than React, leading to faster frontend load times within the Tauri webview.
- **Simplicity & Performance**: Svelte 5's new reactivity model (Runes) provides granular, high-performance state management without the overhead of React hooks or context providers.

## 8. Detailed Technology Stack

- **Shell / Application Framework**: Tauri 2.x (Rust)
- **Frontend Framework**: Svelte 5 + Vite
- **UI Styling**: Tailwind CSS
- **Charting**: Apache ECharts (or Chart.js)
- **Backend / OS Integration**: Rust (stable)
- **Database (In-Process)**: `rusqlite` (binding to SQLite, avoids ABI/CGO issues)
- **Async Runtime**: `tokio` (standard for Rust async ecosystem)
- **Internal Communication**: Direct Tauri IPC (`#[tauri::command]` and events)
- **Platform-Specific Window APIs**:
  - Linux (X11): `x11rb`
  - Linux (Wayland): `wayland-client`
  - Windows: `windows-rs`
  - macOS: `objc2`

## 9. Architecture Note: In-Process vs HTTP API

Tendly evaluated whether an internal HTTP API is necessary.

- **Decision: Direct Tauri IPC (Architecture B)**: For all communication between the Svelte frontend and the Rust backend, direct Tauri IPC is chosen. It requires no open ports on the local network interface, offers high security, eliminates serialization over HTTP sockets, and removes dependencies on web server frameworks like `axum`.
- **External Integrations**: For future integrations (e.g., browser extensions in v0.3 or third-party editor plugins in v0.8), Native Messaging or an opt-in local interface will be provided when those specific features are built. The core desktop application does not include or depend on an internal HTTP server.

## 10. Risks and Mitigations

| Risk | Mitigation |
|:---|:---|
| **Platform Webview Quirks** (e.g., Safari on macOS, Edge on Windows rendering differently). | Stick to standard web APIs, use Tailwind for consistent styling, and test extensively on target platforms. |
| **Rust Learning Curve** for contributors. | Encapsulate complex OS logic in clear, well-documented crates. Rely on Tauri plugins for common tasks to minimize custom Rust code where possible. |
| **SQLite Concurrency** issues with multiple local threads/clients. | Use a dedicated database thread or standard connection pooling (`r2d2` or `deadpool-sqlite`) to serialize writes while allowing concurrent reads via WAL mode. |

## 11. Reversibility

**Moderate-to-High Reversibility.** 
The architecture strongly separates the Rust backend (window tracking, database, AI rules) from the frontend (Svelte/Tailwind). 
- If Tauri proves unviable, the Rust backend can be exposed entirely via HTTP, and a different frontend framework (e.g., Electron, Wails) could consume the local API. 
- If Svelte is rejected later, the Tauri Rust backend remains intact, and the webview can simply load a new React or Vue application. 
- The core logic (window tracking, SQLite schemas) is entirely portable and framework-agnostic.
