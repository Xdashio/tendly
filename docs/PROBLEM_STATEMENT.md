# Problem Statement

## The problem

People who want to understand where their work time actually goes are stuck
choosing between two bad options:

- **Free but dumb** — tools like ActivityWatch are free, open-source, and
  local-first, but they only give you raw logs (app + window title + time).
  There's no interpretation of *whether that time was actually focused work*,
  and visualization is minimal.
- **Smart but locked** — tools like Rize, Drifty, and RescueTime use AI to
  classify activity as focus vs. distraction and coach you toward better
  habits, but they are closed-source, often cloud-dependent, frequently
  Mac-first, and gated behind subscriptions.

Nobody currently offers the combination of **free, open-source, local-first,
AI-classified, cross-platform (Linux, Windows, and macOS)** time tracking.

## What Tendly is

Tendly is a free, open-source, local-first time tracker that uses AI to
classify activity as **Focus / Neutral / Drift**, and helps people notice
and gently correct drift through reflection and optional friction — not
alarms, not paywalls, not a mandatory cloud account.

## Who it's for

Developers, freelancers, students, and anyone who works alone at a computer
for long stretches and has lost track of how the day actually went — and who
specifically doesn't want to hand that data to a company's cloud, or pay a
subscription for the privilege of understanding their own time.

## What success looks like

Someone installs Tendly, gets automatic tracking with zero manual setup, and
within a few days **trusts the Focus/Neutral/Drift classification enough to
act on it** — without ever being asked to pay, create an account, or send
their data anywhere they didn't explicitly choose to.

Trust in the classification — not installs, not raw hours tracked — is the
metric that matters most, because it's the thing every existing option in
this space is worst at. Passive dashboards make people aware of their time
without making them feel more in control of it; the goal here is control,
not just awareness.

## Platforms

**Linux, Windows, and macOS** are all first-class, primary targets — not an
afterthought and not a "maybe later." The capture layer is designed
(see [ARCHITECTURE.md](./ARCHITECTURE.md)) around a common Watcher interface
specifically so a new OS can be added as a watcher without redesigning
anything else.

In practice, build order will still be staged for practical reasons (no
Mac hardware to test against yet, Wayland compositor support varies), but
all three platforms are in scope for v1.0 — see the roadmap in the main
[README](../README.md).

## What Tendly deliberately does *not* try to be (for now)

- **Not a team/employee-monitoring tool.** This is built for the individual
  who wants insight into their own time, not for managers tracking staff.
  Team features are explicitly out of scope until the individual product is
  solid.
- **Not a project/client billing tool.** No invoicing, no client rate
  tracking. Tools like Toggl Track or Timing already do that well.
- **Not an aggressive blocker by default.** Hard blocking, auto-closing
  tabs, and similar interventions are opt-in only, and off by default — see
  the behavioral rationale in the architecture doc.

## Non-negotiables

These are the constraints that every future feature or architecture
decision gets checked against:

1. **Free, forever, no cloud requirement.** Local AI by default; cloud/BYOK
   is an *option*, never a requirement to get core functionality.
2. **Open source, and structurally hard to turn into a closed paid fork.**
   (See [LICENSE_DECISION.md](./LICENSE_DECISION.md).)
3. **Local-first data.** The user's activity data lives on their machine
   unless they explicitly opt into sync or cloud classification.
4. **Extensible.** A contributor should be able to add a new watcher
   (a new OS, a new editor, a new browser) without touching core code.
