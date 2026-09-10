// Color notation — the single parse/format boundary for every color the plugin
// reads or writes. Accepts hex, rgb(), hsl(), oklch(); emits any of the four.

export type Rgba = { r: number; g: number; b: number; a: number };
export type ColorFormat = "hex" | "rgb" | "hsl" | "oklch";

const clamp01 = (v: number) => (v < 0 ? 0 : v > 1 ? 1 : v);

// parseFloat tolerates unit suffixes, so "210deg" and "90%" both yield their number.
const num = (token: string | undefined) => (token == null ? NaN : parseFloat(token));

const isPct = (token: string) => token.trim().endsWith("%");

const parseAlpha = (token: string | undefined) => {
  if (token == null || token.trim() === "") return 1;
  const v = num(token);
  if (isNaN(v)) return NaN;
  return clamp01(isPct(token) ? v / 100 : v);
};

// "59, 130, 246" | "59 130 246 / 50%" | "59,130,246,0.5" -> ["59","130","246","0.5"?]
const splitComponents = (body: string) => {
  const slash = body.split("/");
  const parts = slash[0].trim().split(/[,\s]+/).filter((t) => t !== "");
  if (slash.length > 1) parts.push(slash[1]);
  return parts;
};

const parseHex = (clean: string): Rgba | null => {
  if (!/^[0-9a-f]+$/.test(clean)) return null;
  if (clean.length === 3 || clean.length === 4) {
    const ch = (i: number) => parseInt(clean[i] + clean[i], 16) / 255;
    return { r: ch(0), g: ch(1), b: ch(2), a: clean.length === 4 ? ch(3) : 1 };
  }
  if (clean.length === 6 || clean.length === 8) {
    const ch = (i: number) => parseInt(clean.slice(i, i + 2), 16) / 255;
    return { r: ch(0), g: ch(2), b: ch(4), a: clean.length === 8 ? ch(6) : 1 };
  }
  return null;
};

const hslToRgb = (h: number, s: number, l: number) => {
  const hp = ((((h % 360) + 360) % 360) / 60);
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs((hp % 2) - 1));
  const m = l - c / 2;
  const seg = Math.floor(hp) % 6;
  const table = [[c, x, 0], [x, c, 0], [0, c, x], [0, x, c], [x, 0, c], [c, 0, x]][seg];
  return { r: table[0] + m, g: table[1] + m, b: table[2] + m };
};

const rgbToHsl = (r: number, g: number, b: number) => {
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  const d = max - min;
  if (d < 1e-4) return { h: 0, s: 0, l };
  const s = d / (1 - Math.abs(2 * l - 1));
  let h: number;
  if (max === r) h = ((g - b) / d) % 6;
  else if (max === g) h = (b - r) / d + 2;
  else h = (r - g) / d + 4;
  h *= 60;
  if (h < 0) h += 360;
  return { h, s, l };
};

// sRGB transfer functions.
const gammaEncode = (c: number) =>
  clamp01(c <= 0.0031308 ? 12.92 * c : 1.055 * Math.pow(c, 1 / 2.4) - 0.055);
const gammaDecode = (c: number) =>
  c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);

