import { describe, it, expect, beforeEach } from "bun:test";
import {
  makeSolidPaint,
  applyAutoLayout,
  applyLayoutSizing,
  base64ToBytes,
  getParentNode,
  type AutoLayoutProps,
  type LayoutSizingProps,
} from "./write-helpers";

// ── Figma global mock ─────────────────────────────────────────────────────────

let mockCurrentPage: any;
let mockGetNodeByIdAsync: (id: string) => Promise<any>;

beforeEach(() => {
  mockCurrentPage = { id: "0:1", name: "Page 1" };
  mockGetNodeByIdAsync = async (_id: string) => null;
  (globalThis as any).figma = {
    get currentPage() { return mockCurrentPage; },
    getNodeByIdAsync: (id: string) => mockGetNodeByIdAsync(id),
  };
});

// ── makeSolidPaint ────────────────────────────────────────────────────────────

describe("makeSolidPaint", () => {
  it("creates a solid paint from a hex string", () => {
    const paint = makeSolidPaint("#ff0000");
    expect(paint.type).toBe("SOLID");
    expect((paint.color as any).r).toBeCloseTo(1);
    expect((paint as any).opacity).toBeUndefined();
  });

  it("accepts every color notation, not just hex", () => {
    const oklch = makeSolidPaint("oklch(0.628 0.2577 29.23)");
    expect((oklch.color as any).r).toBeCloseTo(1, 2);
    expect((oklch.color as any).g).toBeCloseTo(0, 2);
    const hsl = makeSolidPaint("hsl(120 100% 25%)");
    expect((hsl.color as any).g).toBeCloseTo(0.5, 2);
    const rgb = makeSolidPaint("rgb(59 130 246 / 50%)");
    expect((rgb as any).opacity).toBeCloseTo(0.5);
  });

  it("omits opacity when alpha is 1", () => {
    const paint = makeSolidPaint("#ffffff");
    expect((paint as any).opacity).toBeUndefined();
  });

  it("sets opacity when alpha < 1", () => {
    // #ff000080 → a ≈ 128/255
    const paint = makeSolidPaint("#ff000080");
    expect((paint as any).opacity).toBeCloseTo(128 / 255);
  });

  it("uses opacityOverride over alpha channel", () => {
    const paint = makeSolidPaint("#ff000080", 0.25);
    expect((paint as any).opacity).toBe(0.25);
  });

  it("creates paint from an object color input", () => {
    const paint = makeSolidPaint({ r: 0, g: 1, b: 0, a: 1 });
    expect(paint.type).toBe("SOLID");
    expect((paint.color as any).g).toBeCloseTo(1);
    expect((paint as any).opacity).toBeUndefined();
  });

  it("uses opacity from object input when a < 1", () => {
    const paint = makeSolidPaint({ r: 0, g: 0, b: 1, a: 0.5 });
    expect((paint as any).opacity).toBe(0.5);
  });

  it("defaults a to 1 when not provided in object input", () => {
    const paint = makeSolidPaint({ r: 0, g: 0, b: 1 });
    expect((paint as any).opacity).toBeUndefined();
  });
});

// ── applyAutoLayout ───────────────────────────────────────────────────────────

describe("applyAutoLayout", () => {
  const makeFrame = (): AutoLayoutProps => ({
    layoutMode: "NONE",
    paddingTop: 0,
    paddingRight: 0,
    paddingBottom: 0,
    paddingLeft: 0,
    itemSpacing: 0,
    primaryAxisAlignItems: "MIN",
    counterAxisAlignItems: "MIN",
    primaryAxisSizingMode: "FIXED",
    counterAxisSizingMode: "FIXED",
    layoutWrap: "NO_WRAP",
    counterAxisSpacing: null,
    counterAxisAlignContent: "AUTO",
    itemReverseZIndex: false,
    strokesIncludedInLayout: false,
  });

  it("sets layoutMode", () => {
    const frame = makeFrame();
    applyAutoLayout(frame, { layoutMode: "HORIZONTAL" });
    expect(frame.layoutMode).toBe("HORIZONTAL");
  });

  it("sets padding values", () => {
    const frame = makeFrame();
    applyAutoLayout(frame, { paddingTop: 8, paddingRight: 16, paddingBottom: 8, paddingLeft: 16 });
    expect(frame.paddingTop).toBe(8);
    expect(frame.paddingRight).toBe(16);
  });

  it("sets itemSpacing", () => {
    const frame = makeFrame();
    applyAutoLayout(frame, { itemSpacing: 12 });
    expect(frame.itemSpacing).toBe(12);
  });

  it("sets axis alignment when layoutMode is not NONE", () => {
    const frame = makeFrame();
    applyAutoLayout(frame, {
      layoutMode: "HORIZONTAL",
      primaryAxisAlignItems: "CENTER",
      counterAxisAlignItems: "MIN",
      primaryAxisSizingMode: "FIXED",
      counterAxisSizingMode: "AUTO",
      layoutWrap: "NO_WRAP",
    });
    expect(frame.primaryAxisAlignItems).toBe("CENTER");
    expect(frame.counterAxisAlignItems).toBe("MIN");
    expect(frame.primaryAxisSizingMode).toBe("FIXED");
    expect(frame.counterAxisSizingMode).toBe("AUTO");
    expect(frame.layoutWrap).toBe("NO_WRAP");
  });

  it("does not set axis props when layoutMode is NONE", () => {
    const frame = makeFrame();
    applyAutoLayout(frame, {
      primaryAxisAlignItems: "CENTER",
    });
    expect(frame.primaryAxisAlignItems).toBe("MIN");
  });

  it("sets counterAxisSpacing only when layoutWrap is WRAP", () => {
    const frame = makeFrame();
    applyAutoLayout(frame, {
      layoutMode: "HORIZONTAL",
      layoutWrap: "WRAP",
      counterAxisSpacing: 8,
    });
    expect(frame.counterAxisSpacing).toBe(8);
  });

  it("skips counterAxisSpacing when not WRAP", () => {
    const frame = makeFrame();
    applyAutoLayout(frame, {
      layoutMode: "HORIZONTAL",
      layoutWrap: "NO_WRAP",
      counterAxisSpacing: 8,
    });
    expect(frame.counterAxisSpacing).toBeNull();
  });

  it("sets counterAxisAlignContent only when layoutWrap is WRAP", () => {
    const wrapped = makeFrame();
    applyAutoLayout(wrapped, {
      layoutMode: "HORIZONTAL",
      layoutWrap: "WRAP",
      counterAxisAlignContent: "SPACE_BETWEEN",
    });
    expect(wrapped.counterAxisAlignContent).toBe("SPACE_BETWEEN");

    const unwrapped = makeFrame();
    applyAutoLayout(unwrapped, {
      layoutMode: "HORIZONTAL",
      layoutWrap: "NO_WRAP",
      counterAxisAlignContent: "SPACE_BETWEEN",
    });
    expect(unwrapped.counterAxisAlignContent).toBe("AUTO");
  });

  it("sets itemReverseZIndex and strokesIncludedInLayout, including false", () => {
    const frame = makeFrame();
    frame.itemReverseZIndex = true;
    applyAutoLayout(frame, {
      layoutMode: "VERTICAL",
      itemReverseZIndex: false,
      strokesIncludedInLayout: true,
    });
    expect(frame.itemReverseZIndex).toBe(false);
    expect(frame.strokesIncludedInLayout).toBe(true);
  });
});

