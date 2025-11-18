import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface LogEntry {
  time: string;
  message: string;
  type: 'success' | 'error' | 'info' | 'warning';
}

function App() {
  const [f5Count, setF5Count] = useState(0);
  const [lastPressTime, setLastPressTime] = useState<string>("");
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const addLog = (message: string, type: LogEntry['type'] = 'info') => {
    const time = new Date().toLocaleTimeString();
    setLogs(prev => [...prev.slice(-49), { time, message, type }]);
  };

  useEffect(() => {
    // Check permission on load
    invoke('plugin:macos-input-monitor|check_permission')
      .then((granted) => {
        setHasPermission(granted as boolean);
        if (granted) {
          addLog('Input Monitoring permission: GRANTED', 'success');
        } else {
          addLog('Input Monitoring permission: NOT GRANTED', 'error');
        }
      })
      .catch((e) => {
        addLog(`Permission check failed: ${e}`, 'error');
        setHasPermission(false);
      });

    // Listen for F5 press events from plugin (via backend)
    const unlisten = listen<number>("f5-frontend", (event) => {
      console.log("F5 pressed event received:", event.payload);
      setF5Count(event.payload);
      setLastPressTime(new Date().toLocaleTimeString());
      addLog(`F5 pressed! Count: ${event.payload}`, 'success');
    });

    // Load initial count
    invoke<number>("get_f5_count").then((count) => {
      setF5Count(count);
      addLog(`Initial count: ${count}`, 'info');
    });

    addLog('App initialized - press F5 anywhere!', 'info');

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const openSettings = () => {
    invoke('plugin:macos-input-monitor|open_input_monitoring_settings')
      .then(() => addLog('Opened System Settings', 'info'))
      .catch((e) => addLog(`Failed to open settings: ${e}`, 'error'));
  };

  const testPermission = () => {
    addLog('Testing permission...', 'info');
    invoke('plugin:macos-input-monitor|check_permission')
      .then((granted) => {
        setHasPermission(granted as boolean);
        if (granted) {
          addLog('✅ Permission test PASSED!', 'success');
        } else {
          addLog('❌ Permission test FAILED - If permission is ON in Settings, RESTART the app!', 'error');
        }
      });
  };

  return (
    <main className="container" style={{ padding: "20px", maxWidth: "1400px", margin: "0 auto" }}>
      {/* Header */}
      <div style={{ textAlign: "center", marginBottom: "24px" }}>
        <h1 style={{ fontSize: "36px", margin: "0" }}>🎯 F5 Global Listener</h1>
        <p style={{ color: "#999", margin: "8px 0 0 0", fontSize: "14px" }}>
          macOS Input Monitor Plugin Example - CGEventTap Hardware-Level Interception
        </p>
      </div>

      {/* Main Grid Layout */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "20px", marginBottom: "16px" }}>
        
        {/* Left Column: Status & Counter */}
        <div>
          {/* Permission Status */}
          <div style={{ 
            padding: "16px", 
            border: "2px solid #646cff", 
            borderRadius: "8px",
            backgroundColor: "#2a2a2a",
            marginBottom: "16px"
          }}>
            <h3 style={{ margin: "0 0 12px 0", fontSize: "18px" }}>System Status</h3>
            
            <div style={{ display: "flex", alignItems: "center", gap: "10px", fontSize: "14px" }}>
              <div style={{
                width: "10px",
                height: "10px",
                borderRadius: "50%",
                background: hasPermission === null ? "#eab308" : hasPermission ? "#22c55e" : "#ef4444",
                boxShadow: hasPermission === null ? "0 0 8px #eab308" : hasPermission ? "0 0 8px #22c55e" : "0 0 8px #ef4444"
              }} />
              <span>
                {hasPermission === null ? "Checking..." : hasPermission ? "Permission Granted ✅" : "Permission Denied ❌"}
              </span>
            </div>

            {hasPermission === false && (
              <div style={{ marginTop: "12px", padding: "12px", background: "#3a1a1a", borderRadius: "6px", fontSize: "13px" }}>
                <button onClick={openSettings} style={{
                  background: "#646cff",
                  color: "white",
                  border: "none",
                  padding: "8px 16px",
                  borderRadius: "6px",
                  cursor: "pointer",
                  fontSize: "13px",
                  width: "100%"
                }}>
                  Open System Settings
                </button>
                <p style={{ margin: "8px 0 0 0", color: "#999", fontSize: "12px" }}>
                  Grant Input Monitoring permission, then restart app
                </p>
              </div>
            )}
          </div>

          {/* F5 Counter */}
          <div style={{ 
            padding: "20px", 
            border: "2px solid #646cff", 
            borderRadius: "8px",
            backgroundColor: "#2a2a2a",
            textAlign: "center"
          }}>
            <h3 style={{ margin: "0 0 16px 0", fontSize: "18px" }}>F5 Press Count</h3>
            <div style={{ fontSize: "72px", fontWeight: "bold", color: "#646cff", lineHeight: "1", margin: "16px 0" }}>
              {f5Count}
            </div>
            <div style={{ color: "#999", fontSize: "14px" }}>
              {lastPressTime ? `Last: ${lastPressTime}` : "Waiting..."}
            </div>
          </div>
        </div>

        {/* Right Column: Event Log */}
        <div style={{ 
          padding: "16px", 
          border: "2px solid #646cff", 
          borderRadius: "8px",
          backgroundColor: "#2a2a2a",
          display: "flex",
          flexDirection: "column"
        }}>
          <h3 style={{ margin: "0 0 12px 0", fontSize: "18px" }}>Event Log</h3>
          
          <div style={{
            background: "#1a1a1a",
            border: "1px solid #444",
            borderRadius: "6px",
            padding: "12px",
            flex: "1",
            minHeight: "280px",
            maxHeight: "280px",
            overflowY: "auto",
            fontFamily: "monospace",
            fontSize: "12px"
          }}>
            {logs.map((log, i) => (
              <div key={i} style={{ 
                padding: "3px 0",
                color: log.type === 'success' ? '#22c55e' : log.type === 'error' ? '#ef4444' : log.type === 'warning' ? '#eab308' : '#3b82f6'
              }}>
                <span style={{ color: "#666", fontSize: "11px" }}>{log.time}</span> {log.message}
              </div>
            ))}
          </div>

          <div style={{ marginTop: "10px", display: "flex", gap: "8px" }}>
            <button onClick={() => setLogs([])} style={{
              background: "#444",
              color: "white",
              border: "none",
              padding: "8px 16px",
              borderRadius: "6px",
              cursor: "pointer",
              fontSize: "13px",
              flex: "1"
            }}>
              Clear Logs
            </button>
            <button onClick={testPermission} style={{
              background: "#444",
              color: "white",
              border: "none",
              padding: "8px 16px",
              borderRadius: "6px",
              cursor: "pointer",
              fontSize: "13px",
              flex: "1"
            }}>
              Test Permission
            </button>
          </div>
        </div>
      </div>

      {/* Instructions Footer */}
      <div style={{ 
        padding: "12px 16px", 
        backgroundColor: "#2d2d2d", 
        borderRadius: "8px",
        fontSize: "13px",
        lineHeight: "1.6"
      }}>
        <strong>Quick Start:</strong> Press <code style={{ background: "#1a1a1a", padding: "2px 6px", borderRadius: "3px" }}>F5</code> anywhere → 
        Counter increments, sound plays, dictation blocked! 
        <span style={{ color: "#666", marginLeft: "12px" }}>Uses CGEventTap with HeadInsertEventTap</span>
      </div>

      <div style={{ 
        padding: "15px", 
        backgroundColor: "#2d2d2d", 
        borderRadius: "8px",
        textAlign: "left",
        marginTop: "20px"
      }}>
        <h3>📋 Instructions</h3>
        <ol style={{ lineHeight: "1.8" }}>
          <li><strong>Press F5</strong> anywhere on your Mac (even in other apps!)</li>
          <li>You'll hear a system sound and see the counter update</li>
          <li><strong>macOS dictation will NOT start</strong> - F5 is intercepted first!</li>
          <li>Required permission:
            <ul style={{ marginTop: "8px", fontSize: "14px" }}>
              <li><strong>Input Monitoring:</strong> System Settings → Privacy & Security → Input Monitoring
                <br/>→ Enable "F5 Global Listener"</li>
              <li>⚠️ <strong>Must restart the app</strong> after granting permissions</li>
            </ul>
          </li>
        </ol>
      </div>

      <div style={{ 
        padding: "15px", 
        backgroundColor: "#1a2b1a", 
        borderRadius: "8px",
        marginTop: "20px",
        border: "1px solid #2d4a2d"
      }}>
        <h4 style={{ margin: "0 0 10px 0", color: "#7fc87f" }}>🔧 Technical Details</h4>
        <p style={{ fontSize: "13px", lineHeight: "1.6", color: "#b0b0b0", margin: 0 }}>
          Uses <strong>CGEventTap</strong> with <code>HeadInsertEventTap</code> to intercept F5 
          at the lowest system level, <em>before</em> macOS dictation sees it. The callback 
          strips internal macOS flags (SecondaryFn) and only checks for intentional modifiers 
          (Cmd/Opt/Ctrl/Shift), then returns <code>nil</code> to consume the event completely.
          <br/><br/>
          <strong>Same technique as BetterTouchTool!</strong>
        </p>
      </div>

      <p style={{ marginTop: "30px", fontSize: "14px", color: "#888" }}>
        This app demonstrates low-level keyboard interception on macOS using CGEventTap
      </p>
    </main>
  );
}

export default App;
