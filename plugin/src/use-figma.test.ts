import { describe, it, expect, beforeEach } from "bun:test";
import { handleUseFigmaRequest } from "./use-figma";

let commitUndoCalled: boolean;
let created: string[];
const sandbox = globalThis as unknown as { figma: unknown };

const makeRequest = (code?: unknown) => ({
  type: "use_figma",
  requestId: "req-test-1",
  params: code === undefined ? {} : { code },
});

beforeEach(() => {
  commitUndoCalled = false;
  created = [];
  sandbox.figma = {
    commitUndo: () => {
      commitUndoCalled = true;
    },
    createFrame: () => {
      const id = `1:${created.length + 1}`;
      created.push(id);
      return { id, name: "Frame" };
    },
    loadFontAsync: async () => {},
  };
});

describe("use_figma", () => {
  it("returns the script's return value", async () => {
    const res = await handleUseFigmaRequest(makeRequest("return 1 + 1"));
    expect(res?.data.result).toBe(2);
    expect(commitUndoCalled).toBe(true);
  });

  it("runs top-level await and reaches the figma global", async () => {
    const res = await handleUseFigmaRequest(
      makeRequest(`
        await figma.loadFontAsync({ family: "Inter", style: "Regular" });
        const frame = figma.createFrame();
        return { createdNodeIds: [frame.id] };
      `),
    );
    expect(res?.data.result).toEqual({ createdNodeIds: ["1:1"] });
    expect(created).toEqual(["1:1"]);
  });

  it("returns null for an undefined return so the frame stays valid JSON", async () => {
    const res = await handleUseFigmaRequest(makeRequest("figma.createFrame();"));
    expect(res?.data.result).toBeNull();
  });

  it("propagates script errors", async () => {
    expect(handleUseFigmaRequest(makeRequest("throw new Error('boom')"))).rejects.toThrow("boom");
  });

  it("rejects missing or blank code", async () => {
    expect(handleUseFigmaRequest(makeRequest())).rejects.toThrow("code is required");
    expect(handleUseFigmaRequest(makeRequest("  "))).rejects.toThrow("code is required");
  });

  it("rejects a return value that cannot cross the JSON bridge", async () => {
    expect(
      handleUseFigmaRequest(makeRequest("const a = {}; a.self = a; return a;")),
    ).rejects.toThrow("not JSON-serializable");
  });

  it("ignores requests for other tools", async () => {
    const res = await handleUseFigmaRequest({ type: "get_metadata", requestId: "r" });
    expect(res).toBeNull();
  });
});