// ponytail: out-of-gamut oklch is clamped per channel, not chroma-mapped. Swap in
// a binary chroma search if designers start feeding wide-gamut P3 values.
const oklchToRgb = (L: number, C: number, H: number) => {
  const hr = (H * Math.PI) / 180;
  const a = C * Math.cos(hr);
  const bb = C * Math.sin(hr);
  const l_ = L + 0.3963377774 * a + 0.2158037573 * bb;
  const m_ = L - 0.1055613458 * a - 0.0638541728 * bb;
  const s_ = L - 0.0894841775 * a - 1.291485548 * bb;
  const l = l_ * l_ * l_;
  const m = m_ * m_ * m_;
  const s = s_ * s_ * s_;
  return {
    r: gammaEncode(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
    g: gammaEncode(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
    b: gammaEncode(-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s),
  };
};

const rgbToOklch = (r: number, g: number, b: number) => {
  const lr = gammaDecode(r);
  const lg = gammaDecode(g);
  const lb = gammaDecode(b);
  const l = Math.cbrt(0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb);
  const m = Math.cbrt(0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb);
  const s = Math.cbrt(0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb);
  const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const A = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const B = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  const C = Math.sqrt(A * A + B * B);
  if (C < 1e-4) return { L, C: 0, H: 0 };
  let H = (Math.atan2(B, A) * 180) / Math.PI;
  if (H < 0) H += 360;
  return { L, C, H };
};

export const parseColor = (input: string): Rgba => {
  const raw = String(input).trim().toLowerCase();
  const fn = /^(rgb|rgba|hsl|hsla|oklch)\(([^)]*)\)$/.exec(raw);

  if (!fn) {
    const hex = parseHex(raw.charAt(0) === "#" ? raw.slice(1) : raw);
    if (hex) return hex;
    throw new Error(
      `Unrecognized color: ${input} — use hex (#3b82f6), rgb(), hsl(), or oklch()`,
    );
  }

  const parts = splitComponents(fn[2]);
  const a = parseAlpha(parts[3]);
  const c1 = num(parts[0]);
  const c2 = num(parts[1]);
  const c3 = num(parts[2]);
  if (parts.length >= 3 && !isNaN(a) && !isNaN(c1) && !isNaN(c2) && !isNaN(c3)) {
    if (fn[1] === "rgb" || fn[1] === "rgba") {
      const ch = (v: number, token: string) => clamp01(isPct(token) ? v / 100 : v / 255);
      return { r: ch(c1, parts[0]), g: ch(c2, parts[1]), b: ch(c3, parts[2]), a };
    }
    if (fn[1] === "hsl" || fn[1] === "hsla") {
      return { ...hslToRgb(c1, clamp01(c2 / 100), clamp01(c3 / 100)), a };
    }
    // oklch: L may be a percentage, C may be a percentage of the 0.4 reference.
    const L = isPct(parts[0]) ? c1 / 100 : c1;
    const C = isPct(parts[1]) ? (c2 / 100) * 0.4 : c2;
    return { ...oklchToRgb(L, C, c3), a };
  }

  throw new Error(
    `Unrecognized color: ${input} — use hex (#3b82f6), rgb(), hsl(), or oklch()`,
  );
};

export const formatColor = (color: Rgba, format: ColorFormat = "hex"): string => {
  const r = clamp01(color.r);
  const g = clamp01(color.g);
  const b = clamp01(color.b);
  const a = color.a == null ? 1 : clamp01(color.a);

  if (format === "rgb") {
    const byte = (v: number) => Math.round(v * 255);
    return a < 1
      ? `rgba(${byte(r)}, ${byte(g)}, ${byte(b)}, ${a.toFixed(2)})`
      : `rgb(${byte(r)}, ${byte(g)}, ${byte(b)})`;
  }

  if (format === "hsl") {
    const hsl = rgbToHsl(r, g, b);
    const body = `${Math.round(hsl.h)} ${(hsl.s * 100).toFixed(1)}% ${(hsl.l * 100).toFixed(1)}%`;
    return a < 1 ? `hsl(${body} / ${a.toFixed(2)})` : `hsl(${body})`;
  }

  if (format === "oklch") {
    const ok = rgbToOklch(r, g, b);
    const body = `${ok.L.toFixed(4)} ${ok.C.toFixed(4)} ${ok.H.toFixed(2)}`;
    return a < 1 ? `oklch(${body} / ${a.toFixed(2)})` : `oklch(${body})`;
  }

  const byte = (v: number) => Math.round(v * 255).toString(16).padStart(2, "0");
  return `#${byte(r)}${byte(g)}${byte(b)}${a < 1 ? byte(a) : ""}`;
};
