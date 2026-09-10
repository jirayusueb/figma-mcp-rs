import { describe, it, expect, beforeEach } from "bun:test";
import { handleWriteVariableRequest } from "./write-variables";

// ── Figma global mock ─────────────────────────────────────────────────────────

type ModeCall = [string, string];

let setValues: [string, any][];
let codeSyntaxCalls: ModeCall[];
let renamedModes: ModeCall[];
let removedModes: string[];
let collectionRemoved: boolean;
let variableRemoved: boolean;
let modeCalls: { target: string; collection: unknown; modeId?: string; cleared: boolean }[];
let mockVariable: any;
let mockCollection: any;
let mockNodes: Record<string, any>;

const makeRequest = (type: string, params?: any, nodeIds?: string[]) => ({
  type,
  requestId: "req-test-1",
  nodeIds: nodeIds ?? [],
  params: params ?? {},
});

beforeEach(() => {
  setValues = [];
  codeSyntaxCalls = [];
  renamedModes = [];
  removedModes = [];
  collectionRemoved = false;
  variableRemoved = false;
  modeCalls = [];
  mockNodes = {};

  mockVariable = {
    id: "v1",
    name: "color/primary",
    description: "",
    resolvedType: "COLOR",
    scopes: ["ALL_SCOPES"],
    codeSyntax: {},
    hiddenFromPublishing: false,
    setValueForMode: (modeId: string, value: any) => setValues.push([modeId, value]),
    setVariableCodeSyntax: (platform: string, value: string) => {
      codeSyntaxCalls.push([platform, value]);
      mockVariable.codeSyntax[platform] = value;
    },
    remove: () => {
      variableRemoved = true;
    },
  };

  mockCollection = {
    id: "c1",
    name: "Semantic",
    modes: [
      { modeId: "m1", name: "Light" },
      { modeId: "m2", name: "Dark" },
    ],
    hiddenFromPublishing: false,
    renameMode: (modeId: string, name: string) => renamedModes.push([modeId, name]),
    removeMode: (modeId: string) => removedModes.push(modeId),
    remove: () => {
      collectionRemoved = true;
    },
  };

  const modeSetters = (id: string) => ({
    setExplicitVariableModeForCollection: (collection: unknown, modeId: string) =>
      modeCalls.push({ target: id, collection, modeId, cleared: false }),
    clearExplicitVariableModeForCollection: (collection: unknown) =>
      modeCalls.push({ target: id, collection, cleared: true }),
  });

  (globalThis as any).figma = {
    editorType: "figma",
    commitUndo: () => {},
    getNodeByIdAsync: async (id: string) => mockNodes[id] ?? null,
    currentPage: { id: "0:1", name: "Page 1", ...modeSetters("0:1") },
    variables: {
      getVariableByIdAsync: async (id: string) => (id === "v1" ? mockVariable : null),
      getVariableCollectionByIdAsync: async (id: string) => (id === "c1" ? mockCollection : null),
      createVariable: (name: string) => ({ ...mockVariable, name }),
      createVariableAliasByIdAsync: async (id: string) => ({ type: "VARIABLE_ALIAS", id }),
    },
  };
  mockNodes["1:1"] = { id: "1:1", name: "Frame", ...modeSetters("1:1") };
});

// ── create_variable / set_variable_value ─────────────────────────────────────

describe("create_variable", () => {
  it("parses a non-hex color notation into the first mode value", async () => {
    await handleWriteVariableRequest(
      makeRequest("create_variable", {
        name: "color/brand",
        collectionId: "c1",
        type: "COLOR",
        value: "hsl(210 90% 55%)",
      }),
    );
    const [modeId, value] = setValues[0];
    expect(modeId).toBe("m1");
    expect(value.r).toBeCloseTo(0.145, 3);
    expect(value.g).toBeCloseTo(0.55, 3);
    expect(value.b).toBeCloseTo(0.955, 3);
  });

  it("seeds an alias when aliasVariableId is given instead of a value", async () => {
    await handleWriteVariableRequest(
      makeRequest("create_variable", {
        name: "color/semantic",
        collectionId: "c1",
        type: "COLOR",
        aliasVariableId: "v2",
      }),
    );
    expect(setValues[0][1]).toEqual({ type: "VARIABLE_ALIAS", id: "v2" });
  });
});

