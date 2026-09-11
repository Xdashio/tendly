# Threat Model

This document outlines the threat model for Tendly, identifying potential attackers, the assets we aim to protect, the attack surfaces, and our mitigation strategies. It also clearly defines the boundaries of what Tendly does *not* protect against.

## 1. Threat Actors

We consider the following potential threat actors:
- **Local attacker with physical access:** Can access the unlocked machine and read the SQLite database directly.
- **Malicious software (Malware):** Software running on the same machine that can read files in the user's home directory.
- **Network attacker:** Attempting to connect to local services (e.g., loopback API on 127.0.0.1).
- **Browser extension compromise:** A malicious or compromised browser extension that attempts to exfiltrate data.
- **BYOK Cloud Provider:** The third-party AI provider (e.g., OpenAI, Anthropic) seeing data sent via API.
- **Supply chain attacker:** Introducing malicious code into Tendly through dependencies or the build process.

## 2. Assets to Protect

The primary assets Tendly protects are:
- **Activity History:** Detailed logs of application names, window titles, and URLs (which imply browsing habits and working contexts).
- **User Profile/Goals:** Personal context and objectives provided by the user for AI analysis.
- **Classification Data:** How activities are categorized (e.g., work vs. personal).
- **API Keys (BYOK):** The user's personal API keys used for cloud AI providers.
- **Usage Context:** The very fact that the user is actively monitoring their time and behavior.

## 3. Attack Surfaces

The system exposes several attack surfaces:
- **SQLite Database File:** Stored on the local filesystem, readable by any process running under the user's account.
- **Tauri IPC Bridge:** Channel between the Svelte webview and Rust application core.
- **Ollama API (Local AI):** External local AI service listening on loopback (default 127.0.0.1:11434).
- **BYOK Cloud API Calls:** Outbound HTTPS requests to third-party AI providers (when enabled by user).
- **Browser Extension Native Messaging:** (Future v0.3) Stdio-based communication between browser and desktop app.
- **Auto-Update Mechanism:** (Future feature) Fetching and executing new binaries.

## 4. Mitigations

| Attack Surface | Current Mitigation | Residual Risk | Future Improvement Options |
| --- | --- | --- | --- |
| **SQLite DB** | User-home permissions (0600 on Unix) to prevent other local users from reading. | Malware running as the user can read the file. | SQLCipher encryption (user-provided passphrase). |
| **Tauri IPC** | Strict capability-based permissions; scoped commands; no open network ports. | Compromised webview could call exposed Tauri commands. | Tauri capability boundary audits; minimal command exposure. |
| **Local AI (Ollama)** | Connects to standard local port 11434; structured output parsing. | Rogue local process altering Ollama model outputs. | Model hash verification; token limits. |
| **Browser Extension** | Minimal permissions requested; native messaging validation. | Extension compromise could leak active domains. | Stricter CSP; signed extension builds. |
| **BYOK Cloud APIs** | User provides own key; explicit opt-in; prompt data minimization. | The cloud provider inherently sees the provided prompt context. | Heuristic redaction before sending. |
| **Supply Chain** | MPL-2.0 open source; `cargo-audit` in CI; dependency reviews. | Zero-day malicious dependency updates. | Reproducible builds; vendored dependencies. |

## 5. What Tendly Does NOT Protect Against

We believe in honest security boundaries. Tendly **cannot** and **does not** protect against:
- **Root/Admin Compromise:** If an attacker has root or administrator access, all local protections can be bypassed.
- **Keyloggers or Screen Capture Malware:** Malware already present on the system that monitors input or the display will capture context regardless of Tendly.
- **Physical Access to Unlocked Machines:** If an unauthorized person sits at your unlocked computer, they can read the database or export the data.
- **Untrusted BYOK Providers:** If a user configures Tendly to send data to a malicious or untrusted third-party API, Tendly cannot prevent that service from logging or misusing the data.
- **OS-Level Compromise:** Kernel-level exploits or compromised operating systems.

## 6. Privacy vs Functionality Tradeoffs

Building a time-awareness tool requires balancing privacy with utility. We acknowledge these explicit tradeoffs:
- **Window Titles:** They are essential for accurate classification (e.g., "Main.rs - VSCode" vs. "YouTube"), but are highly privacy-sensitive.
- **Context Depth:** Providing more context to the AI yields much better classification and insights, but requires collecting more potentially sensitive data.
- **Browser URLs:** Knowing the specific domain (or path) is highly valuable for categorizing web activity, but reveals browsing history.
- **AI Prompts:** AI prompts must necessarily contain some activity data to classify it. By relying on local AI (Ollama), we minimize the exposure, but utilizing cloud AI requires sending this context over the network.
