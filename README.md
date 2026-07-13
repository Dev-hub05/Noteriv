# 🧠 Noteriv + Kriya

### Transforming a Local-First Notes App into an Agent-Native Desktop Application
**Built for Kriyathon 2026**

---

## ⚡ One-Line Pitch
**Noteriv + Kriya** enables humans and AI agents to collaborate inside the same desktop application through typed, governed actions instead of fragile UI automation.

---

## 🚨 The Problem
Today's AI assistants mostly operate outside our software:
*   They **chat** with users in disconnected browser windows.
*   They **generate** text that must be manually copied and pasted.
*   They **automate** actions via fragile UI click-simulators.

They don't truly understand applications because **desktop applications expose capabilities through buttons, not through secure, typed, and auditable AI actions.**

---

## 💡 Our Solution: The Kriya-Native Architecture
By embedding **Kriya** natively into Noteriv, we transformed a local-first markdown editor into an **Agent-Native Local-First Knowledge Workspace**. The agent doesn't click buttons; it interacts with the application's core capabilities via governed, typed actions.

```
                  ┌──────────────┐      ┌──────────────┐
                  │  Human User  │      │   AI Agent   │
                  └──────┬───────┘      └──────┬───────┘
                         │                     │
                         └──────────┬──────────┘
                                    │
                                    ▼
                            [ Typed Actions ]
                                    │
                                    ▼
                            [ Governance Gate ]
                                    │
                                    ▼
                       [ Existing Noteriv Services ]
                                    │
                                    ▼
                          [ Local Markdown Vault ]
```

---

## 🔄 Before vs. After

### Original Noteriv
```
Human User ──> Click Buttons ──> UI Actions ──> Markdown Files
```

### Noteriv + Kriya (Agent-Native)
```
Human User  ──┐
              ├──> Governed Typed Actions ──> Existing App Handlers ──> Markdown Files
AI Agent     ──┘
```

### Why Not Browser/UI Automation?

| Feature / Metric | Browser/UI Automation | Kriya Native |
| :--- | :--- | :--- |
| **Interface** | Clicks pixels, fragile selectors | Direct API calls to typed actions |
| **Stability** | Easily breaks on UI theme or layout updates | Constant, compiler-checked TS/Rust types |
| **Safety Gateways** | Hard to intercept mid-flight | Interactive oneshot approval gates |
| **Compliance** | No audit trail or proof of action | Cryptographically signed transaction receipts |
| **Dependencies** | Requires heavy headless browsers | Native, lightweight local-first execution |

---

## 🏗️ Project Architecture

We designed a unified **Host-Client model** where Rust controls orchestration and React manages application state.

```
       [ React UI / Client ]                 [ Rust Host / Orchestrator ]
 ┌───────────────────────────────┐         ┌──────────────────────────────┐
 │                               │         │                              │
 │   1. TS Action Registry       │         │                              │
 │      (read_note, write_note)  │         │                              │
 │               │               │         │                              │
 │               ▼               │         │                              │
 │       [ Sync Schemas ]────────┼────────>│   2. Schema Verification     │
 │                               │         │              │               │
 │                               │         │              ▼               │
 │                               │         │   3. Governance Engine       │
 │                               │         │      • Permission Checks     │
 │                               │         │      • Approval Gates (UI)   │
 │                               │         │      • Session Budget check  │
 │                               │         │              │               │
 │                               │         │              ▼               │
 │   5. Handler Execution  <─────┼─────────│   4. Signed Audit Receipt    │
 │      (Direct File access)     │         │                              │
 │               │               │         │                              │
 │               ▼               │         │                              │
 │   6. Return Output ───────────┼────────>│   7. Episodic Memory Store   │
 │                               │         │                              │
 └───────────────────────────────┘         └──────────────────────────────┘
```

---

## 🛠️ Kriya Components

| Component | Purpose |
| :--- | :--- |
| **Registry** | Tool discovery. React exports JSON schemas to Rust at startup. |
| **Host** | Governance. Enforces policy, budgets, and oneshot user approval gates. |
| **Dispatcher** | Bridge. Dispatches Rust actions to React and returns results. |
| **Memory** | Persistent Recall. Retains episodic, semantic, and procedural memories in the vault. |
| **Inspector** | Debugging UI. An in-app drawer displaying thought logs, memory query, and approvals. |
| **MCP** | External AI. Exposes Noteriv actions to Cursor, Claude Desktop, and LLM servers. |

---

## 🛡️ Governance Pipeline
Every action initiated by the agent must pass through the governance pipeline before execution:

```
[ Schema Validation ] ──> [ Permission Check ] ──> [ Human Approval Gate ] ──> [ Budget Deduction ] ──> [ Signed Audit Log ] ──> [ Execution ]
```

---

## ⚙️ Action Lifecycle

```
registerAction() [TypeScript]
       │
       ▼
Registry [Rust Cached Metadata]
       │
       ▼
LLM Inference Decision
       │
       ▼
Governance validation (Policy & Budget)
       │
       ▼
Human-in-the-Loop Approval (if Dangerous action)
       │
       ▼
React Action Handler runs
       │
       ▼
Audit Receipt Signed & Saved
       │
       ▼
Episodic Memory Indexed
```

---

## ✨ Core Features

