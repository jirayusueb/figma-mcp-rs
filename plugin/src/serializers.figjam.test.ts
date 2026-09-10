import { describe, it, expect } from "bun:test";
import { serializeNode } from "./serializers";

// ── FigJam node serialization ───────────────────────────────────────────────
// These branches return before touching `figma.getStyleByIdAsync`, so no
// global figma mock is required (unlike the base serializeNode suite).

describe("serializeNode — STICKY", () => {
  it("serializes id/name/type/bounds/text/fills", async () => {
    const node = {
      id: "1:1",
      name: "Note",
      type: "STICKY",
      x: 0, y: 0, width: 120, height: 120,
      text: { characters: "Hello" },
      fills: [{ type: "SOLID", color: { r: 1, g: 1, b: 0 }, opacity: 1 }],
    };
    const result = await serializeNode(node);
    expect(result).toEqual({
      id: "1:1",
      name: "Note",
      type: "STICKY",
      bounds: { x: 0, y: 0, width: 120, height: 120 },
      text: "Hello",
      fills: ["#ffff00"],
    });
  });

  it("reports mixed text as 'mixed'", async () => {
    const node = {
      id: "1:2", name: "Note", type: "STICKY", x: 0, y: 0, width: 10, height: 10,
      text: { characters: Symbol("mixed") },
      fills: [],
    };
    const result = await serializeNode(node);
    expect(result.text).toBe("mixed");
    expect(result.fills).toBeUndefined();
  });
});

describe("serializeNode — CONNECTOR", () => {
  it("serializes id/name/type/bounds/text/connectorStart/connectorEnd", async () => {
    const node = {
      id: "2:1",
      name: "Connector",
      type: "CONNECTOR",
      x: 0, y: 0, width: 300, height: 40,
      text: { characters: "yes" },
      connectorStart: { endpointNodeId: "1:1", magnet: "AUTO" },
      connectorEnd: { endpointNodeId: "1:2", magnet: "AUTO" },
    };
    const result = await serializeNode(node);
    expect(result).toEqual({
      id: "2:1",
      name: "Connector",
      type: "CONNECTOR",
      bounds: { x: 0, y: 0, width: 300, height: 40 },
      text: "yes",
      connectorStart: { endpointNodeId: "1:1", magnet: "AUTO" },
      connectorEnd: { endpointNodeId: "1:2", magnet: "AUTO" },
    });
  });
});

describe("serializeNode — minimal FigJam node types", () => {
  const cases: Array<[string]> = [["STAMP"], ["WIDGET"], ["EMBED"], ["LINK_UNFURL"], ["MEDIA"]];

  for (const [type] of cases) {
    it(`serializes ${type} as id/name/type/bounds only`, async () => {
      const node = { id: `3:${type}`, name: type, type, x: 1, y: 2, width: 3, height: 4 };
      const result = await serializeNode(node);
      expect(result).toEqual({
        id: `3:${type}`,
        name: type,
        type,
        bounds: { x: 1, y: 2, width: 3, height: 4 },
      });
    });
  }
});
