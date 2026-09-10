// Write helpers — utilities used exclusively by write handlers.

import { parseColor } from "./color";

export const makeSolidPaint = (colorInput: any, opacityOverride?: number): SolidPaint => {
  const { r, g, b, a } = typeof colorInput === "string"
    ? parseColor(colorInput)
    : { r: colorInput.r, g: colorInput.g, b: colorInput.b, a: colorInput.a != null ? colorInput.a : 1 };
  const eff = opacityOverride != null ? opacityOverride : a;
  const paint: any = { type: "SOLID", color: { r, g, b } };
  if (eff !== 1) paint.opacity = eff;
  return paint;
};

export const getParentNode = async (parentId: string | undefined) => {
  if (!parentId) return figma.currentPage;
  const parent = await figma.getNodeByIdAsync(parentId);
  if (!parent) throw new Error(`Parent node not found: ${parentId}`);
  if (!("appendChild" in parent)) throw new Error(`Node ${parentId} cannot have children`);
  return parent as ChildrenMixin & BaseNode;
};

// Container-level auto-layout properties, typed straight off the Figma mixin so
// params and target node share one definition.
export type AutoLayoutProps = Pick<
  AutoLayoutMixin,
  | "layoutMode"
  | "paddingTop"
  | "paddingRight"
  | "paddingBottom"
  | "paddingLeft"
  | "itemSpacing"
  | "primaryAxisAlignItems"
  | "counterAxisAlignItems"
  | "primaryAxisSizingMode"
  | "counterAxisSizingMode"
  | "layoutWrap"
  | "counterAxisSpacing"
  | "counterAxisAlignContent"
  | "itemReverseZIndex"
  | "strokesIncludedInLayout"
>;

export type LayoutSizingProps = Pick<LayoutMixin, "layoutSizingHorizontal" | "layoutSizingVertical"> &
  Pick<AutoLayoutChildrenMixin, "layoutPositioning">;

export const applyAutoLayout = (frame: AutoLayoutProps, p: Partial<AutoLayoutProps>) => {
  if (p.layoutMode != null) frame.layoutMode = p.layoutMode;
  if (p.paddingTop != null) frame.paddingTop = Number(p.paddingTop);
  if (p.paddingRight != null) frame.paddingRight = Number(p.paddingRight);
  if (p.paddingBottom != null) frame.paddingBottom = Number(p.paddingBottom);
  if (p.paddingLeft != null) frame.paddingLeft = Number(p.paddingLeft);
  if (p.itemSpacing != null) frame.itemSpacing = Number(p.itemSpacing);
  if (frame.layoutMode !== "NONE") {
    if (p.primaryAxisAlignItems) frame.primaryAxisAlignItems = p.primaryAxisAlignItems;
    if (p.counterAxisAlignItems) frame.counterAxisAlignItems = p.counterAxisAlignItems;
    if (p.primaryAxisSizingMode) frame.primaryAxisSizingMode = p.primaryAxisSizingMode;
    if (p.counterAxisSizingMode) frame.counterAxisSizingMode = p.counterAxisSizingMode;
    if (p.layoutWrap) frame.layoutWrap = p.layoutWrap;
    if (p.counterAxisSpacing != null && frame.layoutWrap === "WRAP") {
      frame.counterAxisSpacing = Number(p.counterAxisSpacing);
    }
    if (p.counterAxisAlignContent && frame.layoutWrap === "WRAP") {
      frame.counterAxisAlignContent = p.counterAxisAlignContent;
    }
    if (p.itemReverseZIndex != null) frame.itemReverseZIndex = !!p.itemReverseZIndex;
    if (p.strokesIncludedInLayout != null) frame.strokesIncludedInLayout = !!p.strokesIncludedInLayout;
  }
};

// Child-level layout properties. Separate from applyAutoLayout because these are
// only valid once the node sits in its final parent, and create_component copies
// container props off a source node where these must not be read.
export const applyLayoutSizing = (node: LayoutSizingProps, p: Partial<LayoutSizingProps>) => {
  if (p.layoutPositioning) node.layoutPositioning = p.layoutPositioning;
  if (p.layoutSizingHorizontal) node.layoutSizingHorizontal = p.layoutSizingHorizontal;
  if (p.layoutSizingVertical) node.layoutSizingVertical = p.layoutSizingVertical;
};

export const base64ToBytes = (b64: string) => {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const lookup: Record<string, number> = {};
  for (let i = 0; i < chars.length; i++) lookup[chars[i]] = i;
  const padded = b64.replace(/[^A-Za-z0-9+/=]/g, "");
  const clean = padded.replace(/=/g, "");
  let outLen = Math.floor(padded.length * 3 / 4);
  if (padded.endsWith("==")) outLen -= 2;
  else if (padded.endsWith("=")) outLen -= 1;
  const bytes = new Uint8Array(outLen);
  let j = 0;
  for (let i = 0; i < clean.length; i += 4) {
    const a = lookup[clean[i]] || 0;
    const bv = lookup[clean[i + 1]] || 0;
    const c = lookup[clean[i + 2]] || 0;
    const d = lookup[clean[i + 3]] || 0;
    bytes[j++] = (a << 2) | (bv >> 4);
    if (j < outLen) bytes[j++] = ((bv & 15) << 4) | (c >> 2);
    if (j < outLen) bytes[j++] = ((c & 3) << 6) | d;
  }
  return bytes;
};
