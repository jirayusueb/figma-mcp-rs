import { describe, it, expect, beforeEach } from "bun:test";
import { handleWriteStyleRequest } from "./write-styles";

// ── Figma global mock ─────────────────────────────────────────────────────────

let mockStyles: Record<string, any>;
let loadedFonts: { family: string; style: string }[];
let boundPaints: { field: string; variable: unknown }[];
let boundEffects: { field: string; variable: unknown }[];
let boundGrids: { field: string; variable: unknown }[];
let textBindings: [string, unknown][];
let commitUndoCalled: boolean;

const mockVariable = { id: "v1", name: "color/primary", resolvedType: "COLOR" };

const makeRequest = (type: string, params?: any) => ({
  type,
  requestId: "req-test-1",
  nodeIds: [],
  params: params ?? {},
});

beforeEach(() => {
  mockStyles = {};
  loadedFonts = [];
  boundPaints = [];
  boundEffects = [];
  boundGrids = [];
  textBindings = [];
  commitUndoCalled = false;

  mockStyles["paint:1"] = {
    id: "paint:1",
    type: "PAINT",
    name: "Brand/Primary",
    description: "",
    paints: [{ type: "SOLID", color: { r: 0, g: 0, b: 1 } }],
  };
  mockStyles["text:1"] = {
    id: "text:1",
    type: "TEXT",
    name: "Body/Regular",
    description: "",
    fontName: { family: "Inter", style: "Regular" },
    fontSize: 16,
    setBoundVariable: (field: string, variable: unknown) => textBindings.push([field, variable]),
  };
  mockStyles["effect:1"] = {
    id: "effect:1",
    type: "EFFECT",
    name: "Shadow/Card",
    description: "",
    effects: [{ type: "DROP_SHADOW", radius: 8 }],
  };
  mockStyles["grid:1"] = {
    id: "grid:1",
    type: "GRID",
    name: "Layout/Columns",
    description: "",
    layoutGrids: [
      { pattern: "COLUMNS", count: 12, gutterSize: 16, offset: 0, alignment: "STRETCH", visible: true },
    ],
  };

  (globalThis as any).figma = {
    editorType: "figma",
    commitUndo: () => {
      commitUndoCalled = true;
    },
    getStyleByIdAsync: async (id: string) => mockStyles[id] ?? null,
    loadFontAsync: async (font: { family: string; style: string }) => {
      loadedFonts.push(font);
    },
    variables: {
      getVariableByIdAsync: async (id: string) => (id === "v1" ? mockVariable : null),
      setBoundVariableForPaint: (paint: any, field: string, variable: unknown) => {
        boundPaints.push({ field, variable });
        return { ...paint, boundVariables: { color: variable } };
      },
      setBoundVariableForEffect: (effect: any, field: string, variable: unknown) => {
        boundEffects.push({ field, variable });
        return { ...effect, boundVariables: { [field]: variable } };
      },
      setBoundVariableForLayoutGrid: (grid: any, field: string, variable: unknown) => {
        boundGrids.push({ field, variable });
        return { ...grid, boundVariables: { [field]: variable } };
      },
    },
  };
});

// ── update_style ──────────────────────────────────────────────────────────────