*   **Agent-Native Note Creation**: The agent can create notes, format structures, and append links natively.
*   **AI-Powered Vault Organization**: The agent scans the vault structure, organizes folders, and links related thoughts.
*   **Human Approval Gate**: Intercepts destructive actions (e.g. `delete_note` or `write_note`) and waits for user confirmation in the Kriya Inspector.
*   **Persistent Memory**: Local-first storage (`memory.json`) mapping episodic logs, semantic facts, and discovered workflows.
*   **Audit Trail**: Cryptographic signatures of all actions stored on disk for tracking history.
*   **Local-First Inference**: Seamlessly connects to Ollama instances running on localhost.
*   **MCP Support**: Standardized API allowing external agents to drive Noteriv.

---

## 📝 Demo Walkthrough

### Scenario: Summarize yesterday's notes and create task items.
```
  [User requests agent to summarize vault notes]
                       │
                       ▼
    [Agent: Calls `search_vault` for "yesterday"]
                       │
                       ▼
        [Agent: Reads matches via `read_note`]
                       │
                       ▼
          [Agent: Synthesizes Summary]
                       │
                       ▼
 [Agent: Triggers `write_note` for "Daily Tasks"]
                       │
                       ▼
     ⚠️ [GOVERNANCE GATED: Action is Dangerous]
                       │
                       ▼
   [Inspector displays Approval Popup to Human]
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
        [Approve]             [Deny]
             │                   │
             ▼                   ▼
    [Budget Deducted]      [Step Aborted]
             │
             ▼
    [Create Daily Note]
             │
             ▼
     [Memory Indexed]
```

---

## 💻 Tech Stack & Rationale

*   **Tauri 2**: Secure desktop runtime. Provides isolated environment, Rust native capabilities, and low resource usage compared to Electron.
*   **Rust**: System language. Drives the background Agent Loop, checks governance policies, and performs cryptographic audits.
*   **React**: UI and client state layer. Houses the existing application logic, editor, and the new Kriya Inspector drawer.
*   **JSON-backed Memory Store**: Local-first storage. Designed as pure JSON to bypass native compiler linking dependencies (avoiding common Windows `link.exe` / `rusqlite` setup friction for developers).
*   **Ollama**: Integrates with local llama3/mistral models for 100% offline intelligence.

---

## 📂 Repository Structure
```
desktop/
├── src-tauri/                 # Rust Host Backend
│   ├── src/
│   │   ├── kriya/             # Kriya Core Engine
│   │   │   ├── mod.rs         # Module entrypoint
│   │   │   ├── types.rs       # Communication protocol models
│   │   │   ├── schema.rs      # Argument schema validator
│   │   │   ├── action_registry.rs # Cached actions list
│   │   │   ├── inference.rs   # Ollama / Anthropic client
│   │   │   ├── agent_loop.rs  # Main orchestrator loop
│   │   │   ├── governance.rs  # Gate/Budget policy engine
│   │   │   └── memory.rs      # Local episodic memory
│   │   ├── commands.rs        # Tauri IPC command handlers
│   │   ├── lib.rs             # Tauri lifecycle and plugin registrations
│   │   └── shim.rs            # JS API wrapper injection
│   └── policy.json            # Security classifications policy
├── src/                       # React Frontend
│   ├── components/
│   │   └── KriyaInspector.tsx # Real-time thought, memory, and approval drawer
│   ├── lib/
│   │   ├── kriya-registry.ts  # Client registry
│   │   ├── kriya-dispatcher.ts # Action receiver
│   │   ├── kriya-init.ts      # Action setup hook
│   │   └── kriya-mcp-export.ts # Schema converter
│   └── app/
│       └── page.tsx           # Main workspace layout
```

---

## ⚔️ Challenges & Solutions

### 1. Business Logic Isolation
*   *Challenge*: We wanted Rust to validate actions and manage governance, but duplication of note-saving or search logic in Rust would violate DRY principles and break existing plugins.
*   *Solution*: We built a **Discovery handshake**. On startup, React registers its actions (and schemas) with Rust. When the agent acts, Rust validates the parameters but dispatches the execution back to React via Tauri events, getting the result back through an asynchronous oneshot listener.

### 2. C-Toolchain Linking on Windows
*   *Challenge*: Storing memories in SQLite via `rusqlite` required local compiler toolchains (`link.exe`), which failed on target dev environments missing MSVC SDKs.
*   *Solution*: Designed a lightweight, optimized, pure-Rust JSON-file-backed episodic and long-term memory engine located directly inside the note vault.

---

## 🎓 Lessons Learned
1.  **AI Safety starts at the boundary**: Prompt engineering cannot guarantee an LLM won't try to delete a folder. The only reliable boundary is strict schema validation and permission gatekeeping at the host system level.
2.  **Synchronous feel in an Async architecture**: Awaiting asynchronous Tauri IPC callbacks in a synchronous Rust loop requires robust oneshot channel matching.

---

## 🔮 Future Roadmap
*   **Semantic Vector Memory**: Integrate local vector embedding engines (e.g. `ort` or llama.cpp embeddings) for deep context retrieval.
*   **Multi-Agent Workspaces**: Enable collaboration where separate specialized Kriya agents read different folders of the vault and reconcile daily tasks.
*   **Voice Interactivity**: Add speech-to-action handlers using native whisper bindings.

---

## ❤️ Why This Matters
We believe future desktop applications won't simply "contain AI." They'll expose their capabilities through governed actions, allowing humans and AI to collaborate safely in a shared runtime. **Kriya** provides the architecture to make this possible, and **Noteriv** demonstrates how an existing local-first application can adopt it natively.