// ── applyLayoutSizing ─────────────────────────────────────────────────────────

describe("applyLayoutSizing", () => {
  const makeChild = (): LayoutSizingProps => ({
    layoutPositioning: "AUTO",
    layoutSizingHorizontal: "FIXED",
    layoutSizingVertical: "FIXED",
  });

  it("sets sizing and positioning", () => {
    const child = makeChild();
    applyLayoutSizing(child, {
      layoutPositioning: "ABSOLUTE",
      layoutSizingHorizontal: "FILL",
      layoutSizingVertical: "HUG",
    });
    expect(child.layoutPositioning).toBe("ABSOLUTE");
    expect(child.layoutSizingHorizontal).toBe("FILL");
    expect(child.layoutSizingVertical).toBe("HUG");
  });

  it("leaves absent params untouched", () => {
    const child = makeChild();
    applyLayoutSizing(child, { layoutSizingHorizontal: "FILL" });
    expect(child.layoutSizingVertical).toBe("FIXED");
    expect(child.layoutPositioning).toBe("AUTO");
  });

  it("applies positioning before sizing", () => {
    const order: string[] = [];
    const tracked = new Proxy(makeChild(), {
      set(target, key, value) {
        order.push(String(key));
        return Reflect.set(target, key, value);
      },
    });
    applyLayoutSizing(tracked, { layoutPositioning: "ABSOLUTE", layoutSizingHorizontal: "FILL" });
    expect(order).toEqual(["layoutPositioning", "layoutSizingHorizontal"]);
  });
});

// ── base64ToBytes ─────────────────────────────────────────────────────────────

describe("base64ToBytes", () => {
  it("decodes a known base64 string", () => {
    // "Man" → TWFu
    const bytes = base64ToBytes("TWFu");
    expect(bytes).toEqual(new Uint8Array([77, 97, 110]));
  });

  it("decodes base64 with single padding", () => {
    // "Ma" → TWE=
    const bytes = base64ToBytes("TWE=");
    expect(bytes).toEqual(new Uint8Array([77, 97]));
  });

  it("decodes base64 with double padding", () => {
    // "M" → TQ==
    const bytes = base64ToBytes("TQ==");
    expect(bytes).toEqual(new Uint8Array([77]));
  });

  it("decodes a longer string", () => {
    // "Hello" → SGVsbG8=
    const bytes = base64ToBytes("SGVsbG8=");
    expect(Array.from(bytes)).toEqual([72, 101, 108, 108, 111]);
  });

  it("strips non-base64 characters (e.g. newlines)", () => {
    const bytes = base64ToBytes("TW\nFu");
    expect(bytes).toEqual(new Uint8Array([77, 97, 110]));
  });
});

// ── getParentNode ─────────────────────────────────────────────────────────────

describe("getParentNode", () => {
  it("returns currentPage when no parentId given", async () => {
    const result = await getParentNode(undefined);
    expect(result).toBe(mockCurrentPage);
  });

  it("throws when parentId node is not found", async () => {
    await expect(getParentNode("1:999")).rejects.toThrow("Parent node not found: 1:999");
  });

  it("throws when found node cannot have children", async () => {
    mockGetNodeByIdAsync = async () => ({ id: "1:2", name: "rect" }); // no appendChild
    await expect(getParentNode("1:2")).rejects.toThrow("cannot have children");
  });

  it("returns node when it supports appendChild", async () => {
    const parentNode = { id: "1:3", name: "frame", appendChild: () => {} } as any;
    mockGetNodeByIdAsync = async () => parentNode;
    const result = await getParentNode("1:3");
    expect(result).toBe(parentNode);
  });
});
