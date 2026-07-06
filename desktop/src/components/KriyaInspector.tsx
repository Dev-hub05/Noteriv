"use client";

import { useState, useEffect, useCallback } from "react";

interface StepEntry {
  session_id: string;
  thought: string;
  action_name?: string;
  timestamp: number;
}

interface ApprovalRequest {
  request_id: string;
  action_name: string;
  arguments: any;
}

interface MemoryStats {
  total: number;
  episodic: number;
  semantic: number;
  procedural: number;
}

interface KriyaInspectorProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function KriyaInspector({ isOpen, onClose }: KriyaInspectorProps) {
  const [activeTab, setActiveTab] = useState<"stepgate" | "memory" | "approvals">("stepgate");
  const [steps, setSteps] = useState<StepEntry[]>([]);
  const [approvals, setApprovals] = useState<ApprovalRequest[]>([]);
  const [memoryStats, setMemoryStats] = useState<MemoryStats>({ total: 0, episodic: 0, semantic: 0, procedural: 0 });
  const [memoryQuery, setMemoryQuery] = useState("");
  const [memoryResults, setMemoryResults] = useState<any[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  // Listen for Kriya events
  useEffect(() => {
    if (!isOpen || typeof window === "undefined" || !window.__TAURI__) return;

    const { listen } = window.__TAURI__.event;
    if (!listen) return;

    const unlisteners: (() => void)[] = [];

    // Agent thoughts
    listen("kriya:agent-thought", (event: any) => {
      const { session_id, thought } = event.payload;
      setSteps((prev) => [...prev, { session_id, thought, timestamp: Date.now() }]);
    }).then((u: () => void) => unlisteners.push(u));

    // Action starts
    listen("kriya:agent-action-start", (event: any) => {
      const { session_id, action_name } = event.payload;
      setSteps((prev) => [...prev, { session_id, thought: `Executing action: ${action_name}`, action_name, timestamp: Date.now() }]);
    }).then((u: () => void) => unlisteners.push(u));

    // Agent finished
    listen("kriya:agent-finished", (event: any) => {
      const { session_id, final_answer, error } = event.payload;
      const msg = final_answer || `Error: ${error}`;
      setSteps((prev) => [...prev, { session_id, thought: `✅ ${msg}`, timestamp: Date.now() }]);
    }).then((u: () => void) => unlisteners.push(u));

    // Approval requests
    listen("kriya:request-approval", (event: any) => {
      setApprovals((prev) => [...prev, event.payload]);
    }).then((u: () => void) => unlisteners.push(u));

    return () => { unlisteners.forEach((u) => u()); };
  }, [isOpen]);

  // Fetch memory stats periodically
  useEffect(() => {
    if (!isOpen || !window.electronAPI?.kriyaMemoryStats) return;
    const fetchStats = async () => {
      try {
        const stats = await window.electronAPI.kriyaMemoryStats();
        setMemoryStats(stats);
      } catch {}
    };
    fetchStats();
    const interval = setInterval(fetchStats, 5000);
    return () => clearInterval(interval);
  }, [isOpen]);

  const handleApproval = useCallback(async (requestId: string, approved: boolean) => {
    if (!window.electronAPI?.kriyaSubmitApproval) return;
    await window.electronAPI.kriyaSubmitApproval(requestId, approved);
    setApprovals((prev) => prev.filter((a) => a.request_id !== requestId));
  }, []);

  const handleMemorySearch = useCallback(async () => {
    if (!window.electronAPI?.kriyaMemoryRecall || !memoryQuery.trim()) return;
    setIsSearching(true);
    try {
      const results = await window.electronAPI.kriyaMemoryRecall(memoryQuery);
      setMemoryResults(results);
    } catch { setMemoryResults([]); }
    setIsSearching(false);
  }, [memoryQuery]);

  if (!isOpen) return null;

  return (
    <div style={{
      position: "fixed", right: 0, top: 0, bottom: 0, width: 380,
      background: "var(--bg-primary, #1a1a2e)", borderLeft: "1px solid var(--border, #333)",
      display: "flex", flexDirection: "column", zIndex: 9999,
      fontFamily: "'Inter', -apple-system, sans-serif", color: "var(--text-primary, #e0e0e0)",
      boxShadow: "-4px 0 24px rgba(0,0,0,0.3)",
    }}>
      {/* Header */}
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        padding: "12px 16px", borderBottom: "1px solid var(--border, #333)",
        background: "linear-gradient(135deg, #16213e, #0f3460)",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ fontSize: 18 }}>🧠</span>
          <span style={{ fontWeight: 700, fontSize: 14, letterSpacing: 1 }}>KRIYA INSPECTOR</span>
        </div>
        <button onClick={onClose} style={{
          background: "none", border: "none", color: "var(--text-primary, #e0e0e0)",
          cursor: "pointer", fontSize: 18, padding: 4,
        }}>✕</button>
      </div>

