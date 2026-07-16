# 🧠 Kriya Native — Explained Simply
### *For your Kriyathon 2026 Hackathon Presentation*

---

## 📌 The One-Liner

> **Kriya Native** is an architecture that lets an AI agent use your desktop app **the same way you do** — but through secure, typed commands instead of clicking buttons.

Think of it like this: Instead of giving the AI a mouse and hoping it clicks the right things, you give it a **menu of official actions** it's allowed to do, with a **security guard** checking every move.

---

## 🚨 The Problem We're Solving

Today, when AI tries to help you inside a desktop app, it has three bad options:

| Approach | What it does | Why it sucks |
|:---|:---|:---|
| **Chat Window** | Talks to you in a separate browser tab | You have to copy-paste everything manually |
| **Text Generation** | Generates text outside the app | No actual app interaction |
| **UI Automation** | Simulates mouse clicks on buttons | Fragile — breaks when UI changes, no safety controls |

**The core issue**: Desktop apps expose their features through **buttons and menus** (designed for humans), not through **secure, typed APIs** (designed for AI).

---

## 💡 What is "Kriya Native"?

**Kriya Native** means embedding the Kriya agent framework **directly inside** the application, so the AI agent becomes a **first-class citizen** of the app — not an outsider trying to puppeteer it.

### The Analogy 🏠

Imagine your app is a house:

- **UI Automation** = An intruder breaking in through windows, fumbling around in the dark
- **Kriya Native** = A trusted roommate who has their own key, knows the house rules, and signs a logbook every time they enter a room

---

## 🔄 How It Works in Noteriv

**Noteriv** is a local-first markdown notes app built with **Tauri** (Rust backend + React frontend). We embedded Kriya natively into it to demonstrate the architecture.

### Before Kriya (Original Noteriv)
```
Human → clicks buttons → UI updates → files saved to disk
```
Only humans could use the app.

### After Kriya (Agent-Native Noteriv)
```
Human  ──┐
          ├──→ Same governed actions ──→ Same app logic ──→ Same files
AI Agent ──┘
```
Now **both** humans and AI go through the **exact same pipeline**. Neither gets special treatment.

---

## 🏗️ The 6 Building Blocks

Here's each component explained simply:

### 1. 📋 Registry — *"The Menu"*
> *"What can the AI do?"*

At app startup, React tells Rust: *"Here are all the actions I support, along with what inputs each one expects."*

**In Noteriv**, these actions are:
- `read_note` — Read a markdown file
- `write_note` — Create or update a note
- `delete_note` — Remove a note
- `search_vault` — Find notes containing a keyword
- `list_vault` — List all notes in the folder

Each action has a **typed schema** — basically a contract saying *"this action needs a `path` (string) and `content` (string)."* The AI can't send garbage.

---

### 2. 🧠 Agent Loop — *"The Brain"*
> *"Think → Act → Observe → Repeat"*

This is the **ReAct loop** running in Rust:

```
1. Read the user's goal + available actions + past memories
2. Ask the LLM: "What should I do next?"
3. LLM says either:
   → "I'm done, here's the answer" → STOP
   → "Call this action with these args" → CONTINUE
4. Validate → Govern → Execute → Remember
5. Go back to step 2
```

The loop runs for a maximum of ~20 steps per session to prevent runaway agents.

---

### 3. 🛡️ Governance — *"The Security Guard"*
> *"Is the AI allowed to do this?"*

Every single action goes through a **4-checkpoint pipeline** before it can execute:

```
Schema Validation → Permission Check → Human Approval → Budget Check
```

| Checkpoint | What it does |
|:---|:---|
| **Schema Validation** | Are the arguments the right type? (e.g., `path` must be a string) |
| **Permission Check** | Is this action allowed at all? (checks `policy.json`) |
| **Human Approval** | For dangerous actions (write, delete), a popup appears asking the human to approve or deny |
| **Budget Check** | Has the AI exceeded its action limit for this session? |

**In Noteriv's `policy.json`:**
- `read_note` → ✅ Safe, no approval needed, max 50/session
- `write_note` → ⚠️ Dangerous, requires approval, max 10/session
- `delete_note` → ⚠️ Dangerous, requires approval, max 5/session
- `search_vault` → ✅ Safe, no approval needed, max 30/session

---

### 4. 🔗 Dispatcher — *"The Bridge"*
> *"How do Rust and React talk to each other?"*

