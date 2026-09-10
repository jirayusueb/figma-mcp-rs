import { makeSolidPaint } from "./write-helpers";
import { parseColor } from "./color";
import { assertNotFigjam } from "./figjam";

// Each style type accepts only its own fields — a fontSize on a paint style is a
// caller mistake, not a silent no-op.
const STYLE_FIELDS: Record<string, readonly string[]> = {
  PAINT: ["name", "description", "color"],
  TEXT: [
    "name", "description", "fontFamily", "fontStyle", "fontSize", "textDecoration",
    "lineHeightValue", "lineHeightUnit", "letterSpacingValue", "letterSpacingUnit",
  ],
  EFFECT: ["name", "description", "effects"],
  GRID: [
    "name", "description", "pattern", "count", "gutterSize", "offset", "alignment",
    "sectionSize", "color", "opacity",
  ],
};

const STYLE_BINDABLE_FIELDS: Record<string, readonly string[]> = {
  PAINT: ["color"],
  TEXT: [
    "fontFamily", "fontSize", "fontStyle", "fontWeight", "letterSpacing", "lineHeight",
    "paragraphSpacing", "paragraphIndent",
  ],
  EFFECT: ["color", "radius", "spread", "offsetX", "offsetY"],
  GRID: ["sectionSize", "count", "offset", "gutterSize"],
};

// Shared by set_effects and update_style: one array-entry effect schema.
const buildEffect = (e: any): Effect => {
  switch (e.type) {
    case "DROP_SHADOW":
    case "INNER_SHADOW": {
      const { r, g, b } = parseColor(e.color || "#000000");
      return {
        type: e.type as "DROP_SHADOW" | "INNER_SHADOW",
        color: { r, g, b, a: e.opacity != null ? Number(e.opacity) : 0.25 },
        offset: { x: Number(e.offsetX ?? 0), y: Number(e.offsetY ?? 4) },
        radius: Number(e.radius ?? 4),
        spread: Number(e.spread ?? 0),
        visible: e.visible ?? true,
        blendMode: (e.blendMode || "NORMAL") as BlendMode,
      } as DropShadowEffect;
    }
    case "LAYER_BLUR":
    case "BACKGROUND_BLUR":
      return {
        type: e.type as "LAYER_BLUR" | "BACKGROUND_BLUR",
        radius: Number(e.radius ?? 4),
        visible: e.visible ?? true,
      } as BlurEffect;
    default:
      throw new Error(`Unknown effect type: ${e.type}. Must be DROP_SHADOW, INNER_SHADOW, LAYER_BLUR, or BACKGROUND_BLUR`);
  }
};

// Shared by create_grid_style and update_style.
const buildLayoutGrid = (p: any): LayoutGrid => {
  const pattern = p.pattern || "GRID";
  if (pattern === "COLUMNS" || pattern === "ROWS") {
    return {
      pattern,
      count: Number(p.count ?? 12),
      gutterSize: Number(p.gutterSize ?? 16),
      offset: Number(p.offset ?? 0),
      alignment: p.alignment || "STRETCH",
      visible: true,
    };
  }
  const { r, g, b, a } = parseColor(p.color || "#FF0000");
  return {
    pattern: "GRID",
    sectionSize: Number(p.sectionSize ?? 8),
    visible: true,
    color: { r, g, b, a: p.opacity != null ? Number(p.opacity) : (a !== 1 ? a : 0.1) },
  };
};

