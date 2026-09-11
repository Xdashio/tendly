# License Decision

## Requirement

From the [problem statement](./PROBLEM_STATEMENT.md): Tendly must be "open
source, and structurally hard to turn into a closed paid fork." That's the
lens for everything below — we're not picking a license in the abstract,
we're picking one that protects that specific promise while staying
friendly to contributors.

## Options considered

### MIT

- **Pros:** Maximum simplicity, maximum contribution ease, no friction for
  anyone who wants to embed Tendly code anywhere. Very familiar to most
  developers.
- **Cons:** Zero protection against the exact failure mode we're worried
  about — anyone can fork the whole project, close the source, and sell it
  as a hosted product, with no obligation to contribute anything back.
- **Verdict:** Rejected. Too weak for a project whose whole pitch is "we
  stay free," when a fork could stop being free the moment it's profitable.

### GPL-3.0

- **Pros:** Strong copyleft — any distributed derivative work must also be
  fully open source under GPL. Well understood, battle-tested, widely used
  for end-user applications.
- **Cons:** "Distributed" is the key word — GPL's copyleft is triggered by
  *distributing* the software. If someone runs a modified version purely as
  a web service without distributing the binary, GPL's obligations don't
  clearly apply (the so-called "SaaS loophole").
- **Verdict:** Good fit for a desktop app specifically, since we're not
  planning to ship this as a hosted service in the near term. But the SaaS
  loophole matters for one specific piece of the roadmap: the optional
  future sync server (v0.8 in the roadmap).

### AGPL-3.0

- **Pros:** Same as GPL, plus it closes the SaaS loophole — if someone runs
  a modified version as a network service that users interact with
  remotely, they're required to make the modified source available to
  those users. This is the strongest protection against someone taking the
  code and running it as a closed paid cloud product.
- **Cons:** That specific extra protection (the network clause) is mostly
  irrelevant to *this* codebase as it stands today, because Tendly is a
  desktop app, not a server. AGPL is also the license some companies are
  most wary of when deciding whether to contribute to or depend on a
  project, because the network clause is unusual and sometimes
  misunderstood — this can quietly reduce contributions and adoption.
- **Verdict:** The right license for the *optional sync server* specifically,
  if/when it's built — not obviously the right default for the whole
  desktop-app codebase today.

### MPL-2.0

- **Pros:** File-level copyleft — modifications to MPL-licensed files must
  themselves stay open, but MPL code can be combined with proprietary code
  in a larger work without forcing the whole work open. This is a
  deliberate middle ground: it protects the core project from a closed
  fork of *this code*, while staying friendly to companies or contributors
  who might want to embed a watcher or plugin alongside proprietary code.
  It is also **the exact license ActivityWatch itself uses** — the closest
  comparable project in this space — which means the license will already
  be familiar to exactly the contributor community most likely to find and
  work on Tendly.
- **Cons:** File-level (not project-level) copyleft is slightly weaker than
  GPL/AGPL in the abstract — someone could technically add a large amount
  of new proprietary code around unmodified MPL files. In practice this
  is a minor risk for an app like this.
- **Verdict:** **Selected as the primary license for the codebase.**

## Decision

- **Core Tendly codebase (capture, storage, classification, UI): MPL-2.0.**
  Matches the closest direct comparison project (ActivityWatch), keeps
  modifications to the code open, and doesn't scare off contributors or
  companies who might want to build on top of it.
- **Optional self-hosted sync server (future, v0.8+): AGPL-3.0**, licensed
  separately when it's built, specifically because that component *is* a
  network service, and that's exactly the scenario AGPL's network clause
  exists for.

This mirrors a pattern several serious open-source projects use: permissive
enough to grow a community, strict enough at the point that actually
matters (a hosted, network-facing component) to prevent the "free core,
paid closed clone" failure mode.

## What this means in practice for contributors

- You can fork Tendly, modify it, and even sell a modified version — but if
  you distribute the modified code, the modifications to MPL-covered files
  must be released under MPL-2.0 too.
- You *can* combine Tendly's code with your own proprietary code in a
  larger work (e.g., a proprietary plugin), as long as you don't modify
  Tendly's own files without releasing those specific changes.
- This is not legal advice — if you're planning a commercial use case
  that's anywhere close to the line, talk to a lawyer, not this document.