This solves a key architecture challenge: **Rust handles security, but React handles the actual file operations.**

The flow uses a clever **oneshot channel pattern**:

```
Rust (agent loop)
  → "Hey React, execute read_note with path='daily.md'"
  → Creates a one-time mailbox, waits for reply
  
React (dispatcher)
  → Receives the request
  → Runs the handler (reads the file)
  → Sends the result back through the mailbox
  
Rust
  → Gets the reply, continues the loop
```

This way, business logic stays in React (DRY — no duplicating file operations in Rust), while all governance stays in Rust (secure — React can't bypass checks).

---

### 5. 🧠 Memory — *"The Notebook"*
> *"What has the AI learned?"*

Three types of memory, all stored as a simple JSON file inside your vault:

| Memory Type | What it stores | Example |
|:---|:---|:---|
| **Episodic** | Log of past actions | "On July 15, I searched for 'meeting' and found 3 notes" |
| **Semantic** | Facts and observations | "The user organizes notes by project name" |
| **Procedural** | Learned workflows | "To create a daily summary: search → read → summarize → write" |

The memory file lives at `<vault>/.kriya/memory.json` — it travels with your notes, not stored in some cloud.

---

### 6. 🔍 Inspector — *"The Dashboard"*
> *"What is the AI thinking right now?"*

A real-time side panel in the app showing:

- 💭 **Thought log** — Every reasoning step the agent takes
- ⚡ **Action log** — What actions were called, with what args, and what results
- ⚠️ **Approval gate** — Approve/Deny buttons for dangerous actions
- 🔎 **Memory search** — Query what the agent remembers

Color-coded: 🟢 safe actions, 🟡 pending approval, 🔴 denied.

---

## 📊 The Complete Flow (Visual)

Here's what happens when you ask the agent to *"Summarize yesterday's notes and create task items"*:

```
┌─ YOU ──────────────────────────────────────────────────────────┐
│  "Summarize yesterday's notes and create tasks"               │
└──────────────────────────────────┬─────────────────────────────┘
                                   │
                                   ▼
┌─ AGENT LOOP (Rust) ─────────────────────────────────────────────┐
│                                                                  │
│  Step 1: LLM decides → call search_vault("yesterday")           │
│          ✅ Schema valid  ✅ Safe action  ✅ Budget OK            │
│          → Dispatched to React → React searches files            │
│          → Returns 3 matching notes                              │
│                                                                  │
│  Step 2: LLM decides → call read_note("meeting-notes.md")       │
│          ✅ Schema valid  ✅ Safe action  ✅ Budget OK            │
│          → Dispatched to React → React reads file                │
│          → Returns note content                                  │
│                                                                  │
│  Step 3: LLM synthesizes summary internally                     │
│                                                                  │
│  Step 4: LLM decides → call write_note("daily-tasks.md", ...)   │
│          ✅ Schema valid  ⚠️ DANGEROUS  → Approval Required      │
│          ┌──────────────────────────────────────────────┐        │
│          │  🛡️ APPROVAL POPUP (Inspector)               │        │
│          │                                              │        │
│          │  Agent wants to: write_note                  │        │
│          │  Path: daily-tasks.md                        │        │
│          │  Content: "## Tasks for Today..."            │        │
│          │                                              │        │
│          │  [✅ Approve]          [❌ Deny]              │        │
│          └──────────────────────────────────────────────┘        │
│          → Human approves → Budget deducted                      │
│          → Dispatched to React → React writes file               │
│          → Audit receipt signed and saved                        │
│          → Episodic memory indexed                               │
│                                                                  │
│  Step 5: LLM → Final Answer: "Done! Created daily-tasks.md"     │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 🔐 The Audit Trail

Every action produces a **cryptographic receipt**:

```json
{
  "id": "receipt-a7f3c",
  "session_id": "session-001",
  "action_name": "write_note",
  "args": { "path": "daily-tasks.md", "content": "..." },
  "output": { "success": true },
  "timestamp": "2026-07-15T10:30:00Z",
  "signature": "HMAC-SHA256:a8b3f2..."
}
```

This means you can **prove** what the AI did, when, and with what parameters. No black box.

---

## 🌐 MCP — External Agent Support

Kriya also exposes Noteriv's actions via the **Model Context Protocol (MCP)**:

```
External AI (Cursor, Claude Desktop, etc.)
    ↓
  MCP Protocol (stdio)
    ↓
  Noteriv MCP Server (Node.js)
    ↓
  Same vault, same actions
```

This means tools like Cursor or Claude Desktop can drive your Noteriv vault without even opening the app — using the same typed, governed actions.

---

## 🏛️ Architecture Summary

```
┌────────────────────────────────────────────────────────────┐
│                    NOTERIV + KRIYA                          │
│                                                            │
│  ┌─────────────┐     ┌─────────────────────────────────┐  │
│  │  React (UI) │     │  Rust (Host/Orchestrator)        │  │
│  │             │     │                                   │  │
│  │  Registry   │────→│  Cached Schemas                   │  │
│  │  Dispatcher │←───→│  Agent Loop (ReAct)               │  │
│  │  Inspector  │←────│  Governance (policy.json)         │  │
│  │  Editor     │     │  Inference (Ollama/Anthropic)     │  │
│  │             │     │  Memory (JSON store)              │  │
│  │             │     │  Audit (signed receipts)          │  │
│  └─────────────┘     └─────────────────────────────────┘  │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Local Markdown Vault (files)             │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │       MCP Server (external AI access point)           │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

---

## ⚔️ Why Kriya Native > Everything Else

| | UI Automation | Plugin/API | **Kriya Native** |
|:---|:---|:---|:---|
| **How AI interacts** | Clicks pixels | REST calls | Typed actions inside the app |
| **Breaks when UI changes?** | Yes ❌ | No | No ✅ |
| **Safety controls** | Almost none | API keys only | Schema + Policy + Approval + Budget + Audit |
| **Audit trail** | None | Server logs | Cryptographic receipts on disk |
| **Offline capable** | Needs browser | Needs server | 100% local (Ollama) ✅ |
| **User sees what AI does** | Nothing | Nothing | Real-time Inspector ✅ |

---

## 🎯 Key Takeaways for Your Presentation

> [!IMPORTANT]
> **These are the 5 points that will make judges remember your project:**

1. **"The agent doesn't click buttons — it calls typed actions"** — This is the core differentiator. No fragile UI automation.

2. **"Every action goes through a governance pipeline"** — Schema validation → Permission → Human approval → Budget. Four safety checkpoints before anything happens.

3. **"The human always has the final say"** — Dangerous actions (write, delete) are gated by an in-app approval popup. The AI literally pauses and waits for you.

4. **"Everything is local-first"** — Notes on disk, memory in JSON, LLM via Ollama. No cloud dependency. Your data never leaves your machine.

5. **"The architecture is reusable"** — Kriya isn't just for Noteriv. Any Tauri app (or any desktop app) can adopt this pattern. Today it's a notes app; tomorrow it could be a design tool, a code editor, or a finance dashboard.

---

## 🗣️ Suggested Pitch Flow (3 minutes)

```
[30s] The Problem  → "AI assistants live outside our apps. They chat, they generate text,
                      but they can't actually DO things inside your software safely."

[30s] The Insight  → "What if apps exposed their capabilities as typed, governed actions —
                      so humans and AI use the exact same pipeline?"

[60s] The Demo     → Show: user asks agent to summarize notes → agent searches → reads →
                      APPROVAL POPUP appears → user approves → note created.
                      Show the Inspector panel with real-time thought stream.

[30s] The Safety   → "Every action is schema-validated, permission-checked, budget-limited,
                      and cryptographically signed. The AI can't go rogue."

[30s] The Vision   → "Kriya Native isn't just for notes. It's a pattern for every desktop
                      app. The future is Agent-Native software."
```

---

## 🔑 Glossary (Quick Reference)

| Term | Meaning |
|:---|:---|
| **Kriya** | The agent framework embedded inside the app |
| **Kriya Native** | The architecture pattern of embedding Kriya directly into an app |
| **Action** | A typed, schema-defined capability (e.g., `read_note`) |
| **Governance** | The safety pipeline (schema → permission → approval → budget) |
| **Inspector** | The in-app UI showing real-time agent activity |
| **Receipt** | A signed audit log of each action the agent performed |
| **ReAct Loop** | Think → Act → Observe → Repeat cycle |
| **MCP** | Model Context Protocol — standard for external AI to call app actions |
| **Vault** | The local folder where all markdown notes are stored |
| **Oneshot Channel** | The Rust mechanism for async request-response between Rust and React |
| **Episodic Memory** | Log of past actions the agent has taken |
| **policy.json** | Configuration file defining which actions are safe/dangerous |