describe("update_style", () => {
  it("updates text style size and line height", async () => {
    const res = await handleWriteStyleRequest(
      makeRequest("update_style", { styleId: "text:1", fontSize: 18, lineHeightValue: 24 }),
    );
    expect(mockStyles["text:1"].fontSize).toBe(18);
    expect(mockStyles["text:1"].lineHeight).toEqual({ value: 24, unit: "PIXELS" });
    expect(res?.data.type).toBe("TEXT");
    expect(commitUndoCalled).toBe(true);
  });

  it("loads the font before switching a text style family", async () => {
    await handleWriteStyleRequest(
      makeRequest("update_style", { styleId: "text:1", fontFamily: "Roboto" }),
    );
    expect(loadedFonts).toEqual([{ family: "Roboto", style: "Regular" }]);
    expect(mockStyles["text:1"].fontName).toEqual({ family: "Roboto", style: "Regular" });
  });

  it("updates a paint style from any color notation", async () => {
    await handleWriteStyleRequest(
      makeRequest("update_style", { styleId: "paint:1", color: "oklch(0.628 0.2577 29.23)" }),
    );
    const paint = mockStyles["paint:1"].paints[0];
    expect(paint.color.r).toBeCloseTo(1, 2);
    expect(paint.color.g).toBeCloseTo(0, 2);
    expect(paint.color.b).toBeCloseTo(0, 2);
  });

  it("replaces effect style effects", async () => {
    await handleWriteStyleRequest(
      makeRequest("update_style", {
        styleId: "effect:1",
        effects: [{ type: "LAYER_BLUR", radius: 12 }],
      }),
    );
    expect(mockStyles["effect:1"].effects).toEqual([
      { type: "LAYER_BLUR", radius: 12, visible: true },
    ]);
  });

  it("patches a grid style in place, keeping untouched fields", async () => {
    await handleWriteStyleRequest(
      makeRequest("update_style", { styleId: "grid:1", count: 6 }),
    );
    expect(mockStyles["grid:1"].layoutGrids[0]).toEqual({
      pattern: "COLUMNS",
      count: 6,
      gutterSize: 16,
      offset: 0,
      alignment: "STRETCH",
      visible: true,
    });
  });

  it("rebuilds the grid when the pattern changes", async () => {
    await handleWriteStyleRequest(
      makeRequest("update_style", { styleId: "grid:1", pattern: "GRID", sectionSize: 4 }),
    );
    const grid = mockStyles["grid:1"].layoutGrids[0];
    expect(grid.pattern).toBe("GRID");
    expect(grid.sectionSize).toBe(4);
    expect(grid.count).toBeUndefined();
  });

  it("rejects a field that does not apply to the style type", async () => {
    await expect(
      handleWriteStyleRequest(makeRequest("update_style", { styleId: "paint:1", fontSize: 18 })),
    ).rejects.toThrow("fontSize is not applicable to a PAINT style");
  });

  it("leaves the style untouched when only styleId is given", async () => {
    await handleWriteStyleRequest(makeRequest("update_style", { styleId: "text:1" }));
    expect(mockStyles["text:1"].fontSize).toBe(16);
    expect(mockStyles["text:1"].name).toBe("Body/Regular");
    expect(loadedFonts).toEqual([]);
  });

  it("throws when the style does not exist", async () => {
    await expect(
      handleWriteStyleRequest(makeRequest("update_style", { styleId: "nope", name: "x" })),
    ).rejects.toThrow("Style not found: nope");
  });
});

// ── bind_variable_to_style ────────────────────────────────────────────────────

describe("bind_variable_to_style", () => {
  it("binds a variable to a paint style without dropping other paints", async () => {
    mockStyles["paint:1"].paints = [
      { type: "SOLID", color: { r: 0, g: 0, b: 1 } },
      { type: "SOLID", color: { r: 1, g: 1, b: 1 } },
    ];
    const res = await handleWriteStyleRequest(
      makeRequest("bind_variable_to_style", { styleId: "paint:1", field: "color", variableId: "v1" }),
    );
    expect(boundPaints).toEqual([{ field: "color", variable: mockVariable }]);
    expect(mockStyles["paint:1"].paints).toHaveLength(2);
    expect(mockStyles["paint:1"].paints[1].color).toEqual({ r: 1, g: 1, b: 1 });
    expect(res?.data.bound).toBe(true);
  });

  it("rejects a field the style type cannot bind", async () => {
    await expect(
      handleWriteStyleRequest(
        makeRequest("bind_variable_to_style", { styleId: "paint:1", field: "fontSize", variableId: "v1" }),
      ),
    ).rejects.toThrow("is not bindable on a PAINT style");
  });

  it("binds a variable to a text style field", async () => {
    await handleWriteStyleRequest(
      makeRequest("bind_variable_to_style", { styleId: "text:1", field: "fontSize", variableId: "v1" }),
    );
    expect(textBindings).toEqual([["fontSize", mockVariable]]);
  });

  it("unbinds when variableId is omitted", async () => {
    const res = await handleWriteStyleRequest(
      makeRequest("bind_variable_to_style", { styleId: "text:1", field: "fontSize" }),
    );
    expect(textBindings).toEqual([["fontSize", null]]);
    expect(res?.data.bound).toBe(false);
  });

  it("binds a variable to an effect style field", async () => {
    await handleWriteStyleRequest(
      makeRequest("bind_variable_to_style", { styleId: "effect:1", field: "radius", variableId: "v1" }),
    );
    expect(boundEffects).toEqual([{ field: "radius", variable: mockVariable }]);
  });

  it("throws when an effect style has nothing to bind", async () => {
    mockStyles["effect:1"].effects = [];
    await expect(
      handleWriteStyleRequest(
        makeRequest("bind_variable_to_style", { styleId: "effect:1", field: "radius", variableId: "v1" }),
      ),
    ).rejects.toThrow("has no effects to bind");
  });

  it("binds a variable to a grid style field", async () => {
    await handleWriteStyleRequest(
      makeRequest("bind_variable_to_style", { styleId: "grid:1", field: "count", variableId: "v1" }),
    );
    expect(boundGrids).toEqual([{ field: "count", variable: mockVariable }]);
  });

  it("throws when the variable does not exist", async () => {
    await expect(
      handleWriteStyleRequest(
        makeRequest("bind_variable_to_style", { styleId: "text:1", field: "fontSize", variableId: "gone" }),
      ),
    ).rejects.toThrow("Variable not found: gone");
  });
});
