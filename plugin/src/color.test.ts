import { describe, it, expect } from "bun:test";
import { parseColor, formatColor } from "./color";

const hex = (input: string) => formatColor(parseColor(input), "hex");

// ── parseColor: hex ───────────────────────────────────────────────────────────

describe("parseColor hex", () => {
  it("converts 6-char hex to rgb with alpha 1", () => {
    const result = parseColor("#ff0000");
    expect(result.r).toBeCloseTo(1);
    expect(result.g).toBe(0);
    expect(result.b).toBe(0);
    expect(result.a).toBe(1);
  });

  it("converts black #000000", () => {
    const result = parseColor("#000000");
    expect(result).toEqual({ r: 0, g: 0, b: 0, a: 1 });
  });

  it("converts white #ffffff", () => {
    const result = parseColor("#ffffff");
    expect(result.r).toBeCloseTo(1);
    expect(result.g).toBeCloseTo(1);
    expect(result.b).toBeCloseTo(1);
  });

  it("converts 8-char hex with alpha", () => {
    const result = parseColor("#ff000080");
    expect(result.r).toBeCloseTo(1);
    expect(result.a).toBeCloseTo(128 / 255);
  });

  it("works without leading #", () => {
    expect(parseColor("00ff00").g).toBeCloseTo(1);
    expect(hex("ff0000")).toBe("#ff0000");
  });

  it("expands 3-digit shorthand", () => {
    expect(hex("#FFF")).toBe("#ffffff");
    expect(hex("#f00")).toBe("#ff0000");
  });

  it("expands 4-digit shorthand with alpha", () => {
    const result = parseColor("#f008");
    expect(hex("#f008")).toBe("#ff000088");
    expect(result.a).toBeCloseTo(8 / 15);
  });
});

// ── parseColor: rgb / hsl / oklch ─────────────────────────────────────────────

describe("parseColor rgb()", () => {
  it("accepts comma, space, and percent forms", () => {
    expect(hex("rgb(59, 130, 246)")).toBe("#3b82f6");
    expect(hex("rgb(59 130 246)")).toBe("#3b82f6");
    expect(hex("rgb(23.1% 51% 96.5%)")).toBe("#3b82f6");
  });

  it("reads alpha from a 4th component or slash syntax", () => {
    expect(parseColor("rgba(59,130,246,0.5)").a).toBe(0.5);
    expect(parseColor("rgb(59 130 246 / 50%)").a).toBe(0.5);
  });
});

describe("parseColor hsl()", () => {
  it("converts hue/saturation/lightness", () => {
    expect(hex("hsl(210 90% 55%)")).toBe("#258cf4");
    expect(hex("hsl(0, 100%, 50%)")).toBe("#ff0000");
    expect(hex("hsl(120 100% 25%)")).toBe("#008000");
  });

  it("tolerates deg units and reads alpha", () => {
    expect(hex("hsl(210deg 90% 55%)")).toBe("#258cf4");
    expect(parseColor("hsla(210 90% 55% / 0.5)").a).toBe(0.5);
  });
});

describe("parseColor oklch()", () => {
  it("converts to sRGB", () => {
    expect(hex("oklch(0.628 0.2577 29.23)")).toBe("#ff0000");
    expect(hex("oklch(0.7 0.1 250)")).toBe("#6da3da");
    expect(hex("oklch(1 0 0)")).toBe("#ffffff");
    expect(hex("oklch(0% 0 0)")).toBe("#000000");
  });

  it("clamps out-of-gamut chroma per channel", () => {
    expect(hex("oklch(0.5 0.4 30)")).toBe("#fd0000");
  });
});

describe("parseColor rejects garbage", () => {
  it("throws on unknown notation", () => {
    expect(() => parseColor("not-a-color")).toThrow(/Unrecognized color/);
    expect(() => parseColor("")).toThrow(/Unrecognized color/);
    expect(() => parseColor("rgb(a, b, c)")).toThrow(/Unrecognized color/);
  });
});

// ── formatColor ───────────────────────────────────────────────────────────────

describe("formatColor", () => {
  const blue = { r: 59 / 255, g: 130 / 255, b: 246 / 255, a: 1 };

  it("emits each notation at full alpha", () => {
    expect(formatColor(blue, "hex")).toBe("#3b82f6");
    expect(formatColor(blue, "rgb")).toBe("rgb(59, 130, 246)");
    expect(formatColor(blue, "hsl")).toBe("hsl(217 91.2% 59.8%)");
    expect(formatColor(blue, "oklch")).toBe("oklch(0.6231 0.1880 259.81)");
  });

  it("carries alpha into each notation", () => {
    const translucent = { ...blue, a: 0.5 };
    expect(formatColor(translucent, "hex")).toBe("#3b82f680");
    expect(formatColor(translucent, "rgb")).toBe("rgba(59, 130, 246, 0.50)");
    expect(formatColor(translucent, "hsl")).toBe("hsl(217 91.2% 59.8% / 0.50)");
    expect(formatColor(translucent, "oklch")).toBe("oklch(0.6231 0.1880 259.81 / 0.50)");
  });

  it("zeroes hue for achromatic colors", () => {
    const white = { r: 1, g: 1, b: 1, a: 1 };
    expect(formatColor(white, "oklch")).toBe("oklch(1.0000 0.0000 0.00)");
    expect(formatColor(white, "hsl")).toBe("hsl(0 0.0% 100.0%)");
  });

  it("defaults to hex", () => {
    expect(formatColor(blue)).toBe("#3b82f6");
  });
});
