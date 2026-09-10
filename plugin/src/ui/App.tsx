import { createSignal, onMount, onCleanup, Show } from "solid-js";
import "./app.css";
import { Button } from "@/components/ui/button";
import { TextField, TextFieldInput } from "@/components/ui/text-field";

const RECONNECT_DELAY_MS = 1500;

export default function App() {
  const [connected, setConnected] = createSignal(false);
  const [fileName, setFileName] = createSignal("—");
  const [pageName, setPageName] = createSignal("—");
  const [selectionCount, setSelectionCount] = createSignal(0);
  const [activeCount, setActiveCount] = createSignal(0);
  const [lastTool, setLastTool] = createSignal("");

  // Configurable server address.
  // Persisted via figma.clientStorage (through plugin core) because localStorage
  // is unavailable inside Figma's data: URL sandbox.
  const [serverHost, setServerHost] = createSignal("127.0.0.1");
  const [serverPort, setServerPort] = createSignal("1998");

  const [showSettings, setShowSettings] = createSignal(false);
  const [editHost, setEditHost] = createSignal(serverHost());
  const [editPort, setEditPort] = createSignal(serverPort());

  let socket: WebSocket | null = null;
  let reconnectTimer: number | undefined;
  let configLoaded = false;

  function connect() {
    // Detach the old handler before closing so its onclose doesn't fire
    // after we've already assigned a new socket, which would null out the
    // new reference and silently break the connection.
    if (socket) {
      socket.onclose = null;
      socket.onerror = null;
      socket.close();
    }
    const ws = new WebSocket(`ws://${serverHost()}:${serverPort()}/ws`);
    socket = ws;

    ws.onopen = () => {
      setConnected(true);
      parent.postMessage({ pluginMessage: { type: "ui-ready" } }, "*");
    };

    // Both close and error must arm the retry: a refused connection in Figma's
    // sandbox can fire onerror alone, and scheduling only from onclose leaves
    // the panel dead until the plugin is re-run.
    const scheduleReconnect = () => {
      if (socket !== ws) return; // stale handler — a newer connect() took over
      setConnected(false);
      socket = null;
      setActiveCount(0);
      if (reconnectTimer !== undefined) return;
      reconnectTimer = window.setTimeout(() => {
        reconnectTimer = undefined;
        connect();
      }, RECONNECT_DELAY_MS);
    };

    ws.onclose = scheduleReconnect;
    ws.onerror = scheduleReconnect;

    ws.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data);
        if (payload.requestId) {
          setActiveCount((c) => c + 1);
          if (payload.type) setLastTool(payload.type);
        }
        parent.postMessage({ pluginMessage: { type: "server-request", payload } }, "*");
      } catch {
        // ignore malformed frames
      }
    };
  }

  function handleMessage(event: MessageEvent) {
    const msg = event.data?.pluginMessage;
    if (!msg) return;

    if (msg.type === "ws_config") {
      setServerHost(msg.host ?? "127.0.0.1");
      setServerPort(msg.port ?? "1998");
      if (!configLoaded) {
        configLoaded = true;
        connect();
      }
      return;
    }

    if (msg.type === "plugin-status") {
      setFileName(msg.payload.fileName);
      setPageName(msg.payload.pageName ?? "—");
      setSelectionCount(msg.payload.selectionCount);
      return;
    }

    if ("requestId" in msg) {
      if (msg.type !== "progress_update") {
        setActiveCount((c) => Math.max(0, c - 1));
      }
      if (socket?.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(msg));
      }
    }
  }

  function openSettings() {
    setEditHost(serverHost());
    setEditPort(serverPort());
    setShowSettings(true);
  }

  function applySettings() {
    setServerHost(editHost().trim() || "127.0.0.1");
    const p = parseInt(editPort(), 10);
    setServerPort(p > 0 && p <= 65535 ? String(p) : "1998");
    // Persist via plugin core (figma.clientStorage), since localStorage is
    // unavailable in Figma's data: URL environment.
    parent.postMessage(
      { pluginMessage: { type: "save_ws_config", host: serverHost(), port: serverPort() } },
      "*"
    );
    setShowSettings(false);
    // Cancel any pending reconnect and reconnect immediately with the new address.
    clearTimeout(reconnectTimer);
    reconnectTimer = undefined;
    connect();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") applySettings();
    if (event.key === "Escape") setShowSettings(false);
  }

  onMount(() => {
    window.addEventListener("message", handleMessage);

    // Request stored config from plugin core (responds with ws_config message).
    // connect() is called once we receive the response.
    parent.postMessage({ pluginMessage: { type: "get_ws_config" } }, "*");

    // Fallback: if the plugin core doesn't respond within 500 ms (e.g. during
    // dev / hot-reload without a running core), connect with defaults.
    const fallback = window.setTimeout(() => {
      if (!configLoaded) {
        configLoaded = true;
        connect();
      }
    }, 500);

    onCleanup(() => {
      clearTimeout(fallback);
      window.removeEventListener("message", handleMessage);
      clearTimeout(reconnectTimer);
      if (socket) socket.close();
    });
  });

  return (
    <div class="flex h-full flex-col overflow-hidden text-xs">
      {/* State band — the whole panel's headline */}
      <div class="flex flex-none items-center gap-2 border-b border-rule-strong bg-card px-3 py-2">
        <span
          class="size-2 flex-none"
          classList={{ "bg-positive": connected(), "bg-destructive": !connected() }}
        ></span>
        <span class="text-[1.25rem] font-semibold leading-none tracking-[-0.02em]">
          {connected() ? "Connected" : "Disconnected"}
        </span>
      </div>

      {/* Context rows */}
      <div class="flex flex-none flex-col divide-y divide-border/60">
        <div class="flex items-center justify-between gap-2 px-3 py-1">
          <span class="font-mono uppercase tracking-[0.1em] text-muted-foreground">File</span>
          <span class="min-w-0 truncate font-mono" title={fileName()}>
            {fileName()}
          </span>
        </div>
        <div class="flex items-center justify-between gap-2 px-3 py-1">
          <span class="font-mono uppercase tracking-[0.1em] text-muted-foreground">Page</span>
          <span class="min-w-0 truncate font-mono" title={pageName()}>
            {pageName()}
          </span>
        </div>
        <div class="flex items-center justify-between gap-2 px-3 py-1">
          <span class="font-mono uppercase tracking-[0.1em] text-muted-foreground">Sel</span>
          <span class="font-mono tabular-nums">{selectionCount()}</span>
        </div>
      </div>

      {/* Activity — the running tool names itself */}
      <Show when={activeCount() > 0}>
        <div class="flex flex-none items-center gap-2 border-y border-border/60 border-l-2 border-l-primary bg-secondary py-1 pl-2 pr-3 font-mono text-primary">
          <span class="truncate">→ {lastTool() || "…"}</span>
          <span class="ml-auto flex-none tabular-nums">×{activeCount()}</span>
        </div>
      </Show>

      {/* Transport — inline re-point */}
      <div class="mt-auto flex flex-none items-center border-t border-border">
        <Show
          when={showSettings()}
          fallback={
            <button
              class="flex h-6 w-full items-center justify-between gap-2 px-3 font-mono text-muted-foreground transition-colors hover:text-primary focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-ring"
              onClick={openSettings}
              title="Configure server address"
            >
              <span class="truncate">{serverHost()}:{serverPort()}</span>
              <span aria-hidden="true">▸</span>
            </button>
          }
        >
          <div class="flex w-full items-center gap-1 px-1.5 py-1">
            <TextField class="min-w-0 flex-1 gap-0">
              <TextFieldInput
                class="h-5 w-full min-w-0 px-1.5 font-mono text-xs"
                value={editHost()}
                onInput={(e) => setEditHost(e.currentTarget.value)}
                placeholder="127.0.0.1"
                onKeyDown={handleKeydown}
              />
            </TextField>
            <TextField class="w-12 gap-0">
              <TextFieldInput
                class="h-5 w-full min-w-0 px-1.5 font-mono text-xs"
                value={editPort()}
                onInput={(e) => setEditPort(e.currentTarget.value)}
                placeholder="1998"
                onKeyDown={handleKeydown}
              />
            </TextField>
            <Button
              variant="ghost"
              size="icon-sm"
              class="size-5 shrink-0 font-mono text-positive hover:bg-accent hover:text-accent-foreground"
              onClick={applySettings}
              title="Apply"
            >
              ✓
            </Button>
            <Button
              variant="ghost"
              size="icon-sm"
              class="size-5 shrink-0 font-mono text-destructive hover:bg-accent hover:text-accent-foreground"
              onClick={() => setShowSettings(false)}
              title="Cancel"
            >
              ✕
            </Button>
          </div>
        </Show>
      </div>
    </div>
  );
}