      {/* Approval Banners */}
      {approvals.length > 0 && (
        <div style={{ padding: "8px 12px", background: "#2d1b00", borderBottom: "1px solid #664400" }}>
          {approvals.map((a) => (
            <div key={a.request_id} style={{
              padding: "8px 12px", borderRadius: 8, marginBottom: 6,
              background: "rgba(255,170,0,0.1)", border: "1px solid rgba(255,170,0,0.3)",
            }}>
              <div style={{ fontSize: 12, fontWeight: 600, color: "#ffaa00", marginBottom: 4 }}>
                ⚠️ Approval Required: <code style={{ color: "#fff" }}>{a.action_name}</code>
              </div>
              <pre style={{ fontSize: 11, color: "#aaa", margin: "4px 0", whiteSpace: "pre-wrap", maxHeight: 60, overflow: "auto" }}>
                {JSON.stringify(a.arguments, null, 2)}
              </pre>
              <div style={{ display: "flex", gap: 8, marginTop: 6 }}>
                <button onClick={() => handleApproval(a.request_id, true)} style={{
                  flex: 1, padding: "5px 0", borderRadius: 4, border: "none",
                  background: "#22c55e", color: "#fff", fontWeight: 600, cursor: "pointer", fontSize: 12,
                }}>Approve</button>
                <button onClick={() => handleApproval(a.request_id, false)} style={{
                  flex: 1, padding: "5px 0", borderRadius: 4, border: "none",
                  background: "#ef4444", color: "#fff", fontWeight: 600, cursor: "pointer", fontSize: 12,
                }}>Deny</button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Tab Bar */}
      <div style={{ display: "flex", borderBottom: "1px solid var(--border, #333)" }}>
        {(["stepgate", "memory", "approvals"] as const).map((tab) => (
          <button key={tab} onClick={() => setActiveTab(tab)} style={{
            flex: 1, padding: "10px 0", border: "none", cursor: "pointer", fontSize: 12,
            fontWeight: activeTab === tab ? 700 : 400, letterSpacing: 0.5,
            background: activeTab === tab ? "rgba(99,102,241,0.15)" : "transparent",
            color: activeTab === tab ? "#818cf8" : "var(--text-secondary, #888)",
            borderBottom: activeTab === tab ? "2px solid #818cf8" : "2px solid transparent",
            transition: "all 0.15s ease",
          }}>
            {tab === "stepgate" ? "🔍 Steps" : tab === "memory" ? "💾 Memory" : `🛡️ Gates (${approvals.length})`}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: "auto", padding: "12px 16px" }}>
        {activeTab === "stepgate" && (
          <div>
            {steps.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: "#555", fontSize: 13 }}>
                No agent steps yet. Start a Kriya session to see live execution steps here.
              </div>
            ) : (
              steps.map((step, i) => (
                <div key={i} style={{
                  padding: "8px 10px", borderRadius: 6, marginBottom: 6,
                  background: step.action_name ? "rgba(34,197,94,0.08)" : "rgba(99,102,241,0.08)",
                  borderLeft: `3px solid ${step.action_name ? "#22c55e" : "#818cf8"}`,
                  fontSize: 12, lineHeight: 1.5,
                }}>
                  {step.action_name && (
                    <div style={{ fontSize: 10, fontWeight: 600, color: "#22c55e", marginBottom: 2 }}>
                      ACTION: {step.action_name}
                    </div>
                  )}
                  <div style={{ color: "#ccc" }}>{step.thought}</div>
                  <div style={{ fontSize: 10, color: "#555", marginTop: 4 }}>
                    {new Date(step.timestamp).toLocaleTimeString()}
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === "memory" && (
          <div>
            {/* Stats */}
            <div style={{
              display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: 8, marginBottom: 16,
            }}>
              {[
                { label: "Total", value: memoryStats.total, color: "#818cf8" },
                { label: "Episodic", value: memoryStats.episodic, color: "#22c55e" },
                { label: "Semantic", value: memoryStats.semantic, color: "#f59e0b" },
                { label: "Procedural", value: memoryStats.procedural, color: "#ec4899" },
              ].map((s) => (
                <div key={s.label} style={{
                  padding: "10px 0", textAlign: "center", borderRadius: 6,
                  background: "rgba(255,255,255,0.04)", border: "1px solid rgba(255,255,255,0.06)",
                }}>
                  <div style={{ fontSize: 20, fontWeight: 700, color: s.color }}>{s.value}</div>
                  <div style={{ fontSize: 10, color: "#888", marginTop: 2 }}>{s.label}</div>
                </div>
              ))}
            </div>

            {/* Search */}
            <div style={{ display: "flex", gap: 6, marginBottom: 12 }}>
              <input
                value={memoryQuery}
                onChange={(e) => setMemoryQuery(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleMemorySearch()}
                placeholder="Search memories..."
                style={{
                  flex: 1, padding: "8px 10px", borderRadius: 6, border: "1px solid #333",
                  background: "rgba(255,255,255,0.05)", color: "#e0e0e0", fontSize: 12,
                  outline: "none",
                }}
              />
              <button onClick={handleMemorySearch} disabled={isSearching} style={{
                padding: "8px 14px", borderRadius: 6, border: "none",
                background: "#6366f1", color: "#fff", cursor: "pointer",
                fontSize: 12, fontWeight: 600, opacity: isSearching ? 0.6 : 1,
              }}>
                {isSearching ? "..." : "🔎"}
              </button>
            </div>

            {/* Results */}
            {memoryResults.map((mem: any, i: number) => (
              <div key={i} style={{
                padding: "8px 10px", borderRadius: 6, marginBottom: 6,
                background: "rgba(255,255,255,0.03)", border: "1px solid rgba(255,255,255,0.06)",
                fontSize: 12,
              }}>
                <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 4 }}>
                  <span style={{
                    fontSize: 10, fontWeight: 600, textTransform: "uppercase",
                    color: mem.category === "episodic" ? "#22c55e" : mem.category === "semantic" ? "#f59e0b" : "#ec4899",
                  }}>{mem.category}</span>
                  <span style={{ fontSize: 10, color: "#555" }}>
                    {new Date(mem.timestamp).toLocaleDateString()}
                  </span>
                </div>
                <div style={{ color: "#ccc", lineHeight: 1.4 }}>{mem.content}</div>
              </div>
            ))}
          </div>
        )}

        {activeTab === "approvals" && (
          <div>
            {approvals.length === 0 ? (
              <div style={{ textAlign: "center", padding: 40, color: "#555", fontSize: 13 }}>
                No pending approvals. Actions classified as "dangerous" will appear here for your confirmation.
              </div>
            ) : (
              approvals.map((a) => (
                <div key={a.request_id} style={{
                  padding: "12px", borderRadius: 8, marginBottom: 8,
                  background: "rgba(255,170,0,0.06)", border: "1px solid rgba(255,170,0,0.2)",
                }}>
                  <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 6, color: "#ffaa00" }}>
                    {a.action_name}
                  </div>
                  <pre style={{ fontSize: 11, color: "#aaa", margin: 0, whiteSpace: "pre-wrap" }}>
                    {JSON.stringify(a.arguments, null, 2)}
                  </pre>
                  <div style={{ display: "flex", gap: 8, marginTop: 10 }}>
                    <button onClick={() => handleApproval(a.request_id, true)} style={{
                      flex: 1, padding: "7px 0", borderRadius: 6, border: "none",
                      background: "#22c55e", color: "#fff", fontWeight: 600, cursor: "pointer",
                    }}>✓ Approve</button>
                    <button onClick={() => handleApproval(a.request_id, false)} style={{
                      flex: 1, padding: "7px 0", borderRadius: 6, border: "none",
                      background: "#ef4444", color: "#fff", fontWeight: 600, cursor: "pointer",
                    }}>✗ Deny</button>
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}
