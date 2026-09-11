# Tendly

Free, open-source, privacy-first, local-first AI-assisted time-awareness application for software developers.

> **Status:** Phase 0 complete. Architecture and product direction validated. Implementation planning in progress.
> See [docs/PROBLEM_STATEMENT.md](docs/PROBLEM_STATEMENT.md) for background context,
> [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for technical design, and
> [docs/ROADMAP.md](docs/ROADMAP.md) for the phased release schedule.

## Value Proposition

| Characteristic | ActivityWatch | RescueTime / Rize / Drifty | Tendly |
|---|---|---|---|
| Free and open source | Yes | No | Yes |
| AI-classified context | No | Yes | Yes |
| Cross-platform desktop | Yes | Partial (often Mac-first) | Yes (Linux-first MVP, then Win/macOS) |
| Zero mandatory cloud | Yes | No | Yes |

No existing tool combines all four. That is the gap Tendly fills. Detailed reasoning is documented in [docs/PROBLEM_STATEMENT.md](docs/PROBLEM_STATEMENT.md).

## Core Principles

1. **Free, forever, no cloud requirement.** Local-first by default; cloud/BYOK AI is always optional, never required.
2. **Local-first data.** Activity data lives on your machine in a local SQLite database.
3. **Reflection over alarms.** Passive dashboards and hard blocking are weak interventions. Tendly focuses on time awareness, context categorization, and reflection.
4. **Desktop-first from day one.** Built as a native desktop application with system tray integration and low background resource consumption.
5. **Structurally hard to close-source.** Licensed under [MPL-2.0](LICENSE). See [docs/LICENSE_DECISION.md](docs/LICENSE_DECISION.md).

## Phased Roadmap

Phased scope with verified outcomes (see [docs/ROADMAP.md](docs/ROADMAP.md) for complete details):

- [ ] **v0.1** - Linux Desktop MVP: Tauri 2.x GUI, in-process X11/Wayland watchers, AFK detection, SQLite storage, rules-based classification, day timeline UI, system tray. No AI required.
- [ ] **v0.2** - Local AI Classification: Ollama integration (Qwen 2.5:3b), structured outputs, low-confidence review, optional BYOK cloud.
- [ ] **v0.3** - Browser Context: Chrome and Firefox extensions with domain-level tracking and privacy controls.
- [ ] **v0.4** - Visualization & Review: Weekly heatmap, category breakdowns, trends, data export.
- [ ] **v0.5** - Reflection & Intentions: Daily intention check-in, drift reflection prompts, daily review.
- [ ] **v0.6** - Windows Support: Win32 watcher, UWP process handling, session lock detection, Windows installer.
- [ ] **v0.7** - macOS Support: macOS watcher via Accessibility API, permission flow, DMG packaging and notarization.
- [ ] **v0.8+** - Advanced Ecosystem: KDE/GNOME Wayland extensions, plugin API, opt-in graduated friction, multi-device encrypted sync.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) to get started.

## License

[MPL-2.0](LICENSE) for the core codebase.
