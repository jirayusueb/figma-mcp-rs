// Escape hatch — runs arbitrary Plugin API JavaScript in the plugin sandbox.
// Mirrors the official server's `use_figma` tool: one tool instead of a
// per-node-type tool for every API the sandbox exposes.

type UseFigmaRequest = {
  type: string;
  requestId: string;
  params?: { code?: unknown };
};

export const handleUseFigmaRequest = async (request: UseFigmaRequest) => {
  if (request.type !== "use_figma") return null;

  const code = request.params?.code;
  if (typeof code !== "string" || code.trim() === "") {
    throw new Error("code is required");
  }

  // Wrapped in an async arrow so scripts can use top-level `await` and `return`.
  // Direct eval: the only dynamic-code path proven to work in the Figma sandbox.
  const compiled: unknown = eval(`(async () => {\n${code}\n})`);
  if (typeof compiled !== "function") throw new Error("code did not compile to a function");
  // eval erases types; the wrapper above is always a zero-arg async function.
  const script = compiled as () => Promise<unknown>;

  const result = await script();
  figma.commitUndo();

  // The bridge frame is JSON — node objects and functions cannot cross it.
  let data: unknown;
  try {
    data = JSON.parse(JSON.stringify(result === undefined ? null : result));
  } catch {
    throw new Error(
      "return value is not JSON-serializable — return plain data (node IDs, counts, names), not node objects",
    );
  }

  return { type: request.type, requestId: request.requestId, data: { result: data } };
};