export const handleWriteStyleRequest = async (request: any) => {
  switch (request.type) {
    case "create_paint_style": {
      assertNotFigjam("create_paint_style");
      const p = request.params || {};
      if (!p.name) throw new Error("name is required");
      if (!p.color) throw new Error("color is required");
      const existing = (await figma.getLocalPaintStylesAsync()).find(s => s.name === p.name);
      if (existing) {
        return { type: request.type, requestId: request.requestId, data: { id: existing.id, name: existing.name } };
      }
      const style = figma.createPaintStyle();
      style.name = p.name;
      style.paints = [makeSolidPaint(p.color)];
      if (p.description) style.description = p.description;
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { id: style.id, name: style.name },
      };
    }

    case "create_text_style": {
      assertNotFigjam("create_text_style");
      const p = request.params || {};
      if (!p.name) throw new Error("name is required");
      const existing = (await figma.getLocalTextStylesAsync()).find(s => s.name === p.name);
      if (existing) {
        return { type: request.type, requestId: request.requestId, data: { id: existing.id, name: existing.name } };
      }
      const family = p.fontFamily || "Inter";
      const fontStyle = p.fontStyle || "Regular";
      await figma.loadFontAsync({ family, style: fontStyle });
      const style = figma.createTextStyle();
      style.name = p.name;
      style.fontName = { family, style: fontStyle };
      if (p.fontSize != null) style.fontSize = Number(p.fontSize);
      if (p.description) style.description = p.description;
      if (p.textDecoration && p.textDecoration !== "NONE") {
        style.textDecoration = p.textDecoration;
      }
      if (p.lineHeightValue != null) {
        style.lineHeight = { value: Number(p.lineHeightValue), unit: p.lineHeightUnit || "PIXELS" };
      }
      if (p.letterSpacingValue != null) {
        style.letterSpacing = { value: Number(p.letterSpacingValue), unit: p.letterSpacingUnit || "PIXELS" };
      }
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { id: style.id, name: style.name },
      };
    }

    case "create_effect_style": {
      assertNotFigjam("create_effect_style");
      const p = request.params || {};
      if (!p.name) throw new Error("name is required");
      const existing = (await figma.getLocalEffectStylesAsync()).find(s => s.name === p.name);
      if (existing) {
        return { type: request.type, requestId: request.requestId, data: { id: existing.id, name: existing.name } };
      }
      const effectType = p.type || "DROP_SHADOW";
      let effect: Effect;
      if (effectType === "LAYER_BLUR") {
        effect = { type: "LAYER_BLUR", blurType: "NORMAL", radius: Number(p.radius ?? 4), visible: true };
      } else if (effectType === "BACKGROUND_BLUR") {
        effect = { type: "BACKGROUND_BLUR", blurType: "NORMAL", radius: Number(p.radius ?? 4), visible: true };
      } else {
        // DROP_SHADOW or INNER_SHADOW
        const { r, g, b, a } = parseColor(p.color || "#000000");
        const alpha = p.opacity != null ? Number(p.opacity) : (a !== 1 ? a : 0.25);
        effect = {
          type: effectType as "DROP_SHADOW" | "INNER_SHADOW",
          color: { r, g, b, a: alpha },
          offset: { x: Number(p.offsetX ?? 0), y: Number(p.offsetY ?? 4) },
          radius: Number(p.radius ?? 8),
          spread: Number(p.spread ?? 0),
          visible: true,
          blendMode: "NORMAL",
        };
      }
      const style = figma.createEffectStyle();
      style.name = p.name;
      style.effects = [effect];
      if (p.description) style.description = p.description;
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { id: style.id, name: style.name },
      };
    }

    case "create_grid_style": {
      const p = request.params || {};
      assertNotFigjam("create_grid_style");
      if (!p.name) throw new Error("name is required");
      const existing = (await figma.getLocalGridStylesAsync()).find(s => s.name === p.name);
      if (existing) {
        return { type: request.type, requestId: request.requestId, data: { id: existing.id, name: existing.name } };
      }
      const grid = buildLayoutGrid(p);
      const style = figma.createGridStyle();
      style.name = p.name;
      style.layoutGrids = [grid];
      if (p.description) style.description = p.description;
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { id: style.id, name: style.name },
      };
    }

    case "update_style": {
      const p = request.params || {};
      assertNotFigjam("update_style");
      if (!p.styleId) throw new Error("styleId is required");
      const style = await figma.getStyleByIdAsync(p.styleId);
      if (!style) throw new Error(`Style not found: ${p.styleId}`);
      const allowed = STYLE_FIELDS[style.type];
      if (!allowed) throw new Error(`Unknown style type: ${style.type}`);
      for (const key of Object.keys(p)) {
        if (key !== "styleId" && !allowed.includes(key)) {
          throw new Error(`${key} is not applicable to a ${style.type} style`);
        }
      }
      if (p.name != null) style.name = p.name;
      if (p.description != null) style.description = p.description;
      if (style.type === "PAINT") {
        if (p.color != null) (style as PaintStyle).paints = [makeSolidPaint(p.color)];
      } else if (style.type === "TEXT") {
        const text = style as TextStyle;
        if (p.fontFamily != null || p.fontStyle != null) {
          const family = p.fontFamily != null ? p.fontFamily : text.fontName.family;
          const fontStyle = p.fontStyle != null ? p.fontStyle : text.fontName.style;
          await figma.loadFontAsync({ family, style: fontStyle });
          text.fontName = { family, style: fontStyle };
        }
        if (p.fontSize != null) text.fontSize = Number(p.fontSize);
        if (p.textDecoration != null) text.textDecoration = p.textDecoration;
        if (p.lineHeightValue != null) {
          text.lineHeight = { value: Number(p.lineHeightValue), unit: p.lineHeightUnit || "PIXELS" };
        }
        if (p.letterSpacingValue != null) {
          text.letterSpacing = { value: Number(p.letterSpacingValue), unit: p.letterSpacingUnit || "PIXELS" };
        }
      } else if (style.type === "EFFECT") {
        if (p.effects != null) {
          if (!Array.isArray(p.effects)) throw new Error("effects array is required");
          (style as EffectStyle).effects = p.effects.map(buildEffect);
        }
      } else {
        const gridStyle = style as GridStyle;
        const current: any = gridStyle.layoutGrids[0];
        // A new pattern means a differently shaped grid — rebuild it. Otherwise
        // patch the existing one so untouched fields survive.
        if (p.pattern != null || !current) {
          gridStyle.layoutGrids = [buildLayoutGrid(p)];
        } else {
          const next: any = { ...current };
          if (p.count != null) next.count = Number(p.count);
          if (p.gutterSize != null) next.gutterSize = Number(p.gutterSize);
          if (p.offset != null) next.offset = Number(p.offset);
          if (p.alignment != null) next.alignment = p.alignment;
          if (p.sectionSize != null) next.sectionSize = Number(p.sectionSize);
          if (p.color != null || p.opacity != null) {
            const base = p.color != null
              ? parseColor(p.color)
              : { ...(current.color || { r: 1, g: 0, b: 0, a: 0.1 }) };
            next.color = {
              r: base.r,
              g: base.g,
              b: base.b,
              a: p.opacity != null ? Number(p.opacity) : base.a,
            };
          }
          gridStyle.layoutGrids = [next];
        }
      }
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { id: style.id, name: style.name, type: style.type },
      };
    }

    case "bind_variable_to_style": {
      const p = request.params || {};
      assertNotFigjam("bind_variable_to_style");
      if (!p.styleId) throw new Error("styleId is required");
      if (!p.field) throw new Error("field is required");
      const style = await figma.getStyleByIdAsync(p.styleId);
      if (!style) throw new Error(`Style not found: ${p.styleId}`);
      const variable = p.variableId
        ? await figma.variables.getVariableByIdAsync(p.variableId)
        : null;
      if (p.variableId && !variable) throw new Error(`Variable not found: ${p.variableId}`);
      const bindable = STYLE_BINDABLE_FIELDS[style.type];
      if (!bindable) throw new Error(`Unknown style type: ${style.type}`);
      if (!bindable.includes(p.field)) {
        throw new Error(
          `field ${p.field} is not bindable on a ${style.type} style — expected ${bindable.join(", ")}`,
        );
      }
      if (style.type === "PAINT") {
        const paintStyle = style as PaintStyle;
        const paints = [...paintStyle.paints];
        const base = paints.length > 0 ? paints[0] : makeSolidPaint("#000000");
        if (base.type !== "SOLID") {
          throw new Error(`Style ${p.styleId} paint 0 is ${base.type}, not SOLID`);
        }
        paints[0] = figma.variables.setBoundVariableForPaint(base, "color", variable);
        paintStyle.paints = paints;
      } else if (style.type === "TEXT") {
        (style as TextStyle).setBoundVariable(p.field as VariableBindableTextField, variable);
      } else if (style.type === "EFFECT") {
        const effectStyle = style as EffectStyle;
        const effects = [...effectStyle.effects];
        if (effects.length === 0) throw new Error(`Style ${p.styleId} has no effects to bind`);
        effects[0] = figma.variables.setBoundVariableForEffect(
          effects[0],
          p.field as VariableBindableEffectField,
          variable,
        );
        effectStyle.effects = effects;
      } else {
        const gridStyle = style as GridStyle;
        const grids = [...gridStyle.layoutGrids];
        if (grids.length === 0) throw new Error(`Style ${p.styleId} has no layout grids to bind`);
        grids[0] = figma.variables.setBoundVariableForLayoutGrid(
          grids[0],
          p.field as VariableBindableLayoutGridField,
          variable,
        );
        gridStyle.layoutGrids = grids;
      }
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          styleId: style.id,
          styleType: style.type,
          field: p.field,
          variableId: p.variableId != null ? p.variableId : null,
          bound: variable !== null,
        },
      };
    }

    case "delete_style": {
      const p = request.params || {};
      assertNotFigjam("delete_style");
      if (!p.styleId) throw new Error("styleId is required");
      const style = await figma.getStyleByIdAsync(p.styleId);
      if (!style) throw new Error(`Style not found: ${p.styleId}`);
      style.remove();
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { styleId: p.styleId, deleted: true },
      };
    }

    case "apply_style_to_node": {
      const p = request.params || {};
      assertNotFigjam("apply_style_to_node");
      const nodeId = request.nodeIds && request.nodeIds[0];
      if (!nodeId) throw new Error("nodeId is required");
      if (!p.styleId) throw new Error("styleId is required");
      const node = await figma.getNodeByIdAsync(nodeId);
      if (!node) throw new Error(`Node not found: ${nodeId}`);
      const style = await figma.getStyleByIdAsync(p.styleId);
      if (!style) throw new Error(`Style not found: ${p.styleId}`);
      const n = node as any;
      switch (style.type) {
        case "PAINT": {
          const target = p.target || "fill";
          if (target === "stroke") {
            if (!("strokeStyleId" in node)) throw new Error(`Node ${nodeId} does not support stroke styles`);
            await n.setStrokeStyleIdAsync(p.styleId);
          } else {
            if (!("fillStyleId" in node)) throw new Error(`Node ${nodeId} does not support fill styles`);
            await n.setFillStyleIdAsync(p.styleId);
          }
          break;
        }
        case "TEXT":
          if (!("textStyleId" in node)) throw new Error(`Node ${nodeId} does not support text styles`);
          await n.setTextStyleIdAsync(p.styleId);
          break;
        case "EFFECT":
          if (!("effectStyleId" in node)) throw new Error(`Node ${nodeId} does not support effect styles`);
          await n.setEffectStyleIdAsync(p.styleId);
          break;
        case "GRID":
          if (!("gridStyleId" in node)) throw new Error(`Node ${nodeId} does not support grid styles`);
          await n.setGridStyleIdAsync(p.styleId);
          break;
        default:
          throw new Error(`Unknown style type: ${(style as any).type}`);
      }
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { id: n.id, name: n.name, styleId: p.styleId, styleType: style.type },
      };
    }

    case "set_effects": {
      const p = request.params || {};
      assertNotFigjam("set_effects");
      const nodeId = request.nodeIds && request.nodeIds[0];
      if (!nodeId) throw new Error("nodeId is required");
      if (!Array.isArray(p.effects)) throw new Error("effects array is required");
      const node = await figma.getNodeByIdAsync(nodeId) as any;
      if (!node) throw new Error(`Node not found: ${nodeId}`);
      if (!("effects" in node)) throw new Error(`Node ${nodeId} does not support effects`);
      const effects: Effect[] = p.effects.map(buildEffect);
      node.effects = effects;
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { id: node.id, name: node.name, effectCount: effects.length },
      };
    }

    case "bind_variable_to_node": {
      const p = request.params || {};
      assertNotFigjam("bind_variable_to_node");
      const nodeId = request.nodeIds && request.nodeIds[0];
      if (!nodeId) throw new Error("nodeId is required");
      if (!p.field) throw new Error("field is required");
      const node = await figma.getNodeByIdAsync(nodeId) as any;
      if (!node) throw new Error(`Node not found: ${nodeId}`);
      const variable = p.variableId
        ? await figma.variables.getVariableByIdAsync(p.variableId)
        : null;
      if (p.variableId && !variable) throw new Error(`Variable not found: ${p.variableId}`);
      if (p.field === "fillColor") {
        if (!("fills" in node)) throw new Error(`Node ${nodeId} does not support fills`);
        const fills = [...(node.fills as Paint[])];
        const base = fills.length > 0 ? fills[0] : makeSolidPaint("#000000");
        if (base.type !== "SOLID") throw new Error(`Node ${nodeId} fill 0 is ${base.type}, not SOLID`);
        fills[0] = figma.variables.setBoundVariableForPaint(base as SolidPaint, "color", variable);
        node.fills = fills;
      } else if (p.field === "strokeColor") {
        if (!("strokes" in node)) throw new Error(`Node ${nodeId} does not support strokes`);
        const strokes = [...(node.strokes as Paint[])];
        const base = strokes.length > 0 ? strokes[0] : makeSolidPaint("#000000");
        if (base.type !== "SOLID") throw new Error(`Node ${nodeId} stroke 0 is ${base.type}, not SOLID`);
        strokes[0] = figma.variables.setBoundVariableForPaint(base as SolidPaint, "color", variable);
        node.strokes = strokes;
      } else {
        if (!(p.field in node)) throw new Error(`Node ${nodeId} does not have field: ${p.field}`);
        node.setBoundVariable(p.field, variable);
      }
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          id: node.id,
          name: node.name,
          variableId: p.variableId != null ? p.variableId : null,
          field: p.field,
          bound: variable !== null,
        },
      };
    }

    default:
      return null;
  }
};
