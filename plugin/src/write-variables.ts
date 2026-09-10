import { parseColor } from "./color";
import { assertNotFigjam } from "./figjam";

const parseVariableValue = (type: string, value: any): VariableValue => {
  if (type === "COLOR") {
    if (typeof value === "string") {
      const { r, g, b, a } = parseColor(value);
      return { r, g, b, a };
    }
    return value as RGBA;
  }
  if (type === "FLOAT") return typeof value === "number" ? value : parseFloat(String(value));
  if (type === "BOOLEAN") return value === true || value === "true";
  return String(value); // STRING
};

// Alias wins when aliasVariableId is given; createVariableAliasByIdAsync throws on
// an unknown id and Figma rejects a type mismatch, so no extra checks here.
const resolveVariableValue = async (type: string, p: any): Promise<VariableValue> =>
  p.aliasVariableId
    ? figma.variables.createVariableAliasByIdAsync(p.aliasVariableId)
    : parseVariableValue(type, p.value);

const CODE_SYNTAX_PLATFORMS: Record<string, CodeSyntaxPlatform> = {
  codeSyntaxWeb: "WEB",
  codeSyntaxAndroid: "ANDROID",
  codeSyntaxIos: "iOS",
};

export const handleWriteVariableRequest = async (request: any) => {
  switch (request.type) {
    case "create_variable_collection": {
      const p = request.params || {};
      assertNotFigjam("create_variable_collection");
      if (!p.name) throw new Error("name is required");
      const collection = figma.variables.createVariableCollection(p.name);
      if (p.initialModeName && collection.modes.length > 0) {
        collection.renameMode(collection.modes[0].modeId, p.initialModeName);
      }
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          id: collection.id,
          name: collection.name,
          modes: collection.modes.map((m) => ({ modeId: m.modeId, name: m.name })),
        },
      };
    }

    case "add_variable_mode": {
      const p = request.params || {};
      assertNotFigjam("add_variable_mode");
      if (!p.collectionId) throw new Error("collectionId is required");
      if (!p.modeName) throw new Error("modeName is required");
      const collection = await figma.variables.getVariableCollectionByIdAsync(p.collectionId);
      if (!collection) throw new Error(`Collection not found: ${p.collectionId}`);
      const modeId = collection.addMode(p.modeName);
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: { collectionId: p.collectionId, modeId, modeName: p.modeName },
      };
    }

    case "create_variable": {
      const p = request.params || {};
      assertNotFigjam("create_variable");
      if (!p.name) throw new Error("name is required");
      if (!p.collectionId) throw new Error("collectionId is required");
      const validTypes = ["COLOR", "FLOAT", "STRING", "BOOLEAN"];
      if (!p.type || !validTypes.includes(p.type)) {
        throw new Error("type is required: COLOR, FLOAT, STRING, or BOOLEAN");
      }
      const collection = await figma.variables.getVariableCollectionByIdAsync(p.collectionId);
      if (!collection) throw new Error(`Collection not found: ${p.collectionId}`);
      const variable = figma.variables.createVariable(p.name, collection, p.type as VariableResolvedDataType);
      if ((p.value != null || p.aliasVariableId) && collection.modes.length > 0) {
        const modeId = collection.modes[0].modeId;
        variable.setValueForMode(modeId, await resolveVariableValue(p.type, p));
      }
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          id: variable.id,
          name: variable.name,
          resolvedType: variable.resolvedType,
          collectionId: p.collectionId,
        },
      };
    }

    case "set_variable_value": {
      const p = request.params || {};
      assertNotFigjam("set_variable_value");
      if (!p.variableId) throw new Error("variableId is required");
      if (!p.modeId) throw new Error("modeId is required");
      if (p.value == null && !p.aliasVariableId) {
        throw new Error("value or aliasVariableId is required");
      }
      const variable = await figma.variables.getVariableByIdAsync(p.variableId);
      if (!variable) throw new Error(`Variable not found: ${p.variableId}`);
      variable.setValueForMode(
        p.modeId,
        await resolveVariableValue(variable.resolvedType, p),
      );
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          variableId: variable.id,
          name: variable.name,
          modeId: p.modeId,
          alias: p.aliasVariableId != null ? p.aliasVariableId : null,
        },
      };
    }

    case "update_variable": {
      const p = request.params || {};
      assertNotFigjam("update_variable");
      if (p.variableId) {
        const variable = await figma.variables.getVariableByIdAsync(p.variableId);
        if (!variable) throw new Error(`Variable not found: ${p.variableId}`);
        if (p.name != null) variable.name = p.name;
        if (p.description != null) variable.description = p.description;
        if (p.scopes != null) variable.scopes = p.scopes as VariableScope[];
        if (p.hiddenFromPublishing != null) variable.hiddenFromPublishing = !!p.hiddenFromPublishing;
        for (const key of Object.keys(CODE_SYNTAX_PLATFORMS)) {
          if (p[key] != null) variable.setVariableCodeSyntax(CODE_SYNTAX_PLATFORMS[key], p[key]);
        }
        figma.commitUndo();
        return {
          type: request.type,
          requestId: request.requestId,
          data: {
            variableId: variable.id,
            name: variable.name,
            scopes: variable.scopes,
            codeSyntax: variable.codeSyntax,
          },
        };
      }
      if (p.collectionId) {
        const collection = await figma.variables.getVariableCollectionByIdAsync(p.collectionId);
        if (!collection) throw new Error(`Collection not found: ${p.collectionId}`);
        if (p.name != null) collection.name = p.name;
        if (p.hiddenFromPublishing != null) collection.hiddenFromPublishing = !!p.hiddenFromPublishing;
        if (p.modeId && p.modeName) collection.renameMode(p.modeId, p.modeName);
        figma.commitUndo();
        return {
          type: request.type,
          requestId: request.requestId,
          data: {
            collectionId: collection.id,
            name: collection.name,
            modes: collection.modes.map((m) => ({ modeId: m.modeId, name: m.name })),
          },
        };
      }
      throw new Error("variableId or collectionId is required");
    }

    case "set_variable_mode": {
      const p = request.params || {};
      assertNotFigjam("set_variable_mode");
      if (!p.collectionId) throw new Error("collectionId is required");
      const collection = await figma.variables.getVariableCollectionByIdAsync(p.collectionId);
      if (!collection) throw new Error(`Collection not found: ${p.collectionId}`);
      const nodeId = request.nodeIds && request.nodeIds[0];
      const target = nodeId ? await figma.getNodeByIdAsync(nodeId) : figma.currentPage;
      if (!target) throw new Error(`Node not found: ${nodeId}`);
      if (!("setExplicitVariableModeForCollection" in target)) {
        throw new Error(`Node ${nodeId} does not support variable modes`);
      }
      // Object overloads only: manifest.json runs in dynamic-page mode, where the
      // deprecated string-id overloads throw.
      if (p.modeId) target.setExplicitVariableModeForCollection(collection, p.modeId);
      else target.clearExplicitVariableModeForCollection(collection);
      figma.commitUndo();
      return {
        type: request.type,
        requestId: request.requestId,
        data: {
          id: target.id,
          name: target.name,
          collectionId: collection.id,
          modeId: p.modeId != null ? p.modeId : null,
          cleared: !p.modeId,
        },
      };
    }

    case "delete_variable": {
      const p = request.params || {};
      assertNotFigjam("delete_variable");
      if (p.collectionId && p.modeId) {
        const collection = await figma.variables.getVariableCollectionByIdAsync(p.collectionId);
        if (!collection) throw new Error(`Collection not found: ${p.collectionId}`);
        collection.removeMode(p.modeId);
        figma.commitUndo();
        return {
          type: request.type,
          requestId: request.requestId,
          data: { collectionId: p.collectionId, modeId: p.modeId, deleted: true },
        };
      }
      if (p.variableId) {
        const variable = await figma.variables.getVariableByIdAsync(p.variableId);
        if (!variable) throw new Error(`Variable not found: ${p.variableId}`);
        variable.remove();
        figma.commitUndo();
        return {
          type: request.type,
          requestId: request.requestId,
          data: { variableId: p.variableId, deleted: true },
        };
      } else if (p.collectionId) {
        const collection = await figma.variables.getVariableCollectionByIdAsync(p.collectionId);
        if (!collection) throw new Error(`Collection not found: ${p.collectionId}`);
        collection.remove();
        figma.commitUndo();
        return {
          type: request.type,
          requestId: request.requestId,
          data: { collectionId: p.collectionId, deleted: true },
        };
      } else {
        throw new Error("variableId or collectionId is required");
      }
    }

    default:
      return null;
  }
};
