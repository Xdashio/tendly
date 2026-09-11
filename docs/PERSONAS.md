# Tendly User Personas

## 1. Overview
This document analyzes candidate user personas for Tendly. By evaluating different user segments against Tendly's core pillars (free, open-source, privacy-first, local-first AI-assisted time-awareness), we identify the optimal primary target for the MVP. This focus ensures early adoption, relevant feature development, and organic growth within a highly aligned community.

## 2. Evaluation Framework

Candidate personas were evaluated across five critical dimensions on a 5-point scale:

1. **Problem Intensity**: How acutely do they feel the pain of lost time and context switching?
2. **Auto-Track Value**: How much do they benefit from zero-effort automatic tracking versus manual logging?
3. **FOSS Affinity**: How likely are they to prefer, trust, and contribute to Free and Open-Source Software?
4. **Privacy Need**: How strict are their data privacy requirements (NDAs, proprietary data, corporate policies)?
5. **MVP Reach**: How easily can we reach them through organic, zero-budget distribution channels?

### Scoring Summary

| Persona | Problem Intensity | Auto-Track Value | FOSS Affinity | Privacy Need | MVP Reach | Total |
|---------|:---:|:---:|:---:|:---:|:---:|:---:|
| **Developers** | 5/5 | 5/5 | 5/5 | 5/5 | 5/5 | **25/25** |
| Freelancers | 4/5 | 3/5 | 2.5/5 | 3.5/5 | 3/5 | 16/25 |
| Knowledge Workers | 3.5/5 | 4/5 | 1.5/5 | 3/5 | 2/5 | 14/25 |
| Students | 3/5 | 2/5 | 2/5 | 1.5/5 | 2/5 | 10.5/25 |

---

## 3. Persona 1: Software Developers (Primary)
**Archetype: "The Distracted Engineer"**

*Core Frustration: "I spent 9 hours at my desk but only coded for 1 hour. Where did my day go?"*

* **Demographics & Work Setup**: Full-stack, backend, or data engineers; technical founders. Multi-monitor setups. Heavy usage of terminal, IDEs (VS Code, JetBrains), and browsers.
* **Workflows & Daily Rhythm**: Makers schedule. Deep work requirements punctuated by PR reviews, Slack interruptions, and constant context switching.
* **Pain Points with Time Awareness**: Studies (Gloria Mark, UCI) show a 23-minute recovery time after a context switch. Developers average only ~52 mins/day writing code (Software.com). They experience extreme "timer fatigue" and routinely abandon manual trackers within 3-7 days. Existing tools either fail on coverage (WakaTime is IDE-only), cost/privacy (RescueTime/Rize are cloud-only), or usability (ActivityWatch requires manual regex rules).
* **Distractions & Drift Patterns**: "Yak shaving" (getting sidetracked by tooling/environment issues), endless documentation rabbit holes, Reddit/Hacker News drift during build times, or Slack interruptions.
* **Privacy Expectations**: Extreme. They handle proprietary codebases, NDAs, API tokens, and customer PII. Any cloud-synced tracking of active windows is a non-starter and often violates their employment contracts.
* **Willingness to Install Monitoring**: High, *provided* it is open-source, strictly local, and auditable. Over 80% of developers favor OSS (Stack Overflow).
* **Useful Metrics They'd Want**: Deep work streaks, IDE vs. Browser vs. Communication time, time spent recovering from context switches, "yak shaving" detection.
* **Common Objections**: "Will this slow down my machine?" "Does it send my window titles to a server?" "Is it basically spyware?"
* **Onboarding Friction Points**: Giving macOS Accessibility/Screen Recording permissions; resource footprint of local AI models.
* **Desired Outcomes**: Reclaiming 1-2 hours of deep work daily. Awareness of drift patterns to close distracting tabs. A sense of agency over their time without the overhead of manual logging.

---

## 4. Persona 2: Technical Freelancers (Secondary)

* **Demographics & Work Setup**: Contract developers, technical designers, indie hackers.
* **Workflows & Daily Rhythm**: Juggling multiple clients daily. Frequent shifts between diverse tech stacks and communication channels.
* **Pain Points**: Forgetting to start/stop timers for specific clients leads to underbilling.
* **Distractions & Drift Patterns**: Context switching between different client contexts.
* **Privacy Expectations**: High, to protect multiple clients' confidentiality.
* **Willingness to Install Monitoring**: Moderate to high, as it translates directly to revenue.
* **Useful Metrics They'd Want**: Billable hours per client, granular timeline to reconstruct a timesheet.
* **Common Objections**: "If it can't generate an invoice, I still need another tool."
* **Onboarding Friction Points**: Mapping app usage to specific client projects automatically.
* **Desired Outcomes**: Accurate, effortless timesheets for billing. 
* **Key Misalignment**: Freelancers need billing, invoicing, and complex project allocation—features that distract from Tendly's core mission of *personal time awareness*.

