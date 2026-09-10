// FigJam-specific tools: stickies, shapes, tables, code blocks, auto-arrange, board reads.

import { isFigjam } from "./figjam";
import { makeSolidPaint, getBounds } from "./write-helpers";

export const handleFigjamToolRequest = async (request: unknown) => {
  const req = request as any;
  if (!req || typeof req !== "object") return null;

  switch (req.type) {
    case "create_stickies":
      return handleCreateStickies(req);
    case "create_shape_with_text":
      return handleCreateShapeWithText(req);
    case "create_table":
      return handleCreateTable(req);
    case "create_code_block":
      return handleCreateCodeBlock(req);
    case "auto_arrange":
      return handleAutoArrange(req);
    case "get_board_contents":
      return handleGetBoardContents(req);
    default:
      return null;
  }
};

async function handleCreateStickies(request: any) {
  if (!isFigjam()) throw new Error("create_stickies is only available in FigJam");
  const p = request.params || {};
  const items = Array.isArray(p.items) ? p.items : [];
  if (!items.length) throw new Error("items must be a non-empty array");
  if (items.length > 200) throw new Error("items supports at most 200 stickies per call");

  const startX = p.startX ?? 0;
  const startY = p.startY ?? 0;
  const columns = p.columns ?? 5;
  const spacing = p.spacing ?? 40;

  const nodes = [];
  const grid_items = items.filter(it => it.x == null || it.y == null);
  let row = 0, col = 0;

  for (const item of items) {
    const sticky = figma.createSticky();
    const fontName = typeof sticky.text.fontName === "symbol"
      ? { family: "Inter", style: "Regular" }
      : sticky.text.fontName;
    await figma.loadFontAsync(fontName);
    sticky.text.characters = item.text || "";
    if (item.fillColor) sticky.fills = [makeSolidPaint(item.fillColor)];
    if (item.name) sticky.name = item.name;

    if (item.x != null && item.y != null) {
      sticky.x = item.x;
      sticky.y = item.y;
    } else {
      sticky.x = startX + col * (sticky.width + spacing);
      sticky.y = startY + row * (sticky.height + spacing);
      col++;
      if (col >= columns) {
        col = 0;
        row++;
      }
    }

    nodes.push({ id: sticky.id, name: sticky.name, type: sticky.type, bounds: getBounds(sticky) });
  }

  figma.commitUndo();
  return {
    type: request.type,
    requestId: request.requestId,
    data: { created: nodes.length, nodes },
  };
}

async function handleCreateShapeWithText(request: any) {
  if (!isFigjam()) throw new Error("create_shape_with_text is only available in FigJam");
  const p = request.params || {};
  if (!p.text) throw new Error("text is required");

  const shapeTypes = [
    "SQUARE", "ELLIPSE", "ROUNDED_RECTANGLE", "DIAMOND", "TRIANGLE_UP", "TRIANGLE_DOWN",
    "PARALLELOGRAM_RIGHT", "PARALLELOGRAM_LEFT", "HEXAGON", "PENTAGON", "OCTAGON", "STAR",
    "PLUS", "ARROW_LEFT", "ARROW_RIGHT", "ENG_DATABASE", "ENG_QUEUE", "ENG_FILE", "ENG_FOLDER",
    "TRAPEZOID", "PREDEFINED_PROCESS", "SHIELD", "DOCUMENT_SINGLE", "DOCUMENT_MULTIPLE",
    "MANUAL_INPUT", "SUMMING_JUNCTION", "OR", "SPEECH_BUBBLE", "INTERNAL_STORAGE", "CHEVRON"
  ];
  const shapeType = (p.shapeType || "ROUNDED_RECTANGLE").toUpperCase();
  if (!shapeTypes.includes(shapeType)) throw new Error(`shapeType must be one of: ${shapeTypes.join(", ")}`);

  const shape = figma.createShapeWithText();
  const fontName = typeof shape.text.fontName === "symbol"
    ? { family: "Inter", style: "Regular" }
    : shape.text.fontName;
  await figma.loadFontAsync(fontName);
  shape.text.characters = p.text;
  if (p.fillColor) shape.fills = [makeSolidPaint(p.fillColor)];
  if (p.x != null) shape.x = p.x;
  if (p.y != null) shape.y = p.y;
  if (p.width != null && p.height != null) shape.resize(p.width, p.height);

  figma.commitUndo();
  return {
    type: request.type,
    requestId: request.requestId,
    data: { id: shape.id, name: shape.name, type: shape.type, shapeType, bounds: getBounds(shape) },
  };
}

async function handleCreateTable(request: any) {
  if (!isFigjam()) throw new Error("create_table is only available in FigJam");
  const p = request.params || {};
  const rows = Math.floor(Number(p.rows) || 1);
  const cols = Math.floor(Number(p.columns) || 1);
  if (rows < 1 || rows > 50) throw new Error("rows must be between 1 and 50");
  if (cols < 1 || cols > 50) throw new Error("columns must be between 1 and 50");

  const table = figma.createTable(rows, cols);
  const cells = Array.isArray(p.cells) ? p.cells : [];

  for (let r = 0; r < cells.length && r < rows; r++) {
    const row_cells = Array.isArray(cells[r]) ? cells[r] : [];
    for (let c = 0; c < row_cells.length && c < cols; c++) {
      const cell = table.cellAt(r, c);
      if (typeof row_cells[c] === "string") {
        const fontName = typeof cell.text.fontName === "symbol"
          ? { family: "Inter", style: "Regular" }
          : cell.text.fontName;
        await figma.loadFontAsync(fontName);
        cell.text.characters = row_cells[c];
      }
    }
  }

  if (p.x != null) table.x = p.x;
  if (p.y != null) table.y = p.y;
  if (p.name) table.name = p.name;

  figma.commitUndo();
  return {
    type: request.type,
    requestId: request.requestId,
    data: { id: table.id, name: table.name, type: table.type, rows, columns: cols, bounds: getBounds(table) },
  };
}

