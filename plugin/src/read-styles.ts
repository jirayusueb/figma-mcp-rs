import {
  serializeBoundVariables,
  serializeVariableValue,
  variableName,
  withAliasName,
} from "./serializers";
import { formatColor, type ColorFormat } from "./color";

const cssVarName = (name: string) =>
  "--" + name.toLowerCase().replace(/[/\s]+/g, "-").replace(/[^a-z0-9-]/g, "");

// Replaces the raw {r,g,b} / {r,g,b,a} on each paint, effect, or layout grid with
// a formatted string. Paints carry their alpha in `opacity`; effects and grids
// carry it on the color itself. Entries with no color (gradients, images, column
// grids) pass through untouched.
const formatColors = (items: readonly any[], colorFormat: ColorFormat) =>
  items.map((item) => {
    if (!item || typeof item !== "object" || !item.color) return item;
    const a = item.color.a != null ? item.color.a : item.opacity != null ? item.opacity : 1;
    return { ...item, color: formatColor({ ...item.color, a }, colorFormat) };
  });

const styleMeta = async (style: BaseStyle) => ({
  id: style.id,
  name: style.name,
  description: style.description || undefined,
  remote: style.remote,
  key: style.key,
  boundVariables: await serializeBoundVariables(style as any),
});

export const handleReadStyleRequest = async (request: any) => {
  switch (request.type) {
    case "get_styles": {
      const colorFormat = ((request.params && request.params.colorFormat) ||
        "hex") as ColorFormat;
      const [paintStyles, textStyles, effectStyles, gridStyles] =
        await Promise.all([
          figma.getLocalPaintStylesAsync(),
          figma.getLocalTextStylesAsync(),
          figma.getLocalEffectStylesAsync(),
          figma.getLocalGridStylesAsync(),
        ]);
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          paints: await Promise.all(
            paintStyles.map(async (s) => ({
              ...(await styleMeta(s)),
              paints: formatColors(s.paints, colorFormat),
            })),
          ),
          text: await Promise.all(
            textStyles.map(async (s) => ({
              ...(await styleMeta(s)),
              fontSize: s.fontSize,
              fontFamily: s.fontName ? s.fontName.family : undefined,
              fontStyle: s.fontName ? s.fontName.style : undefined,
              textDecoration:
                s.textDecoration !== "NONE" ? s.textDecoration : undefined,
              lineHeight: (s as any).lineHeight,
              letterSpacing: (s as any).letterSpacing,
            })),
          ),
          effects: await Promise.all(
            effectStyles.map(async (s) => ({
              ...(await styleMeta(s)),
              effects: formatColors(s.effects, colorFormat),
            })),
          ),
          grids: await Promise.all(
            gridStyles.map(async (s) => ({
              ...(await styleMeta(s)),
              layoutGrids: formatColors(s.layoutGrids, colorFormat),
            })),
          ),
        },
      };
    }

    case "get_variable_defs": {
      const collections =
        await figma.variables.getLocalVariableCollectionsAsync();
      const variableData = await Promise.all(
        collections.map(async (collection) => {
          const variables = await Promise.all(
            collection.variableIds.map((id) =>
              figma.variables.getVariableByIdAsync(id),
            ),
          );
          return {
            id: collection.id,
            name: collection.name,
            defaultModeId: collection.defaultModeId,
            remote: collection.remote,
            hiddenFromPublishing: collection.hiddenFromPublishing,
            key: collection.key,
            modes: collection.modes.map((mode) => ({
              modeId: mode.modeId,
              name: mode.name,
            })),
            variables: await Promise.all(
              variables
                .filter((v) => v !== null)
                .map(async (variable) => ({
                  id: variable!.id,
                  name: variable!.name,
                  resolvedType: variable!.resolvedType,
                  description: variable!.description || undefined,
                  scopes: variable!.scopes,
                  codeSyntax: variable!.codeSyntax,
                  hiddenFromPublishing: variable!.hiddenFromPublishing,
                  remote: variable!.remote,
                  key: variable!.key,
                  variableCollectionId: variable!.variableCollectionId,
                  valuesByMode: Object.fromEntries(
                    await Promise.all(
                      Object.entries(variable!.valuesByMode).map(
                        async ([modeId, value]) => [
                          modeId,
                          await withAliasName(serializeVariableValue(value)),
                        ],
                      ),
                    ),
                  ),
                })),
            ),
          };
        }),
      );
      return {
        type: request.type,
        requestId: request.requestId,
        data: { collections: variableData },
      };
    }

    case "get_local_components": {
      const pages = figma.root.children;
      const allComponents: any[] = [];
      const componentSetsMap = new Map<string, any>();
      for (let i = 0; i < pages.length; i++) {
        const page = pages[i];
        await page.loadAsync();
        const pageNodes = page.findAllWithCriteria({
          types: ["COMPONENT", "COMPONENT_SET"],
        });
        for (const n of pageNodes) {
          if (n.type === "COMPONENT_SET") {
            componentSetsMap.set(n.id, {
              id: n.id,
              name: n.name,
              key: "key" in n ? n.key : null,
            });
          } else {
            const parentIsSet =
              n.parent && n.parent.type === "COMPONENT_SET";
            allComponents.push({
              id: n.id,
              name: n.name,
              key: "key" in n ? n.key : null,
              componentSetId: parentIsSet ? n.parent!.id : null,
              variantProperties:
                "variantProperties" in n ? n.variantProperties : null,
            });
          }
        }
        figma.ui.postMessage({
          type: "progress_update",
          requestId: request.requestId,
          progress: Math.round(((i + 1) / pages.length) * 90) + 1,
          message: `Scanned ${page.name}: ${allComponents.length} components so far`,
        });
        await new Promise((r) => setTimeout(r, 0));
      }
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          count: allComponents.length,
          components: allComponents,
          componentSets: Array.from(componentSetsMap.values()),
        },
      };
    }

    case "get_annotations": {
      const nodeId = request.params && request.params.nodeId;
      const nodeAnnotations = (n: any) => {
        const anns = n.annotations;
        return Array.isArray(anns) ? anns : null;
      };
      if (nodeId) {
        const node = await figma.getNodeByIdAsync(nodeId);
        if (!node) throw new Error(`Node not found: ${nodeId}`);
        const mergedAnnotations: any[] = [];
        const collect = async (n: any) => {
          const anns = nodeAnnotations(n);
          if (anns)
            for (const a of anns)
              mergedAnnotations.push({ nodeId: n.id, annotation: a });
          if ("children" in n)
            for (const child of n.children) await collect(child);
        };
        await collect(node);
        return {
          type: request.type,
          requestId: request.requestId,
          data: {
            nodeId: node.id,
            name: node.name,
            annotations: mergedAnnotations,
          },
        };
      }
      const annotated: any[] = [];
      const processNode = async (n: any) => {
        const anns = nodeAnnotations(n);
        if (anns && anns.length > 0)
          annotated.push({ nodeId: n.id, name: n.name, annotations: anns });
        if ("children" in n)
          for (const child of n.children) await processNode(child);
      };
      await processNode(figma.currentPage);
      return {
        type: request.type,
        requestId: request.requestId,
        data: { annotatedNodes: annotated },
      };
    }

    case "export_tokens": {
      const format = (request.params && request.params.format) || "json";
      const requestedFormat = request.params && request.params.colorFormat;
      const colorFormat = (requestedFormat ||
        (format === "css" ? "rgb" : "hex")) as ColorFormat;

      const collections = await figma.variables.getLocalVariableCollectionsAsync();
      const paintStyles = await figma.getLocalPaintStylesAsync();

      if (format === "css") {
        const lines: string[] = [":root {"];
        for (const coll of collections) {
          const firstMode = coll.modes[0];
          if (!firstMode) continue;
          for (const varId of coll.variableIds) {
            const variable = await figma.variables.getVariableByIdAsync(varId);
            if (!variable) continue;
            const val = variable.valuesByMode[firstMode.modeId];
            let cssValue: string | null = null;
            if (val && typeof val === "object" && "type" in val && val.type === "VARIABLE_ALIAS") {
              const target = await variableName(val.id);
              if (target) cssValue = `var(${cssVarName(target)})`;
            } else if (variable.resolvedType === "COLOR" && val && typeof val === "object" && "r" in val) {
              cssValue = formatColor(val as RGBA, colorFormat);
            } else if (variable.resolvedType === "FLOAT" || variable.resolvedType === "STRING" || variable.resolvedType === "BOOLEAN") {
              cssValue = String(val);
            }
            if (cssValue !== null) lines.push(`  ${cssVarName(variable.name)}: ${cssValue};`);
          }
        }
        for (const style of paintStyles) {
          if (style.paints.length === 1 && style.paints[0].type === "SOLID") {
            const paint = style.paints[0] as SolidPaint;
            const a = paint.opacity != null ? paint.opacity : 1;
            const cssValue = formatColor({ ...paint.color, a }, colorFormat);
            lines.push(`  ${cssVarName(style.name)}: ${cssValue};`);
          }
        }
        lines.push("}");
        return { type: request.type, requestId: request.requestId, data: { css: lines.join("\n") } };
      }

      // JSON format: nested token tree per collection
      const tokens: any = {};
      for (const coll of collections) {
        const collTokens: any = {};
        for (const varId of coll.variableIds) {
          const variable = await figma.variables.getVariableByIdAsync(varId);
          if (!variable) continue;
          const modeValues: any = {};
          for (const mode of coll.modes) {
            const serialized = await withAliasName(
              serializeVariableValue(variable.valuesByMode[mode.modeId]),
            );
            const isColor =
              !!serialized && typeof serialized === "object" && (serialized as any).type === "COLOR";
            modeValues[mode.name] =
              requestedFormat && isColor
                ? formatColor(serialized as RGBA, colorFormat)
                : serialized;
          }
          const parts = variable.name.split("/");
          let obj = collTokens;
          for (let i = 0; i < parts.length - 1; i++) {
            if (!obj[parts[i]]) obj[parts[i]] = {};
            obj = obj[parts[i]];
          }
          obj[parts[parts.length - 1]] = { type: variable.resolvedType, value: modeValues };
        }
        tokens[coll.name] = collTokens;
      }
      const styleTokens: any = {};
      for (const style of paintStyles) {
          if (style.paints.length === 1 && style.paints[0].type === "SOLID") {
            const paint = style.paints[0] as SolidPaint;
            const a = paint.opacity != null ? paint.opacity : 1;
            const parts = style.name.split("/");
            let obj = styleTokens;
            for (let i = 0; i < parts.length - 1; i++) {
              if (!obj[parts[i]]) obj[parts[i]] = {};
              obj = obj[parts[i]];
            }
            obj[parts[parts.length - 1]] = {
              type: "COLOR",
              value: formatColor({ ...paint.color, a }, colorFormat),
            };
          }
      }
      if (Object.keys(styleTokens).length > 0) {
        tokens["_styles"] = { paint: styleTokens };
      }
      return { type: request.type, requestId: request.requestId, data: { tokens } };
    }

    default:
      return null;
  }
};
