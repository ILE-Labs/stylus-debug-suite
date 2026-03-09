/// <reference path="./shims.d.ts" />
import * as vscode from "vscode";
import { spawn } from "child_process";
import * as path from "path";
import * as fs from "fs";

export function activate(context: vscode.ExtensionContext) {
  const output = vscode.window.createOutputChannel("Stylus Debug Suite");

  const startDisposable = vscode.commands.registerCommand(
    "stylusDebugSuite.startDebugSession",
    async () => {
      output.show(true);
      output.appendLine("Starting Stylus debug session (demo)...");

      const folder = vscode.workspace.workspaceFolders?.[0];
      if (!folder) {
        vscode.window.showErrorMessage("Open a workspace folder first.");
        return;
      }

      const proc = spawnAdapter(folder.uri.fsPath, output);
      const responses = await runDemoProtocol(proc, output);
      output.appendLine("Session complete.");
      output.appendLine(JSON.stringify(responses, null, 2));
    }
  );

  const dashboardDisposable = vscode.commands.registerCommand(
    "stylusDebugSuite.openDashboard",
    async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      if (!folder) {
        vscode.window.showErrorMessage("Open a workspace folder first.");
        return;
      }

      output.show(true);
      output.appendLine("Opening Stylus Debug Suite dashboard...");

      const proc = spawnAdapter(folder.uri.fsPath, output);
      const responses = await runDemoProtocol(proc, output);

      const panel = vscode.window.createWebviewPanel(
        "stylusDebugSuite.dashboard",
        "Stylus Debug Suite",
        vscode.ViewColumn.One,
        { enableScripts: false }
      );

      panel.webview.html = renderDashboardHtml(responses);
    }
  );

  context.subscriptions.push(output, startDisposable, dashboardDisposable);
}

export function deactivate() {
  // no-op
}

type AnyJson = any;

function spawnAdapter(workspaceFsPath: string, output: vscode.OutputChannel) {
  // Prefer running via `cargo run` so the demo works without a prebuilt binary.
  // If a local binary exists, we use it.
  const binPath = path.join(workspaceFsPath, "target", "debug", "stylus-dap");
  if (fs.existsSync(binPath)) {
    output.appendLine(`Using adapter binary: ${binPath}`);
    return spawn(binPath, [], { cwd: workspaceFsPath });
  }

  output.appendLine("Using adapter via cargo run (first run may take a moment)...");
  return spawn(
    "cargo",
    ["run", "-p", "debug-adapter", "--bin", "stylus-dap"],
    { cwd: workspaceFsPath }
  );
}

async function runDemoProtocol(proc: ReturnType<typeof spawn>, output: vscode.OutputChannel) {
  let seq = 1;
  const results: Record<string, AnyJson> = {};

  const send = (command: string, args: AnyJson = {}) => {
    const msg = {
      seq: seq++,
      type: "request",
      command,
      arguments: args,
    };
    output.appendLine(`→ ${command}`);
    proc.stdin?.write(JSON.stringify(msg) + "\n");
  };

  const parseLine = (line: string) => {
    try {
      return JSON.parse(line);
    } catch {
      return null;
    }
  };

  let buffer = "";
      proc.stdout?.on("data", (data: any) => {
    buffer += data.toString("utf8");
    let idx: number;
    while ((idx = buffer.indexOf("\n")) >= 0) {
      const line = buffer.slice(0, idx).trim();
      buffer = buffer.slice(idx + 1);
      if (!line) continue;
      const msg = parseLine(line);
      if (!msg) {
        output.appendLine(`(non-json) ${line}`);
        continue;
      }
      if (msg.type === "response" && typeof msg.command === "string") {
        results[msg.command] = msg.body;
        output.appendLine(`← ${msg.command} (success=${msg.success})`);
      } else {
        output.appendLine(`← ${line}`);
      }
    }
  });

  proc.stderr?.on("data", (data: any) => {
    output.appendLine(`[adapter stderr] ${data.toString("utf8")}`);
  });

  // Protocol sequence for the demo: init → launch → next → storage → gas → disconnect
  send("initialize");
  send("launch", {
    contractPath: "examples/demo-contracts/vault.rs",
    entrypoint: "deposit_and_withdraw",
  });
  send("next");
  send("stylusStorage");
  send("stylusGas");
  send("disconnect");

  // Wait briefly for adapter to respond and exit.
  await new Promise<void>((resolve) => {
    const t = setTimeout(() => resolve(), 2500);
    proc.on("close", () => {
      clearTimeout(t);
      resolve();
    });
  });

  return results;
}

