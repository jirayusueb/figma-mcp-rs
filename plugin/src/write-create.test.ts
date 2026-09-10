import { describe, it, expect, beforeEach } from "bun:test";
import { handleWriteCreateRequest } from "./write-create";

// ── Figma global mock ─────────────────────────────────────────────────────────

let mockNodes: Record<string, any>;
let commitUndoCalled: boolean;
let createdComponents: any[];
let createdFrames: { sizingHorizontal: string; appended: boolean }[];

const makeRequest = (type: string, nodeIds?: string[], params?: any) => ({
  type,
  requestId: "req-test-1",
  nodeIds: nodeIds ?? [],
  params: params ?? {},
});

beforeEach(() => {
  commitUndoCalled = false;
  createdComponents = [];
  createdFrames = [];
  mockNodes = {};
  (globalThis as any).figma = {
    get currentPage() { return { id: "0:1", name: "Page 1", appendChild: () => {} }; },
    getNodeByIdAsync: async (id: string) => mockNodes[id] ?? null,
    createComponent: () => {
      const comp: any = {
        id: "comp:new",
        name: "Component",
        type: "COMPONENT",
        x: 0, y: 0, width: 100, height: 100,
        fills: [], strokes: [], cornerRadius: 0, layoutMode: "NONE",
        children: [] as any[],
        resize(w: number, h: number) { this.width = w; this.height = h; },
        appendChild(child: any) { this.children.push(child); },
      };
      createdComponents.push(comp);
      return comp;
    },
    createFrame: () => {
      const frame = {
        id: "frame:new", name: "Frame", type: "FRAME",
        x: 0, y: 0, width: 100, height: 100,
        fills: [] as unknown[],
        layoutMode: "NONE",
        appended: false,
        sizingHorizontal: "FIXED",
        resize(w: number, h: number) { this.width = w; this.height = h; },
        get layoutSizingHorizontal() { return this.sizingHorizontal; },
        set layoutSizingHorizontal(v: string) {
          // Mirrors Figma: FILL is rejected until the node has an auto-layout parent.
          if (!this.appended) throw new Error("FILL requires an auto-layout parent");
          this.sizingHorizontal = v;
        },
      };
      createdFrames.push(frame);
      return frame;
    },
    commitUndo: () => { commitUndoCalled = true; },
    mixed: Symbol("mixed"),
  };
});

// ── create_component ──────────────────────────────────────────────────────────