describe("set_variable_value", () => {
  it("writes an alias for the given mode", async () => {
    const res = await handleWriteVariableRequest(
      makeRequest("set_variable_value", { variableId: "v1", modeId: "m1", aliasVariableId: "v2" }),
    );
    expect(setValues).toEqual([["m1", { type: "VARIABLE_ALIAS", id: "v2" }]]);
    expect(res?.data.alias).toBe("v2");
  });

  it("requires either a value or an alias", async () => {
    await expect(
      handleWriteVariableRequest(makeRequest("set_variable_value", { variableId: "v1", modeId: "m1" })),
    ).rejects.toThrow("value or aliasVariableId is required");
  });
});

// ── update_variable ──────────────────────────────────────────────────────────

describe("update_variable", () => {
  it("updates name, scopes, and code syntax on a variable", async () => {
    const res = await handleWriteVariableRequest(
      makeRequest("update_variable", {
        variableId: "v1",
        name: "color/brand",
        description: "brand base",
        scopes: ["ALL_FILLS"],
        codeSyntaxWeb: "--color-brand",
      }),
    );
    expect(mockVariable.name).toBe("color/brand");
    expect(mockVariable.description).toBe("brand base");
    expect(codeSyntaxCalls).toEqual([["WEB", "--color-brand"]]);
    expect(res?.data.scopes).toEqual(["ALL_FILLS"]);
  });

  it("renames a mode on a collection", async () => {
    const res = await handleWriteVariableRequest(
      makeRequest("update_variable", { collectionId: "c1", modeId: "m2", modeName: "Night" }),
    );
    expect(renamedModes).toEqual([["m2", "Night"]]);
    expect(res?.data.modes).toHaveLength(2);
  });

  it("requires a variableId or a collectionId", async () => {
    await expect(
      handleWriteVariableRequest(makeRequest("update_variable", { name: "nope" })),
    ).rejects.toThrow("variableId or collectionId is required");
  });
});

// ── delete_variable ──────────────────────────────────────────────────────────

describe("delete_variable", () => {
  it("removes a single mode without deleting the collection", async () => {
    const res = await handleWriteVariableRequest(
      makeRequest("delete_variable", { collectionId: "c1", modeId: "m2" }),
    );
    expect(removedModes).toEqual(["m2"]);
    expect(collectionRemoved).toBe(false);
    expect(res?.data.deleted).toBe(true);
  });

  it("still deletes the whole collection when no modeId is given", async () => {
    await handleWriteVariableRequest(makeRequest("delete_variable", { collectionId: "c1" }));
    expect(collectionRemoved).toBe(true);
    expect(removedModes).toEqual([]);
  });

  it("still deletes a variable", async () => {
    await handleWriteVariableRequest(makeRequest("delete_variable", { variableId: "v1" }));
    expect(variableRemoved).toBe(true);
  });
});

// ── set_variable_mode ────────────────────────────────────────────────────────

describe("set_variable_mode", () => {
  it("applies a mode to the current page when no node is given", async () => {
    const res = await handleWriteVariableRequest(
      makeRequest("set_variable_mode", { collectionId: "c1", modeId: "m2" }),
    );
    expect(modeCalls).toEqual([
      { target: "0:1", collection: mockCollection, modeId: "m2", cleared: false },
    ]);
    expect(res?.data.cleared).toBe(false);
  });

  it("applies a mode to a node when nodeIds is given", async () => {
    await handleWriteVariableRequest(
      makeRequest("set_variable_mode", { collectionId: "c1", modeId: "m2" }, ["1:1"]),
    );
    expect(modeCalls[0].target).toBe("1:1");
  });

  it("clears the override when modeId is omitted", async () => {
    const res = await handleWriteVariableRequest(
      makeRequest("set_variable_mode", { collectionId: "c1" }, ["1:1"]),
    );
    expect(modeCalls[0].cleared).toBe(true);
    expect(res?.data.cleared).toBe(true);
    expect(res?.data.modeId).toBeNull();
  });

  it("requires a collectionId", async () => {
    await expect(
      handleWriteVariableRequest(makeRequest("set_variable_mode", {})),
    ).rejects.toThrow("collectionId is required");
  });

  it("rejects a node that cannot carry variable modes", async () => {
    mockNodes["2:2"] = { id: "2:2", name: "Vector" };
    await expect(
      handleWriteVariableRequest(makeRequest("set_variable_mode", { collectionId: "c1" }, ["2:2"])),
    ).rejects.toThrow("does not support variable modes");
  });
});