async function handleCreateCodeBlock(request: any) {
  if (!isFigjam()) throw new Error("create_code_block is only available in FigJam");
  const p = request.params || {};
  if (!p.code) throw new Error("code is required");

  const languages = ["TYPESCRIPT", "JAVASCRIPT", "HTML", "CSS", "JSON", "GRAPHQL", "PYTHON",
    "GO", "SQL", "SWIFT", "KOTLIN", "RUST", "BASH", "RUBY", "CPP", "PLAINTEXT"];
  const language = (p.language || "PLAINTEXT").toUpperCase();
  if (!languages.includes(language)) throw new Error(`language must be one of: ${languages.join(", ")}`);

  const codeBlock = figma.createCodeBlock();
  codeBlock.code = p.code;
  codeBlock.codeLanguage = language;
  if (p.x != null) codeBlock.x = p.x;
  if (p.y != null) codeBlock.y = p.y;

  figma.commitUndo();
  return {
    type: request.type,
    requestId: request.requestId,
    data: { id: codeBlock.id, name: codeBlock.name, type: codeBlock.type, codeLanguage: language, bounds: getBounds(codeBlock) },
  };
}

async function handleAutoArrange(request: any) {
  if (!isFigjam()) throw new Error("auto_arrange is only available in FigJam");
  const p = request.params || {};
  const layout = (p.layout || "grid").toLowerCase();
  if (!["grid", "row", "column"].includes(layout)) throw new Error("layout must be one of: grid, row, column");

  let targets: SceneNode[] = [];
  if (Array.isArray(p.nodeIds) && p.nodeIds.length) {
    const skipped: string[] = [];
    for (const id of p.nodeIds) {
      const node = await figma.getNodeByIdAsync(id);
      if (node) targets.push(node);
      else skipped.push(id);
    }
    if (!targets.length) throw new Error("no nodes to arrange");
  } else {
    targets = figma.currentPage.children.slice();
  }

  const spacing = Number(p.spacing) || 40;
  const cols = layout === "grid" ? Math.ceil(Math.sqrt(targets.length)) : undefined;

  let x = p.startX ?? 0, y = p.startY ?? 0;
  for (const node of targets) {
    (node as any).x = x;
    (node as any).y = y;
    const w = (node as any).width || 100;
    const h = (node as any).height || 100;
    if (layout === "row") {
      x += w + spacing;
    } else if (layout === "column") {
      y += h + spacing;
    } else {
      x += w + spacing;
      if (cols && targets.indexOf(node) % cols === cols - 1) {
        x = (p.startX ?? 0);
        y += h + spacing;
      }
    }
  }

  const minX = Math.min(...targets.map(n => (n as any).x));
  const minY = Math.min(...targets.map(n => (n as any).y));
  const maxX = Math.max(...targets.map(n => (n as any).x + ((n as any).width || 100)));
  const maxY = Math.max(...targets.map(n => (n as any).y + ((n as any).height || 100)));

  figma.commitUndo();
  return {
    type: request.type,
    requestId: request.requestId,
    data: { arranged: targets.length, skipped: [], bounds: { x: minX, y: minY, width: maxX - minX, height: maxY - minY } },
  };
}

function handleGetBoardContents(request: any) {
  if (!isFigjam()) throw new Error("get_board_contents is only available in FigJam");
  const p = request.params || {};
  const includeConnections = p.includeConnections !== false;

  const nodes = [];
  const connections = [];

  const walk = (children: SceneNode[]) => {
    for (const node of children) {
      const base = { id: node.id, name: node.name, type: node.type, bounds: getBounds(node) };

      if (node.type === "STICKY") {
        nodes.push({ ...base, text: (node as any).text?.characters ?? "" });
      } else if (node.type === "SHAPE_WITH_TEXT") {
        nodes.push({ ...base, text: (node as any).text?.characters ?? "", shapeType: (node as any).shapeType });
      } else if (node.type === "TEXT") {
        nodes.push({ ...base, text: (node as any).characters ?? "" });
      } else if (node.type === "CODE_BLOCK") {
        nodes.push({ ...base, codeLanguage: (node as any).codeLanguage, code: (node as any).code });
      } else if (node.type === "TABLE") {
        nodes.push({ ...base, rows: (node as any).rowCount, columns: (node as any).columnCount });
      } else if (node.type === "CONNECTOR") {
        if (includeConnections) {
          const start = (node as any).connectorStart?.endpointNodeId;
          const end = (node as any).connectorEnd?.endpointNodeId;
          connections.push({
            id: node.id,
            from: start ?? null,
            to: end ?? null,
            text: (node as any).text?.characters ?? "",
          });
        }
      } else {
        nodes.push(base);
      }

      if ("children" in node) {
        walk((node as any).children);
      }
    }
  };

  walk(figma.currentPage.children);
  return {
    type: request.type,
    requestId: request.requestId,
    data: { nodes, connections, total: nodes.length + connections.length },
  };
}