function escapeHtml(s: string) {
  return s.replace(/[&<>"']/g, (c) => {
    switch (c) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case '"':
        return "&quot;";
      case "'":
        return "&#39;";
      default:
        return c;
    }
  });
}

function renderDashboardHtml(responses: Record<string, AnyJson>) {
  const events = responses["next"]?.events ?? [];
  const storageSlots = responses["stylusStorage"]?.slots ?? [];
  const gas = responses["stylusGas"] ?? null;

  const traceRows = Array.isArray(events)
    ? events
        .slice(0, 10)
        .map((ev: any) => {
          const step = ev.step ?? "";
          const op = ev.opcode ?? "";
          const stackDepth = Array.isArray(ev.stack) ? ev.stack.length : "";
          const writes = Array.isArray(ev.storage_diff) ? ev.storage_diff.length : "";
          return `<tr><td>${escapeHtml(String(step))}</td><td>${escapeHtml(String(op))}</td><td>${escapeHtml(String(stackDepth))}</td><td>${escapeHtml(String(writes))}</td></tr>`;
        })
        .join("")
    : "";

  const storageRows = Array.isArray(storageSlots)
    ? storageSlots
        .map((s: any) => {
          const key = s.key ?? "";
          const value = s.value ?? "null";
          return `<tr><td>${escapeHtml(String(key))}</td><td><code>${escapeHtml(String(value))}</code></td></tr>`;
        })
        .join("")
    : "";

  const gasBlock = gas
    ? `<div class="card">
        <h2>Gas Report (Wasmtime Fuel)</h2>
        <div class="kv"><span>Function</span><span>${escapeHtml(String(gas.function ?? ""))}</span></div>
        <div class="kv"><span>Total gas</span><span>${escapeHtml(String(gas.total_gas ?? ""))}</span></div>
        <h3>Hotspots</h3>
        <pre>${escapeHtml(JSON.stringify(gas.hotspots ?? [], null, 2))}</pre>
        <h3>Suggestions</h3>
        <pre>${escapeHtml(JSON.stringify(gas.suggestions ?? [], null, 2))}</pre>
      </div>`
    : `<div class="card"><h2>Gas Report</h2><p>${responses["initialize"] ? "Pending — VM not connected" : "No data."}</p></div>`;

  return `<!doctype html>
  <html>
    <head>
      <meta charset="utf-8" />
      <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline';" />
      <style>
        body { font-family: -apple-system, BlinkMacSystemFont, Segoe WPC, Segoe UI, sans-serif; padding: 16px; color: #ddd; background: #111; }
        .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
        .card { background: #1a1a1a; border: 1px solid #2a2a2a; border-radius: 10px; padding: 12px; }
        h1 { font-size: 18px; margin: 0 0 10px; }
        h2 { font-size: 14px; margin: 0 0 10px; }
        h3 { font-size: 12px; margin: 12px 0 6px; color: #bbb; }
        table { width: 100%; border-collapse: collapse; }
        th, td { text-align: left; padding: 6px 8px; border-bottom: 1px solid #2a2a2a; font-size: 12px; }
        th { color: #bbb; font-weight: 600; }
        code, pre { background: #121212; border: 1px solid #2a2a2a; border-radius: 8px; padding: 8px; overflow: auto; }
        pre { margin: 0; font-size: 12px; }
        .kv { display: flex; justify-content: space-between; font-size: 12px; padding: 4px 0; border-bottom: 1px solid #2a2a2a; }
        .kv span:first-child { color: #bbb; }
        .muted { color: #aaa; font-size: 12px; }
      </style>
    </head>
    <body>
      <h1>Stylus Debug Suite Dashboard</h1>
      <p class="muted">Data source: local debug adapter (stdio). This is a demo UI scaffold.</p>

      <div class="grid">
        <div class="card">
          <h2>Trace Preview</h2>
          <table>
            <thead>
              <tr><th>Step</th><th>Opcode</th><th>Stack</th><th>Writes</th></tr>
            </thead>
            <tbody>
              ${traceRows || "<tr><td colspan='4'>No trace data.</td></tr>"}
            </tbody>
          </table>
        </div>

        <div class="card">
          <h2>Storage Viewer</h2>
          <table>
            <thead>
              <tr><th>Key</th><th>Value</th></tr>
            </thead>
            <tbody>
              ${storageRows || "<tr><td colspan='2'>No storage data.</td></tr>"}
            </tbody>
          </table>
        </div>
      </div>

      <div style="height: 12px"></div>
      ${gasBlock}

      <div style="height: 12px"></div>
      <div class="card">
        <h2>Raw Responses</h2>
        <pre>${escapeHtml(JSON.stringify(responses, null, 2))}</pre>
      </div>
    </body>
  </html>`;
}



