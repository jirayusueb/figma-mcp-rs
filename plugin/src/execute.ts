// Execute arbitrary code in the plugin sandbox.

export const handleExecuteRequest = async (request: any) => {
  if (request.type !== "execute_code") return null;

  const p = request.params || {};
  if (typeof p.code !== "string" || !p.code.trim()) {
    throw new Error("code is required");
  }

  // Clamp timeout between 1 and 25000 ms.
  const timeoutMs = Math.min(Math.max(Number(p.timeoutMs) || 5000, 1), 25000);

  const logs: string[] = [];

  // Capture console output.
  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;

  const captureLog = (level: string, ...args: any[]) => {
    const msg = args.map(a => String(a)).join(" ");
    logs.push(`${level}: ${msg}`);
    if (logs.length > 200) {
      logs.shift(); // Keep only the last 200 lines.
    }
  };

  console.log = (...args: any[]) => captureLog("log", ...args);
  console.warn = (...args: any[]) => captureLog("warn", ...args);
  console.error = (...args: any[]) => captureLog("error", ...args);

  try {
    // Wrap the code as an async function and execute it.
    const wrapped = `(async function() {\n${p.code}\n})()`;
    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error(`Execution timed out after ${timeoutMs}ms`)), timeoutMs)
    );

    // Use eval in an IIFE to get a fresh scope, then race with timeout.
    const value = await Promise.race([
      (0, eval)(wrapped),
      timeoutPromise,
    ]);

    figma.commitUndo();

    return {
      type: request.type,
      requestId: request.requestId,
      data: {
        result: serializeResult(value),
        logs,
      },
    };
  } finally {
    // Restore console.
    console.log = originalLog;
    console.warn = originalWarn;
    console.error = originalError;
  }
};

function serializeResult(value: any): any {
  if (value === undefined || value === null) {
    return null;
  }

  // Handle arrays: map through serializeResult recursively, cap at 500 entries.
  if (Array.isArray(value)) {
    const arr = value.slice(0, 500).map(serializeResult);
    if (value.length > 500) {
      arr.push({ truncated: value.length - 500 });
    }
    return arr;
  }

  // Handle Figma nodes: extract id, name, type.
  if (
    value &&
    typeof value === "object" &&
    typeof value.id === "string" &&
    typeof value.type === "string" &&
    typeof value.name === "string"
  ) {
    return {
      id: value.id,
      name: value.name,
      type: value.type,
    };
  }

  // Try to serialize as JSON; fall back to string if cyclic.
  try {
    return JSON.parse(JSON.stringify(value));
  } catch {
    return String(value);
  }
}