describe("create_component", () => {
  const makeParent = () => ({
    id: "0:1",
    children: [] as any[],
    insertChild(_: number, c: any) { this.children.push(c); },
  });

  it("converts a FRAME to a COMPONENT in place", async () => {
    const child = { id: "2:1", type: "RECTANGLE" };
    let frameRemoved = false;
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Card", type: "FRAME",
      x: 10, y: 20, width: 200, height: 100,
      fills: [{ type: "SOLID" }], strokes: [],
      cornerRadius: 8, layoutMode: "NONE",
      children: [child], parent,
      remove() { frameRemoved = true; },
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    const res = await handleWriteCreateRequest(makeRequest("create_component", ["1:1"]));
    expect(res?.data.type).toBe("COMPONENT");
    expect(createdComponents[0].name).toBe("Card");
    expect(createdComponents[0].cornerRadius).toBe(8);
    expect(createdComponents[0].children).toContain(child);
    expect(frameRemoved).toBe(true);
    expect(commitUndoCalled).toBe(true);
  });

  it("copies frame dimensions", async () => {
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Banner", type: "FRAME",
      x: 0, y: 0, width: 320, height: 64,
      fills: [], strokes: [], cornerRadius: 0, layoutMode: "NONE",
      children: [], parent,
      remove() {},
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    await handleWriteCreateRequest(makeRequest("create_component", ["1:1"]));
    expect(createdComponents[0].width).toBe(320);
    expect(createdComponents[0].height).toBe(64);
  });

  it("uses custom name when provided", async () => {
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Frame", type: "FRAME",
      x: 0, y: 0, width: 100, height: 100,
      fills: [], strokes: [], cornerRadius: 0, layoutMode: "NONE",
      children: [], parent,
      remove() {},
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    await handleWriteCreateRequest(makeRequest("create_component", ["1:1"], { name: "Button" }));
    expect(createdComponents[0].name).toBe("Button");
  });

  it("copies auto-layout properties when layoutMode is set", async () => {
    const parent = makeParent();
    const frame = {
      id: "1:1", name: "Row", type: "FRAME",
      x: 0, y: 0, width: 200, height: 48,
      fills: [], strokes: [], cornerRadius: 0,
      layoutMode: "HORIZONTAL",
      paddingTop: 8, paddingRight: 16, paddingBottom: 8, paddingLeft: 16,
      itemSpacing: 12,
      primaryAxisAlignItems: "CENTER",
      counterAxisAlignItems: "CENTER",
      primaryAxisSizingMode: "AUTO",
      counterAxisSizingMode: "FIXED",
      layoutWrap: "WRAP",
      counterAxisSpacing: 4,
      children: [], parent,
      remove() {},
    };
    parent.children = [frame];
    mockNodes["1:1"] = frame;

    await handleWriteCreateRequest(makeRequest("create_component", ["1:1"]));
    const comp = createdComponents[0];
    expect(comp.layoutMode).toBe("HORIZONTAL");
    expect(comp.paddingTop).toBe(8);
    expect(comp.paddingRight).toBe(16);
    expect(comp.itemSpacing).toBe(12);
    expect(comp.primaryAxisAlignItems).toBe("CENTER");
    expect(comp.primaryAxisSizingMode).toBe("AUTO");
    expect(comp.counterAxisSizingMode).toBe("FIXED");
    expect(comp.layoutWrap).toBe("WRAP");
    expect(comp.counterAxisSpacing).toBe(4);
  });

  it("throws when nodeId not found", async () => {
    await expect(
      handleWriteCreateRequest(makeRequest("create_component", ["9:9"]))
    ).rejects.toThrow("Node not found: 9:9");
  });

  it("throws when node is not a FRAME", async () => {
    mockNodes["1:1"] = { id: "1:1", type: "RECTANGLE" };
    await expect(
      handleWriteCreateRequest(makeRequest("create_component", ["1:1"]))
    ).rejects.toThrow("is not a FRAME");
  });

  it("throws when no nodeId provided", async () => {
    await expect(
      handleWriteCreateRequest(makeRequest("create_component", []))
    ).rejects.toThrow("nodeId is required");
  });
});

// ── create_frame ──────────────────────────────────────────────────────────────

describe("create_frame", () => {
  it("applies FILL sizing after the frame is appended to its parent", async () => {
    mockNodes["0:2"] = {
      id: "0:2",
      layoutMode: "VERTICAL",
      appendChild(child) { child.appended = true; },
    };

    const res = await handleWriteCreateRequest(makeRequest("create_frame", [], {
      parentId: "0:2",
      name: "Fill",
      height: 40,
      layoutSizingHorizontal: "FILL",
    }));
    expect(res?.data.name).toBe("Fill");
    expect(createdFrames[0].sizingHorizontal).toBe("FILL");
    expect(commitUndoCalled).toBe(true);
  });
});

// ── create_section ────────────────────────────────────────────────────────────

describe("create_section", () => {
  let createdSection: any;

  beforeEach(() => {
    createdSection = null;
    (globalThis as any).figma = {
      ...(globalThis as any).figma,
      currentPage: { id: "0:1", name: "Page 1", appendChild: () => {} },
      createSection: () => {
        createdSection = {
          id: "section:new", name: "Section", type: "SECTION",
          x: 0, y: 0, width: 200, height: 200,
          resizeWithoutConstraints(w: number, h: number) { this.width = w; this.height = h; },
        };
        return createdSection;
      },
    };
  });

  it("creates a section with a name", async () => {
    const res = await handleWriteCreateRequest(makeRequest("create_section", [], { name: "Sprint 1" }));
    expect(createdSection.name).toBe("Sprint 1");
    expect(res?.data.type).toBe("SECTION");
    expect(res?.data.id).toBe("section:new");
    expect(commitUndoCalled).toBe(true);
  });

  it("creates a section at a specific position", async () => {
    const res = await handleWriteCreateRequest(makeRequest("create_section", [], { x: 100, y: 200 }));
    expect(createdSection.x).toBe(100);
    expect(createdSection.y).toBe(200);
  });

  it("creates a section with custom size", async () => {
    await handleWriteCreateRequest(makeRequest("create_section", [], { width: 800, height: 600 }));
    expect(createdSection.width).toBe(800);
    expect(createdSection.height).toBe(600);
  });

  it("creates a section with default values when no params given", async () => {
    const res = await handleWriteCreateRequest(makeRequest("create_section", [], {}));
    expect(res?.data.id).toBe("section:new");
  });
});