---

## 5. Persona 3: Remote Knowledge Workers (Secondary)

* **Demographics & Work Setup**: Product Managers, Marketers, Operations. Often working on company-issued hardware.
* **Workflows & Daily Rhythm**: Manager's schedule. Meeting-dominated, heavily reliant on Slack, Zoom, Google Workspace, and web apps.
* **Pain Points**: "Zoom fatigue" and days consumed entirely by reactive communication.
* **Distractions & Drift Patterns**: Inbox refreshing, Slack "doom-scrolling," constant context switching between tasks.
* **Privacy Expectations**: Moderate personal expectation, but bound by strict corporate IT policies.
* **Willingness to Install Monitoring**: Low. Suspicious of "bossware" and tracking tools.
* **Useful Metrics They'd Want**: Meeting vs. Focus time, context switching frequency.
* **Common Objections**: "My employer might use this against me."
* **Onboarding Friction Points**: Corporate MDM (Mobile Device Management) policies blocking unauthorized software installations.
* **Desired Outcomes**: Finding blocks of focus time amidst meetings.
* **Key Misalignment**: Blocked by corporate IT; lower FOSS affinity makes zero-budget distribution difficult.

---

## 6. Persona 4: Students (Secondary)

* **Demographics & Work Setup**: University students, coding bootcamp attendees. Laptops, highly mobile.
* **Workflows & Daily Rhythm**: Unstructured time, cramming sessions, intermittent focus.
* **Pain Points**: Procrastination and lack of structured focus habits.
* **Distractions & Drift Patterns**: Predominantly mobile (TikTok, Instagram, YouTube), not just desktop multi-tasking.
* **Privacy Expectations**: Low to Moderate. 
* **Willingness to Install Monitoring**: Moderate, if it helps them study.
* **Useful Metrics They'd Want**: Pomodoro tracking, study streaks, time spent on specific assignments.
* **Common Objections**: "Does this cost money?" "I get distracted on my phone, not my laptop."
* **Onboarding Friction Points**: Understanding the value of granular desktop tracking when their main distractions are mobile.
* **Desired Outcomes**: Better grades through better study habits.
* **Key Misalignment**: The primary source of their distraction (mobile phones) is invisible to a desktop-first tracker like Tendly.

---

## 7. Recommendation

**The primary persona for Tendly must be Software Developers ("The Distracted Engineer").** 

**Rationale:**
1. **Perfect Problem-Solution Fit**: Developers suffer acutely from context switching and "yak shaving." They hate manual tracking ("timer fatigue"). An automatic, AI-assisted tool solves a massive daily pain point.
2. **Privacy as a Feature**: Developers cannot use cloud-based trackers (like RescueTime or Rize) due to NDAs, proprietary codebases, and API keys. A local-first, FOSS tracker is the *only* acceptable architecture for them.
3. **Distribution Advantage**: Tendly can reach developers for $0 through highly engaged, tech-focused communities: Hacker News (Show HN), Reddit (r/programming, r/opensource, r/selfhosted, r/rust), and GitHub Trending.
4. **FOSS Flywheel**: Developers are the only persona likely to contribute back to the codebase (Rust/Svelte), build integrations, and refine the local AI categorization rules.

---

## 8. Implications for MVP

Focusing exclusively on Developers dictates clear boundaries for the MVP:

* **Must-Haves:**
  * **Absolute Privacy**: 100% local operation by default. No cloud telemetry.
  * **Developer Tool Tracking**: Granular tracking for Terminal usage, IDEs (VS Code, IntelliJ, Neovim), and browser tabs (GitHub, StackOverflow vs. Reddit, Twitter).
  * **Resource Efficiency**: Low CPU/RAM overhead, as developers need their machine resources for compiling and running Docker containers.
* **Won't-Haves (Explicit Exclusions):**
  * **Client Billing & Invoicing**: Do not build features for freelancers. Tendly is for awareness, not accounting.
  * **Mobile App**: Do not try to solve student procrastination on TikTok. Stick strictly to desktop (macOS/Linux/Windows).
  * **Enterprise SSO / Admin Dashboards**: Do not build "bossware" or team views. Tendly is single-user, local-first.
