# Contributing to Tendly

Thanks for considering it — this project only works as a community effort.

## Before you write code

Please read, in order:

1. [docs/PROBLEM_STATEMENT.md](docs/PROBLEM_STATEMENT.md) — what we're
   building and why. If a proposed feature doesn't serve this, it's
   probably out of scope (or belongs in a discussion first).
2. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — the layer boundaries and
   interfaces. Most contributions should fit cleanly into one layer
   (a new watcher, a new classification backend, a UI improvement) without
   needing to touch the others.
3. Check open issues and discussions before starting significant work, so
   two people don't build the same thing in parallel.

## Ways to contribute right now (pre-code stage)

- **Watchers** — X11, Wayland, Windows, macOS, browser extensions. Each is
  independent and independently testable. See `spikes/` for starting points.
- **Classification prompt design** — help make the local-model
  Focus/Neutral/Drift labeling actually good. See `spikes/ai-classification/`.
- **Platform testing** — especially macOS and non-wlroots Wayland
  compositors (GNOME, KDE), which nobody on the initial build has hardware
  to test against.
- **Design/UX** — the visualization layer (v0.4+) needs someone who cares
  about this more than the initial builders do.

## Development setup

Coming with v0.1's first real commit — this section will be filled in once
there's an actual build/run process to document. Until then, spikes under
`spikes/` are self-contained and documented individually.

## Pull requests

- Keep PRs scoped to one watcher / one layer / one fix where possible —
  easier to review, easier to merge.
- Reference the architecture doc section your change relates to, if
  applicable.
- New watchers should follow the Watcher contract in
  [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md#1-capture-layer--watchers).

## License of contributions

By contributing, you agree your contribution is licensed under MPL-2.0 (the
project's license — see [docs/LICENSE_DECISION.md](docs/LICENSE_DECISION.md)),
consistent with the rest of the codebase.

## Code of conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md). Please read
it before participating.