// ── create_component FigJam guard ───────────────────────────────────────────

describe("create_component FigJam guard", () => {
  it("throws in FigJam", async () => {
    (globalThis as any).figma = { ...(globalThis as any).figma, editorType: "figjam" };
    mockNodes["1:1"] = {
      id: "1:1", name: "Card", type: "FRAME", x: 0, y: 0, width: 100, height: 100,
      fills: [], strokes: [], layoutMode: "NONE", children: [],
      parent: { children: [], indexOf: () => 0, insertChild: () => {} },
      remove: () => {},
    };
    await expect(
      handleWriteCreateRequest(makeRequest("create_component", ["1:1"]))
    ).rejects.toThrow("create_component is not available in FigJam");
  });
});

// ── create_sticky ────────────────────────────────────────────────────────────

describe("create_sticky", () => {
  let createdSticky: any;

  beforeEach(() => {
    createdSticky = null;
    (globalThis as any).figma = {
      ...(globalThis as any).figma,
      editorType: "figjam",
      loadFontAsync: async () => {},
      createSticky: () => {
        createdSticky = {
          id: "sticky:new", name: "Sticky", type: "STICKY",
          x: 0, y: 0, fills: [],
          text: { characters: "", fontName: { family: "Inter", style: "Regular" } },
        };
        return createdSticky;
      },
    };
  });

  it("creates a sticky with text", async () => {
    const res = await handleWriteCreateRequest(makeRequest("create_sticky", [], { text: "Hello" }));
    expect(createdSticky.text.characters).toBe("Hello");
    expect(res?.data.type).toBe("STICKY");
    expect(res?.data.id).toBe("sticky:new");
    expect(commitUndoCalled).toBe(true);
  });

  it("sets position and name when provided", async () => {
    await handleWriteCreateRequest(makeRequest("create_sticky", [], { text: "Hi", x: 50, y: 60, name: "Note" }));
    expect(createdSticky.x).toBe(50);
    expect(createdSticky.y).toBe(60);
    expect(createdSticky.name).toBe("Note");
  });

  it("throws when text is missing", async () => {
    await expect(
      handleWriteCreateRequest(makeRequest("create_sticky", [], {}))
    ).rejects.toThrow("text is required");
  });

  it("throws outside FigJam", async () => {
    (globalThis as any).figma = { ...(globalThis as any).figma, editorType: "figma" };
    await expect(
      handleWriteCreateRequest(makeRequest("create_sticky", [], { text: "Hello" }))
    ).rejects.toThrow("create_sticky is only available in FigJam");
  });
});

// ── create_connector ─────────────────────────────────────────────────────────

describe("create_connector", () => {
  let createdConnector: any;

  beforeEach(() => {
    createdConnector = null;
    (globalThis as any).figma = {
      ...(globalThis as any).figma,
      editorType: "figjam",
      loadFontAsync: async () => {},
      createConnector: () => {
        createdConnector = {
          id: "connector:new", name: "Connector", type: "CONNECTOR",
          text: { characters: "", fontName: { family: "Inter", style: "Regular" } },
          connectorStart: undefined,
          connectorEnd: undefined,
        };
        return createdConnector;
      },
    };
  });

  it("creates a connector between two nodes", async () => {
    const res = await handleWriteCreateRequest(
      makeRequest("create_connector", [], { startNodeId: "1:1", endNodeId: "1:2" })
    );
    expect(createdConnector.connectorStart).toEqual({ endpointNodeId: "1:1", magnet: "AUTO" });
    expect(createdConnector.connectorEnd).toEqual({ endpointNodeId: "1:2", magnet: "AUTO" });
    expect(res?.data.type).toBe("CONNECTOR");
    expect(commitUndoCalled).toBe(true);
  });

  it("sets text when provided", async () => {
    await handleWriteCreateRequest(makeRequest("create_connector", [], { text: "Label" }));
    expect(createdConnector.text.characters).toBe("Label");
  });

  it("throws outside FigJam", async () => {
    (globalThis as any).figma = { ...(globalThis as any).figma, editorType: "figma" };
    await expect(
      handleWriteCreateRequest(makeRequest("create_connector", [], {}))
    ).rejects.toThrow("create_connector is only available in FigJam");
  });
});
