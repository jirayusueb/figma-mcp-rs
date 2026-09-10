//! Node-ID normalization and leader-side RPC validation.
//! Ported from figma-mcp-go internal/schema.go.
use std::sync::LazyLock;
use regex::Regex;
use serde_json::{Map, Value};

static NODE_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^I?\d+:\d+(;\d+:\d+)*$").expect("valid regex"));

/// Normalize a node ID from hyphen format (4029-12345) to colon format (4029:12345).
pub fn normalize_node_id(id: &str) -> String {
    id.replace('-', ":")
}

/// Check that a node ID is in colon format (possibly compound, optionally instance-prefixed).
pub fn valid_node_id(id: &str) -> bool {
    NODE_ID_RE.is_match(id)
}

fn str_param<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(Value::as_str)
}

fn f64_param(params: &Value, key: &str) -> Option<f64> {
    params.get(key).and_then(Value::as_f64)
}

fn has_param(params: &Value, key: &str) -> bool {
    params.get(key).is_some()
}

fn valid_export_format(f: &str) -> bool {
    matches!(f, "PNG" | "SVG" | "JPG" | "PDF")
}

fn valid_trigger_type(t: &str) -> bool {
    matches!(
        t,
        "ON_CLICK"
            | "ON_HOVER"
            | "ON_PRESS"
            | "ON_DRAG"
            | "AFTER_TIMEOUT"
            | "MOUSE_ENTER"
            | "MOUSE_LEAVE"
            | "MOUSE_UP"
            | "MOUSE_DOWN"
    )
}

fn valid_action_type(t: &str) -> bool {
    matches!(
        t,
        "NODE"
            | "BACK"
            | "CLOSE"
            | "URL"
            | "CONDITIONAL"
            | "SET_VARIABLE"
            | "SET_VARIABLE_MODE"
            | "UPDATE_MEDIA_RUNTIME"
    )
}

fn valid_blend_mode(m: &str) -> bool {
    matches!(
        m,
        "NORMAL"
            | "MULTIPLY"
            | "SCREEN"
            | "OVERLAY"
            | "DARKEN"
            | "LIGHTEN"
            | "COLOR_DODGE"
            | "COLOR_BURN"
            | "HARD_LIGHT"
            | "SOFT_LIGHT"
            | "DIFFERENCE"
            | "EXCLUSION"
            | "HUE"
            | "SATURATION"
            | "COLOR"
            | "LUMINOSITY"
            | "PASS_THROUGH"
    )
}

fn validate_trigger_type_field(idx: usize, trigger: &Map<String, Value>) -> Option<String> {
    let t = trigger.get("type").and_then(Value::as_str).unwrap_or("");
    if !t.is_empty() && !valid_trigger_type(t) {
        return Some(format!("reactions[{idx}].trigger.type is invalid: {t}"));
    }
    if t == "AFTER_TIMEOUT" && trigger.get("timeout").and_then(Value::as_f64).is_none() {
        return Some(format!(
            "reactions[{idx}].trigger.timeout is required for AFTER_TIMEOUT and must be a number (milliseconds)"
        ));
    }
    None
}

fn validate_action_type_field(idx: usize, action: &Map<String, Value>) -> Option<String> {
    let t = action.get("type").and_then(Value::as_str).unwrap_or("");
    if !t.is_empty() && !valid_action_type(t) {
        return Some(format!("reactions[{idx}].action.type is invalid: {t}"));
    }
    match t {
        "NODE" => {
            let nav = action.get("navigation").and_then(Value::as_str).unwrap_or("");
            if nav.is_empty() {
                return Some(format!(
                    "reactions[{idx}].action.navigation is required for NODE (e.g. NAVIGATE, OVERLAY, SCROLL_TO, SWAP, CHANGE_TO)"
                ));
            }
        }
        "URL" => {
            let url = action.get("url").and_then(Value::as_str).unwrap_or("");
            if url.is_empty() {
                return Some(format!("reactions[{idx}].action.url is required for URL"));
            }
        }
        _ => {}
    }
    None
}

fn validate_reaction(idx: usize, r: &Map<String, Value>) -> Option<String> {
    if let Some(trigger) = r.get("trigger").and_then(Value::as_object) {
        if let Some(msg) = validate_trigger_type_field(idx, trigger) {
            return Some(msg);
        }
    }
    if let Some(action) = r.get("action").and_then(Value::as_object) {
        if let Some(msg) = validate_action_type_field(idx, action) {
            return Some(msg);
        }
    }
    None
}

fn validate_auto_layout_params(params: &Value) -> Option<String> {
    if let Some(lm) = str_param(params, "layoutMode") {
        if !lm.is_empty() && !matches!(lm, "HORIZONTAL" | "VERTICAL" | "NONE") {
            return Some(format!("layoutMode must be HORIZONTAL, VERTICAL, or NONE, got: {lm}"));
        }
    }
    if let Some(v) = str_param(params, "primaryAxisAlignItems") {
        if !v.is_empty() && !matches!(v, "MIN" | "CENTER" | "MAX" | "SPACE_BETWEEN") {
            return Some(format!(
                "primaryAxisAlignItems must be MIN, CENTER, MAX, or SPACE_BETWEEN, got: {v}"
            ));
        }
    }
    if let Some(v) = str_param(params, "counterAxisAlignItems") {
        if !v.is_empty() && !matches!(v, "MIN" | "CENTER" | "MAX" | "BASELINE") {
            return Some(format!(
                "counterAxisAlignItems must be MIN, CENTER, MAX, or BASELINE, got: {v}"
            ));
        }
    }
    if let Some(v) = str_param(params, "primaryAxisSizingMode") {
        if !v.is_empty() && !matches!(v, "FIXED" | "AUTO") {
            return Some(format!("primaryAxisSizingMode must be FIXED or AUTO, got: {v}"));
        }
    }
    if let Some(v) = str_param(params, "counterAxisSizingMode") {
        if !v.is_empty() && !matches!(v, "FIXED" | "AUTO") {
            return Some(format!("counterAxisSizingMode must be FIXED or AUTO, got: {v}"));
        }
    }
    if let Some(v) = str_param(params, "layoutWrap") {
        if !v.is_empty() && !matches!(v, "NO_WRAP" | "WRAP") {
            return Some(format!("layoutWrap must be NO_WRAP or WRAP, got: {v}"));
        }
    }
    None
}

/// Validate an incoming follower RPC before it reaches the plugin.
/// Returns Some(error message) on validation failure.
pub fn validate_rpc(tool: &str, node_ids: &[String], params: &Value) -> Option<String> {
    match tool {
        "get_node" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
        }

        "get_nodes_info" | "export_frames_to_pdf" | "ungroup_nodes" | "delete_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required and must not be empty".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
        }

        "get_screenshot" => {
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            if let Some(fmt) = str_param(params, "format") {
                if !valid_export_format(fmt) {
                    return Some(format!("format must be PNG, SVG, JPG, or PDF, got: {fmt}"));
                }
            }
        }

        "save_screenshots" => {
            let items = match params.get("items") {
                None => return Some("items is required".into()),
                Some(v) => v,
            };
            let item_list = match items.as_array() {
                Some(a) if !a.is_empty() => a,
                _ => return Some("items must be a non-empty array".into()),
            };
            for (i, item) in item_list.iter().enumerate() {
                let m = match item.as_object() {
                    Some(m) => m,
                    None => return Some(format!("items[{i}] must be an object")),
                };
                let node_id = m.get("nodeId").and_then(Value::as_str).unwrap_or("");
                if !valid_node_id(node_id) {
                    return Some(format!(
                        "items[{i}].nodeId must use colon format e.g. 4029:12345"
                    ));
                }
                let output_path = m.get("outputPath").and_then(Value::as_str).unwrap_or("");
                if output_path.is_empty() {
                    return Some(format!("items[{i}].outputPath is required"));
                }
            }
        }

        "get_design_context" => {
            if let Some(depth) = f64_param(params, "depth") {
                if depth < 0.0 {
                    return Some("depth must be a non-negative number".into());
                }
            }
            if let Some(detail) = str_param(params, "detail") {
                if !detail.is_empty() && !matches!(detail, "minimal" | "compact" | "full") {
                    return Some(format!("detail must be minimal, compact, or full, got: {detail}"));
                }
            }
        }

        "search_nodes" => {
            let query = str_param(params, "query").unwrap_or("");
            if query.is_empty() {
                return Some("query is required".into());
            }
            if let Some(node_id) = str_param(params, "nodeId") {
                if !node_id.is_empty() && !valid_node_id(node_id) {
                    return Some(format!(
                        "nodeId must use colon format e.g. 4029:12345, got: {node_id}"
                    ));
                }
            }
            if let Some(limit) = f64_param(params, "limit") {
                if limit <= 0.0 {
                    return Some("limit must be a positive number".into());
                }
            }
        }

        "get_reactions" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
        }

        "scan_text_nodes" | "scan_nodes_by_types" => {
            let node_id = str_param(params, "nodeId").unwrap_or("");
            if node_id.is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(node_id) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {node_id}"
                ));
            }
            if tool == "scan_nodes_by_types" {
                let types_ok = params
                    .get("types")
                    .and_then(Value::as_array)
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if !types_ok {
                    return Some("types must be a non-empty array".into());
                }
            }
        }

        "set_opacity" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            let opacity = match f64_param(params, "opacity") {
                Some(o) => o,
                None => return Some("opacity is required".into()),
            };
            if !(0.0..=1.0).contains(&opacity) {
                return Some("opacity must be between 0 and 1".into());
            }
        }

        "set_corner_radius" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            let has_any = has_param(params, "cornerRadius")
                || has_param(params, "topLeftRadius")
                || has_param(params, "topRightRadius")
                || has_param(params, "bottomLeftRadius")
                || has_param(params, "bottomRightRadius");
            if !has_any {
                return Some(
                    "at least one of cornerRadius, topLeftRadius, topRightRadius, bottomLeftRadius, or bottomRightRadius is required"
                        .into(),
                );
            }
        }

        "group_nodes" => {
            if node_ids.len() < 2 {
                return Some("nodeIds must contain at least 2 nodes to group".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
        }

        "navigate_to_page" => {
            let page_id = str_param(params, "pageId").unwrap_or("");
            let page_name = str_param(params, "pageName").unwrap_or("");
            if page_id.is_empty() && page_name.is_empty() {
                return Some("pageId or pageName is required".into());
            }
        }

        "create_component" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
        }

        "export_tokens" => {
            if let Some(fmt) = str_param(params, "format") {
                if !fmt.is_empty() && !matches!(fmt, "json" | "css") {
                    return Some(format!("format must be json or css, got: {fmt}"));
                }
            }
        }

        "create_frame" => {
            if let Some(w) = f64_param(params, "width") {
                if w <= 0.0 {
                    return Some("width must be positive".into());
                }
            }
            if let Some(h) = f64_param(params, "height") {
                if h <= 0.0 {
                    return Some("height must be positive".into());
                }
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {pid}"
                    ));
                }
            }
            if let Some(msg) = validate_auto_layout_params(params) {
                return Some(msg);
            }
        }

        "set_auto_layout" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            if let Some(msg) = validate_auto_layout_params(params) {
                return Some(msg);
            }
        }

        "create_rectangle" | "create_ellipse" => {
            if let Some(w) = f64_param(params, "width") {
                if w <= 0.0 {
                    return Some("width must be positive".into());
                }
            }
            if let Some(h) = f64_param(params, "height") {
                if h <= 0.0 {
                    return Some("height must be positive".into());
                }
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {pid}"
                    ));
                }
            }
        }

        "create_text" => {
            let text = str_param(params, "text").unwrap_or("");
            if text.is_empty() {
                return Some("text is required".into());
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {pid}"
                    ));
                }
            }
        }

        "set_text" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            if str_param(params, "text").is_none() {
                return Some("text is required".into());
            }
        }

        "set_fills" | "set_strokes" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            let color = str_param(params, "color").unwrap_or("");
            if color.is_empty() {
                return Some("color is required (hex string e.g. #FF5733)".into());
            }
            if let Some(mode) = str_param(params, "mode") {
                if mode != "replace" && mode != "append" {
                    return Some("mode must be 'replace' or 'append'".into());
                }
            }
        }

        "move_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            if !has_param(params, "x") && !has_param(params, "y") {
                return Some("at least one of x or y is required".into());
            }
        }

        "resize_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            if !has_param(params, "width") && !has_param(params, "height") {
                return Some("at least one of width or height is required".into());
            }
        }

        "rename_node" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            let name = str_param(params, "name").unwrap_or("");
            if name.is_empty() {
                return Some("name is required".into());
            }
        }

        "clone_node" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {pid}"
                    ));
                }
            }
        }

        "import_image" => {
            let image_data = str_param(params, "imageData").unwrap_or("");
            if image_data.is_empty() {
                return Some("imageData (base64) is required".into());
            }
            if let Some(sm) = str_param(params, "scaleMode") {
                if !sm.is_empty() && !matches!(sm, "FILL" | "FIT" | "CROP" | "TILE") {
                    return Some(format!("scaleMode must be FILL, FIT, CROP, or TILE, got: {sm}"));
                }
            }
            if let Some(pid) = str_param(params, "parentId") {
                if !pid.is_empty() && !valid_node_id(pid) {
                    return Some(format!(
                        "parentId must use colon format e.g. 4029:12345, got: {pid}"
                    ));
                }
            }
        }

        "create_paint_style" => {
            let name = str_param(params, "name").unwrap_or("");
            if name.is_empty() {
                return Some("name is required".into());
            }
            let color = str_param(params, "color").unwrap_or("");
            if color.is_empty() {
                return Some("color is required (hex string e.g. #FF5733)".into());
            }
        }

        "create_text_style" => {
            let name = str_param(params, "name").unwrap_or("");
            if name.is_empty() {
                return Some("name is required".into());
            }
            if let Some(td) = str_param(params, "textDecoration") {
                if !td.is_empty() && !matches!(td, "NONE" | "UNDERLINE" | "STRIKETHROUGH") {
                    return Some(format!(
                        "textDecoration must be NONE, UNDERLINE, or STRIKETHROUGH, got: {td}"
                    ));
                }
            }
            if let Some(unit) = str_param(params, "lineHeightUnit") {
                if !unit.is_empty() && !matches!(unit, "PIXELS" | "PERCENT") {
                    return Some(format!("lineHeightUnit must be PIXELS or PERCENT, got: {unit}"));
                }
            }
            if let Some(unit) = str_param(params, "letterSpacingUnit") {
                if !unit.is_empty() && !matches!(unit, "PIXELS" | "PERCENT") {
                    return Some(format!(
                        "letterSpacingUnit must be PIXELS or PERCENT, got: {unit}"
                    ));
                }
            }
        }

        "create_effect_style" => {
            let name = str_param(params, "name").unwrap_or("");
            if name.is_empty() {
                return Some("name is required".into());
            }
            if let Some(t) = str_param(params, "type") {
                if !t.is_empty()
                    && !matches!(t, "DROP_SHADOW" | "INNER_SHADOW" | "LAYER_BLUR" | "BACKGROUND_BLUR")
                {
                    return Some(format!(
                        "type must be DROP_SHADOW, INNER_SHADOW, LAYER_BLUR, or BACKGROUND_BLUR, got: {t}"
                    ));
                }
            }
        }

        "create_grid_style" => {
            let name = str_param(params, "name").unwrap_or("");
            if name.is_empty() {
                return Some("name is required".into());
            }
            if let Some(p) = str_param(params, "pattern") {
                if !p.is_empty() && !matches!(p, "GRID" | "COLUMNS" | "ROWS") {
                    return Some(format!("pattern must be GRID, COLUMNS, or ROWS, got: {p}"));
                }
            }
            if let Some(a) = str_param(params, "alignment") {
                if !a.is_empty() && !matches!(a, "STRETCH" | "CENTER" | "MIN" | "MAX") {
                    return Some(format!(
                        "alignment must be STRETCH, CENTER, MIN, or MAX, got: {a}"
                    ));
                }
            }
        }

        "update_paint_style" => {
            let style_id = str_param(params, "styleId").unwrap_or("");
            if style_id.is_empty() {
                return Some("styleId is required".into());
            }
            if !has_param(params, "name") && !has_param(params, "color") && !has_param(params, "description")
            {
                return Some("at least one of name, color, or description is required".into());
            }
        }

        "delete_style" => {
            let style_id = str_param(params, "styleId").unwrap_or("");
            if style_id.is_empty() {
                return Some("styleId is required".into());
            }
        }

        "create_variable_collection" => {
            let name = str_param(params, "name").unwrap_or("");
            if name.is_empty() {
                return Some("name is required".into());
            }
        }

        "add_variable_mode" => {
            let collection_id = str_param(params, "collectionId").unwrap_or("");
            if collection_id.is_empty() {
                return Some("collectionId is required".into());
            }
            let mode_name = str_param(params, "modeName").unwrap_or("");
            if mode_name.is_empty() {
                return Some("modeName is required".into());
            }
        }

        "create_variable" => {
            let name = str_param(params, "name").unwrap_or("");
            if name.is_empty() {
                return Some("name is required".into());
            }
            let collection_id = str_param(params, "collectionId").unwrap_or("");
            if collection_id.is_empty() {
                return Some("collectionId is required".into());
            }
            let var_type = str_param(params, "type").unwrap_or("");
            if !matches!(var_type, "COLOR" | "FLOAT" | "STRING" | "BOOLEAN") {
                return Some(format!(
                    "type must be COLOR, FLOAT, STRING, or BOOLEAN, got: {var_type}"
                ));
            }
        }

        "set_variable_value" => {
            let variable_id = str_param(params, "variableId").unwrap_or("");
            if variable_id.is_empty() {
                return Some("variableId is required".into());
            }
            let mode_id = str_param(params, "modeId").unwrap_or("");
            if mode_id.is_empty() {
                return Some("modeId is required".into());
            }
            if !has_param(params, "value") {
                return Some("value is required".into());
            }
        }

        "delete_variable" => {
            let vid = str_param(params, "variableId").unwrap_or("");
            let cid = str_param(params, "collectionId").unwrap_or("");
            if vid.is_empty() && cid.is_empty() {
                return Some("variableId or collectionId is required".into());
            }
        }

        "apply_style_to_node" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            let style_id = str_param(params, "styleId").unwrap_or("");
            if style_id.is_empty() {
                return Some("styleId is required".into());
            }
            if let Some(target) = str_param(params, "target") {
                if !target.is_empty() && !matches!(target, "fill" | "stroke") {
                    return Some(format!("target must be fill or stroke, got: {target}"));
                }
            }
        }

        "bind_variable_to_node" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            let variable_id = str_param(params, "variableId").unwrap_or("");
            if variable_id.is_empty() {
                return Some("variableId is required".into());
            }
            let field = str_param(params, "field").unwrap_or("");
            if field.is_empty() {
                return Some("field is required".into());
            }
        }

        "swap_component" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            let component_id = str_param(params, "componentId").unwrap_or("");
            if component_id.is_empty() {
                return Some("componentId is required".into());
            }
            if !valid_node_id(component_id) {
                return Some(format!(
                    "componentId must use colon format e.g. 4029:12345, got: {component_id}"
                ));
            }
        }

        "detach_instance" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required and must not be empty".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
        }

        "set_reactions" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            let raw_reactions = match params.get("reactions") {
                None => return Some("reactions is required".into()),
                Some(v) => v,
            };
            let reactions = match raw_reactions.as_array() {
                Some(a) => a,
                None => return Some("reactions must be an array".into()),
            };
            if let Some(mode) = str_param(params, "mode") {
                if !mode.is_empty() && mode != "replace" && mode != "append" {
                    return Some(format!("mode must be 'replace' or 'append', got: {mode}"));
                }
            }
            for (i, raw) in reactions.iter().enumerate() {
                let r = match raw.as_object() {
                    Some(r) => r,
                    None => return Some(format!("reactions[{i}] must be an object")),
                };
                if let Some(msg) = validate_reaction(i, r) {
                    return Some(msg);
                }
            }
        }

        "remove_reactions" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            if let Some(indices) = params.get("indices").and_then(Value::as_array) {
                for (i, v) in indices.iter().enumerate() {
                    if v.as_f64().is_none() {
                        return Some(format!("indices[{i}] must be a number"));
                    }
                }
            }
        }

        "set_visible" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            if params.get("visible").and_then(Value::as_bool).is_none() {
                return Some("visible (boolean) is required".into());
            }
        }

        "lock_nodes" | "unlock_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
        }

        "rotate_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            if f64_param(params, "rotation").is_none() {
                return Some("rotation (degrees) is required".into());
            }
        }

        "reorder_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            let order = str_param(params, "order").unwrap_or("");
            if !matches!(order, "bringToFront" | "sendToBack" | "bringForward" | "sendBackward") {
                return Some(format!(
                    "order must be bringToFront, sendToBack, bringForward, or sendBackward, got: {order}"
                ));
            }
        }

        "set_blend_mode" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            let blend_mode = str_param(params, "blendMode").unwrap_or("");
            if blend_mode.is_empty() {
                return Some("blendMode is required".into());
            }
            if !valid_blend_mode(blend_mode) {
                return Some(format!("blendMode {blend_mode:?} is not a valid Figma blend mode"));
            }
        }

        "set_constraints" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            if !has_param(params, "horizontal") && !has_param(params, "vertical") {
                return Some("at least one of horizontal or vertical is required".into());
            }
            if let Some(h) = str_param(params, "horizontal") {
                if !h.is_empty() && !matches!(h, "MIN" | "MAX" | "CENTER" | "STRETCH" | "SCALE") {
                    return Some(format!(
                        "horizontal must be MIN, MAX, CENTER, STRETCH, or SCALE, got: {h}"
                    ));
                }
            }
            if let Some(v) = str_param(params, "vertical") {
                if !v.is_empty() && !matches!(v, "MIN" | "MAX" | "CENTER" | "STRETCH" | "SCALE") {
                    return Some(format!(
                        "vertical must be MIN, MAX, CENTER, STRETCH, or SCALE, got: {v}"
                    ));
                }
            }
        }

        "reparent_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            let parent_id = str_param(params, "parentId").unwrap_or("");
            if parent_id.is_empty() {
                return Some("parentId is required".into());
            }
            if !valid_node_id(parent_id) {
                return Some(format!(
                    "parentId must use colon format e.g. 4029:12345, got: {parent_id}"
                ));
            }
        }

        "batch_rename_nodes" => {
            if node_ids.is_empty() {
                return Some("nodeIds is required".into());
            }
            for id in node_ids {
                if !valid_node_id(id) {
                    return Some(format!(
                        "invalid nodeId: {id} — must use colon format e.g. 4029:12345"
                    ));
                }
            }
            let has_find = has_param(params, "find");
            let has_replace = has_param(params, "replace");
            let has_prefix = has_param(params, "prefix");
            let has_suffix = has_param(params, "suffix");
            if !has_find && !has_replace && !has_prefix && !has_suffix {
                return Some("at least one of find/replace, prefix, or suffix is required".into());
            }
            if has_find && !has_replace {
                return Some("replace is required when find is provided".into());
            }
        }

        "find_replace_text" => {
            let find = str_param(params, "find").unwrap_or("");
            if find.is_empty() {
                return Some("find is required".into());
            }
            if !has_param(params, "replace") {
                return Some("replace is required".into());
            }
            if let Some(node_id) = str_param(params, "nodeId") {
                if !node_id.is_empty() && !valid_node_id(node_id) {
                    return Some(format!(
                        "nodeId must use colon format e.g. 4029:12345, got: {node_id}"
                    ));
                }
            }
            if !node_ids.is_empty() && !node_ids[0].is_empty() && !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
        }

        "add_page" => {
            if let Some(idx) = f64_param(params, "index") {
                if idx < 0.0 {
                    return Some("index must be non-negative".into());
                }
            }
        }

        "delete_page" | "rename_page" => {
            let page_id = str_param(params, "pageId").unwrap_or("");
            let page_name = str_param(params, "pageName").unwrap_or("");
            if page_id.is_empty() && page_name.is_empty() {
                return Some("pageId or pageName is required".into());
            }
            if tool == "rename_page" {
                let new_name = str_param(params, "newName").unwrap_or("");
                if new_name.is_empty() {
                    return Some("newName is required".into());
                }
            }
        }

        "set_effects" => {
            if node_ids.is_empty() || node_ids[0].is_empty() {
                return Some("nodeId is required".into());
            }
            if !valid_node_id(&node_ids[0]) {
                return Some(format!(
                    "nodeId must use colon format e.g. 4029:12345, got: {}",
                    node_ids[0]
                ));
            }
            let effects = match params.get("effects") {
                None => return Some("effects array is required".into()),
                Some(v) => v,
            };
            let effect_list = match effects.as_array() {
                Some(a) => a,
                None => return Some("effects must be an array".into()),
            };
            for (i, e) in effect_list.iter().enumerate() {
                let em = match e.as_object() {
                    Some(m) => m,
                    None => return Some(format!("effects[{i}] must be an object")),
                };
                let t = em.get("type").and_then(Value::as_str).unwrap_or("");
                if !matches!(t, "DROP_SHADOW" | "INNER_SHADOW" | "LAYER_BLUR" | "BACKGROUND_BLUR") {
                    return Some(format!(
                        "effects[{i}].type must be DROP_SHADOW, INNER_SHADOW, LAYER_BLUR, or BACKGROUND_BLUR, got: {t}"
                    ));
                }
            }
        }

        "create_section" => {
            if let Some(w) = f64_param(params, "width") {
                if w <= 0.0 {
                    return Some("width must be positive".into());
                }
            }
            if let Some(h) = f64_param(params, "height") {
                if h <= 0.0 {
                    return Some("height must be positive".into());
                }
            }
        }

        "execute_code" => {
            if str_param(params, "code").unwrap_or("").trim().is_empty() {
                return Some("code is required".into());
            }
            if let Some(t) = params.get("timeoutMs").and_then(|v| v.as_f64()) {
                if !(1.0..=25000.0).contains(&t) {
                    return Some(format!("timeoutMs must be between 1 and 25000, got: {t}"));
                }
            }
        }
        "create_stickies" => {
            if !params.get("items").map_or(false, |v| v.is_array() && v.as_array().map_or(false, |a| !a.is_empty())) {
                return Some("items must be a non-empty array".into());
            }
            if let Some(items) = params.get("items").and_then(|v| v.as_array()) {
                if items.len() > 200 {
                    return Some("items supports at most 200 stickies per call".into());
                }
            }
        }

        "create_table" => {
            let rows = params.get("rows").and_then(|v| v.as_f64());
            let cols = params.get("columns").and_then(|v| v.as_f64());
            if rows.is_none() || cols.is_none() {
                return Some("rows and columns are required".into());
            }
            if let (Some(r), Some(c)) = (rows, cols) {
                if !(1.0..=50.0).contains(&r) || !(1.0..=50.0).contains(&c) {
                    return Some("rows and columns must be between 1 and 50".into());
                }
            }
        }

        "create_code_block" => {
            if str_param(params, "code").unwrap_or("").is_empty() {
                return Some("code is required".into());
            }
        }

        "auto_arrange" => {
            if let Some(layout) = str_param(params, "layout") {
                if !["grid", "row", "column"].contains(&layout) {
                    return Some("layout must be one of: grid, row, column".into());
                }
            }
        }


        "use_figma" => {
            if str_param(params, "code").unwrap_or("").trim().is_empty() {
                return Some("code is required".into());
            }
        }

        _ => {}
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn normalizes_hyphens() {
        assert_eq!(normalize_node_id("4029-12345"), "4029:12345");
        assert_eq!(normalize_node_id("4029:12345"), "4029:12345");
        assert_eq!(normalize_node_id("I4029-12345"), "I4029:12345");
    }

    #[test]
    fn validates_node_ids() {
        assert!(valid_node_id("4029:12345"));
        assert!(valid_node_id("I4029:12345"));
        assert!(valid_node_id("4029:12345;4029:67890"));
        assert!(!valid_node_id("4029-12345"));
        assert!(!valid_node_id("abc"));
        assert!(!valid_node_id(""));
    }

    #[test]
    fn unknown_tool_passes_through() {
        assert_eq!(validate_rpc("unknown_tool", &[], &Value::Null), None);
    }

    #[test]
    fn get_node_requires_valid_id() {
        assert!(validate_rpc("get_node", &[], &Value::Null).is_some());
        assert!(validate_rpc("get_node", &ids(&["bad-id"]), &Value::Null).is_some());
        assert!(validate_rpc("get_node", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn get_nodes_info_requires_nonempty_valid_ids() {
        assert!(validate_rpc("get_nodes_info", &[], &Value::Null).is_some());
        assert!(validate_rpc("get_nodes_info", &ids(&["1:1", "bad"]), &Value::Null).is_some());
        assert!(validate_rpc("get_nodes_info", &ids(&["1:1", "2:2"]), &Value::Null).is_none());
        // shared branch with export_frames_to_pdf / ungroup_nodes / delete_nodes
        assert!(validate_rpc("export_frames_to_pdf", &[], &Value::Null).is_some());
        assert!(validate_rpc("ungroup_nodes", &[], &Value::Null).is_some());
        assert!(validate_rpc("delete_nodes", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn get_screenshot_validates_ids_and_format() {
        assert!(validate_rpc("get_screenshot", &ids(&["bad"]), &Value::Null).is_some());
        assert!(validate_rpc("get_screenshot", &ids(&["1:1"]), &json!({"format": "GIF"})).is_some());
        assert!(validate_rpc("get_screenshot", &ids(&["1:1"]), &json!({"format": "PNG"})).is_none());
    }

    #[test]
    fn save_screenshots_validates_items() {
        assert!(validate_rpc("save_screenshots", &[], &Value::Null).is_some());
        assert!(validate_rpc("save_screenshots", &[], &json!({"items": []})).is_some());
        assert!(validate_rpc(
            "save_screenshots",
            &[],
            &json!({"items": [{"nodeId": "bad", "outputPath": "a.png"}]})
        )
        .is_some());
        assert!(validate_rpc(
            "save_screenshots",
            &[],
            &json!({"items": [{"nodeId": "1:1", "outputPath": ""}]})
        )
        .is_some());
        assert!(validate_rpc(
            "save_screenshots",
            &[],
            &json!({"items": [{"nodeId": "1:1", "outputPath": "a.png"}]})
        )
        .is_none());
    }

    #[test]
    fn get_design_context_validates_depth_and_detail() {
        assert!(validate_rpc("get_design_context", &[], &json!({"depth": -1})).is_some());
        assert!(validate_rpc("get_design_context", &[], &json!({"detail": "bogus"})).is_some());
        assert!(validate_rpc("get_design_context", &[], &json!({"depth": 2, "detail": "full"})).is_none());
    }

    #[test]
    fn search_nodes_requires_query() {
        assert!(validate_rpc("search_nodes", &[], &Value::Null).is_some());
        assert!(validate_rpc("search_nodes", &[], &json!({"query": "x", "nodeId": "bad"})).is_some());
        assert!(validate_rpc("search_nodes", &[], &json!({"query": "x", "limit": 0})).is_some());
        assert!(validate_rpc("search_nodes", &[], &json!({"query": "x"})).is_none());
    }

    #[test]
    fn get_reactions_requires_valid_id() {
        assert!(validate_rpc("get_reactions", &[], &Value::Null).is_some());
        assert!(validate_rpc("get_reactions", &ids(&["bad-id"]), &Value::Null).is_some());
        assert!(validate_rpc("get_reactions", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn scan_text_nodes_and_by_types_require_node_id() {
        assert!(validate_rpc("scan_text_nodes", &[], &Value::Null).is_some());
        assert!(validate_rpc("scan_text_nodes", &[], &json!({"nodeId": "1:1"})).is_none());
        assert!(validate_rpc("scan_nodes_by_types", &[], &json!({"nodeId": "1:1"})).is_some());
        assert!(validate_rpc(
            "scan_nodes_by_types",
            &[],
            &json!({"nodeId": "1:1", "types": []})
        )
        .is_some());
        assert!(validate_rpc(
            "scan_nodes_by_types",
            &[],
            &json!({"nodeId": "1:1", "types": ["TEXT"]})
        )
        .is_none());
    }

    #[test]
    fn set_opacity_validates_range() {
        assert!(validate_rpc("set_opacity", &[], &Value::Null).is_some());
        assert!(validate_rpc("set_opacity", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_opacity", &ids(&["1:1"]), &json!({"opacity": 1.5})).is_some());
        assert!(validate_rpc("set_opacity", &ids(&["1:1"]), &json!({"opacity": 0.5})).is_none());
    }

    #[test]
    fn set_corner_radius_requires_one_field() {
        assert!(validate_rpc("set_corner_radius", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_corner_radius", &ids(&["1:1"]), &json!({"cornerRadius": 4})).is_none());
        assert!(validate_rpc("set_corner_radius", &ids(&["1:1"]), &json!({"topLeftRadius": 4})).is_none());
    }

    #[test]
    fn group_nodes_requires_two_or_more() {
        assert!(validate_rpc("group_nodes", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("group_nodes", &ids(&["1:1", "bad"]), &Value::Null).is_some());
        assert!(validate_rpc("group_nodes", &ids(&["1:1", "2:2"]), &Value::Null).is_none());
    }

    #[test]
    fn navigate_to_page_requires_id_or_name() {
        assert!(validate_rpc("navigate_to_page", &[], &Value::Null).is_some());
        assert!(validate_rpc("navigate_to_page", &[], &json!({"pageName": "Page 1"})).is_none());
    }

    #[test]
    fn create_component_requires_valid_id() {
        assert!(validate_rpc("create_component", &[], &Value::Null).is_some());
        assert!(validate_rpc("create_component", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn export_tokens_validates_format() {
        assert!(validate_rpc("export_tokens", &[], &json!({"format": "yaml"})).is_some());
        assert!(validate_rpc("export_tokens", &[], &json!({"format": "json"})).is_none());
        assert!(validate_rpc("export_tokens", &[], &Value::Null).is_none());
    }

    #[test]
    fn create_frame_validates_dims_parent_and_autolayout() {
        assert!(validate_rpc("create_frame", &[], &json!({"width": 0})).is_some());
        assert!(validate_rpc("create_frame", &[], &json!({"height": -1})).is_some());
        assert!(validate_rpc("create_frame", &[], &json!({"parentId": "bad"})).is_some());
        assert!(validate_rpc("create_frame", &[], &json!({"layoutMode": "SIDEWAYS"})).is_some());
        assert!(validate_rpc(
            "create_frame",
            &[],
            &json!({"width": 100, "height": 100, "layoutMode": "HORIZONTAL"})
        )
        .is_none());
    }

    #[test]
    fn set_auto_layout_requires_node_and_validates_params() {
        assert!(validate_rpc("set_auto_layout", &[], &Value::Null).is_some());
        assert!(validate_rpc(
            "set_auto_layout",
            &ids(&["1:1"]),
            &json!({"primaryAxisSizingMode": "BOGUS"})
        )
        .is_some());
        assert!(validate_rpc("set_auto_layout", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn create_rectangle_ellipse_validate_dims() {
        assert!(validate_rpc("create_rectangle", &[], &json!({"width": -1})).is_some());
        assert!(validate_rpc("create_ellipse", &[], &json!({"height": 0})).is_some());
        assert!(validate_rpc("create_rectangle", &[], &json!({"width": 10, "height": 10})).is_none());
    }

    #[test]
    fn create_text_requires_text() {
        assert!(validate_rpc("create_text", &[], &Value::Null).is_some());
        assert!(validate_rpc("create_text", &[], &json!({"text": "hi", "parentId": "bad"})).is_some());
        assert!(validate_rpc("create_text", &[], &json!({"text": "hi"})).is_none());
    }

    #[test]
    fn set_text_requires_node_and_text() {
        assert!(validate_rpc("set_text", &[], &Value::Null).is_some());
        assert!(validate_rpc("set_text", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_text", &ids(&["1:1"]), &json!({"text": ""})).is_none());
    }

    #[test]
    fn set_fills_and_strokes_require_color() {
        assert!(validate_rpc("set_fills", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_fills", &ids(&["1:1"]), &json!({"color": "#FFF", "mode": "bogus"})).is_some());
        assert!(validate_rpc("set_strokes", &ids(&["1:1"]), &json!({"color": "#FFF"})).is_none());
    }

    #[test]
    fn move_and_resize_require_one_axis() {
        assert!(validate_rpc("move_nodes", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("move_nodes", &ids(&["1:1"]), &json!({"x": 1})).is_none());
        assert!(validate_rpc("resize_nodes", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("resize_nodes", &ids(&["1:1"]), &json!({"height": 1})).is_none());
    }

    #[test]
    fn rename_node_requires_name() {
        assert!(validate_rpc("rename_node", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("rename_node", &ids(&["1:1"]), &json!({"name": "x"})).is_none());
    }

    #[test]
    fn clone_node_validates_parent() {
        assert!(validate_rpc("clone_node", &ids(&["1:1"]), &json!({"parentId": "bad"})).is_some());
        assert!(validate_rpc("clone_node", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn import_image_requires_data_and_validates_scale_mode() {
        assert!(validate_rpc("import_image", &[], &Value::Null).is_some());
        assert!(validate_rpc("import_image", &[], &json!({"imageData": "abc", "scaleMode": "BOGUS"})).is_some());
        assert!(validate_rpc("import_image", &[], &json!({"imageData": "abc"})).is_none());
    }

    #[test]
    fn style_creation_tools_require_name() {
        assert!(validate_rpc("create_paint_style", &[], &Value::Null).is_some());
        assert!(validate_rpc("create_paint_style", &[], &json!({"name": "x"})).is_some()); // missing color
        assert!(validate_rpc("create_paint_style", &[], &json!({"name": "x", "color": "#FFF"})).is_none());

        assert!(validate_rpc("create_text_style", &[], &json!({"name": "x", "textDecoration": "BOGUS"})).is_some());
        assert!(validate_rpc("create_text_style", &[], &json!({"name": "x"})).is_none());

        assert!(validate_rpc("create_effect_style", &[], &json!({"name": "x", "type": "BOGUS"})).is_some());
        assert!(validate_rpc("create_effect_style", &[], &json!({"name": "x"})).is_none());

        assert!(validate_rpc("create_grid_style", &[], &json!({"name": "x", "pattern": "BOGUS"})).is_some());
        assert!(validate_rpc("create_grid_style", &[], &json!({"name": "x"})).is_none());
    }

    #[test]
    fn update_and_delete_style() {
        assert!(validate_rpc("update_paint_style", &[], &Value::Null).is_some());
        assert!(validate_rpc("update_paint_style", &[], &json!({"styleId": "s1"})).is_some());
        assert!(validate_rpc("update_paint_style", &[], &json!({"styleId": "s1", "name": "x"})).is_none());
        assert!(validate_rpc("delete_style", &[], &Value::Null).is_some());
        assert!(validate_rpc("delete_style", &[], &json!({"styleId": "s1"})).is_none());
    }

    #[test]
    fn variable_tools_validate_required_fields() {
        assert!(validate_rpc("create_variable_collection", &[], &Value::Null).is_some());
        assert!(validate_rpc("create_variable_collection", &[], &json!({"name": "x"})).is_none());

        assert!(validate_rpc("add_variable_mode", &[], &json!({"collectionId": "c1"})).is_some());
        assert!(validate_rpc("add_variable_mode", &[], &json!({"collectionId": "c1", "modeName": "Dark"})).is_none());

        assert!(validate_rpc("create_variable", &[], &json!({"name": "x", "collectionId": "c1"})).is_some());
        assert!(validate_rpc(
            "create_variable",
            &[],
            &json!({"name": "x", "collectionId": "c1", "type": "COLOR"})
        )
        .is_none());

        assert!(validate_rpc("set_variable_value", &[], &json!({"variableId": "v1", "modeId": "m1"})).is_some());
        assert!(validate_rpc(
            "set_variable_value",
            &[],
            &json!({"variableId": "v1", "modeId": "m1", "value": 1})
        )
        .is_none());

        assert!(validate_rpc("delete_variable", &[], &Value::Null).is_some());
        assert!(validate_rpc("delete_variable", &[], &json!({"variableId": "v1"})).is_none());
    }

    #[test]
    fn linked_tools_validate_ids_and_fields() {
        assert!(validate_rpc("apply_style_to_node", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc(
            "apply_style_to_node",
            &ids(&["1:1"]),
            &json!({"styleId": "s1", "target": "bogus"})
        )
        .is_some());
        assert!(validate_rpc("apply_style_to_node", &ids(&["1:1"]), &json!({"styleId": "s1"})).is_none());

        assert!(validate_rpc("bind_variable_to_node", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc(
            "bind_variable_to_node",
            &ids(&["1:1"]),
            &json!({"variableId": "v1", "field": "fills"})
        )
        .is_none());

        assert!(validate_rpc("swap_component", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("swap_component", &ids(&["1:1"]), &json!({"componentId": "bad"})).is_some());
        assert!(validate_rpc("swap_component", &ids(&["1:1"]), &json!({"componentId": "2:2"})).is_none());

        assert!(validate_rpc("detach_instance", &[], &Value::Null).is_some());
        assert!(validate_rpc("detach_instance", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn set_reactions_validates_array_and_entries() {
        assert!(validate_rpc("set_reactions", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_reactions", &ids(&["1:1"]), &json!({"reactions": "nope"})).is_some());
        assert!(validate_rpc(
            "set_reactions",
            &ids(&["1:1"]),
            &json!({"reactions": [{"trigger": {"type": "BOGUS"}}]})
        )
        .is_some());
        assert!(validate_rpc(
            "set_reactions",
            &ids(&["1:1"]),
            &json!({"reactions": [{"trigger": {"type": "AFTER_TIMEOUT"}}]})
        )
        .is_some());
        assert!(validate_rpc(
            "set_reactions",
            &ids(&["1:1"]),
            &json!({"reactions": [{"action": {"type": "URL"}}]})
        )
        .is_some());
        assert!(validate_rpc(
            "set_reactions",
            &ids(&["1:1"]),
            &json!({"reactions": [{"action": {"type": "URL", "url": "https://x"}}]})
        )
        .is_none());
    }

    #[test]
    fn remove_reactions_validates_indices() {
        assert!(validate_rpc("remove_reactions", &ids(&["1:1"]), &json!({"indices": ["x"]})).is_some());
        assert!(validate_rpc("remove_reactions", &ids(&["1:1"]), &json!({"indices": [0, 1]})).is_none());
    }

    #[test]
    fn set_visible_requires_boolean() {
        assert!(validate_rpc("set_visible", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_visible", &ids(&["1:1"]), &json!({"visible": true})).is_none());
    }

    #[test]
    fn lock_unlock_nodes_validate_ids() {
        assert!(validate_rpc("lock_nodes", &[], &Value::Null).is_some());
        assert!(validate_rpc("unlock_nodes", &ids(&["1:1"]), &Value::Null).is_none());
    }

    #[test]
    fn rotate_nodes_requires_rotation() {
        assert!(validate_rpc("rotate_nodes", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("rotate_nodes", &ids(&["1:1"]), &json!({"rotation": 45})).is_none());
    }

    #[test]
    fn reorder_nodes_validates_order() {
        assert!(validate_rpc("reorder_nodes", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("reorder_nodes", &ids(&["1:1"]), &json!({"order": "bogus"})).is_some());
        assert!(validate_rpc("reorder_nodes", &ids(&["1:1"]), &json!({"order": "bringToFront"})).is_none());
    }

    #[test]
    fn set_blend_mode_validates_mode() {
        assert!(validate_rpc("set_blend_mode", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_blend_mode", &ids(&["1:1"]), &json!({"blendMode": "BOGUS"})).is_some());
        assert!(validate_rpc("set_blend_mode", &ids(&["1:1"]), &json!({"blendMode": "MULTIPLY"})).is_none());
    }

    #[test]
    fn set_constraints_requires_axis_and_validates_values() {
        assert!(validate_rpc("set_constraints", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc(
            "set_constraints",
            &ids(&["1:1"]),
            &json!({"horizontal": "BOGUS"})
        )
        .is_some());
        assert!(validate_rpc("set_constraints", &ids(&["1:1"]), &json!({"vertical": "CENTER"})).is_none());
    }

    #[test]
    fn reparent_nodes_requires_valid_parent() {
        assert!(validate_rpc("reparent_nodes", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("reparent_nodes", &ids(&["1:1"]), &json!({"parentId": "bad"})).is_some());
        assert!(validate_rpc("reparent_nodes", &ids(&["1:1"]), &json!({"parentId": "2:2"})).is_none());
    }

    #[test]
    fn batch_rename_nodes_requires_one_op_and_pairs_find_replace() {
        assert!(validate_rpc("batch_rename_nodes", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("batch_rename_nodes", &ids(&["1:1"]), &json!({"find": "a"})).is_some());
        assert!(validate_rpc(
            "batch_rename_nodes",
            &ids(&["1:1"]),
            &json!({"find": "a", "replace": "b"})
        )
        .is_none());
        assert!(validate_rpc("batch_rename_nodes", &ids(&["1:1"]), &json!({"prefix": "x"})).is_none());
    }

    #[test]
    fn find_replace_text_requires_find_and_replace() {
        assert!(validate_rpc("find_replace_text", &[], &Value::Null).is_some());
        assert!(validate_rpc("find_replace_text", &[], &json!({"find": "a"})).is_some());
        assert!(validate_rpc("find_replace_text", &[], &json!({"find": "a", "replace": "b", "nodeId": "bad"})).is_some());
        assert!(validate_rpc("find_replace_text", &ids(&["bad"]), &json!({"find": "a", "replace": "b"})).is_some());
        assert!(validate_rpc("find_replace_text", &[], &json!({"find": "a", "replace": "b"})).is_none());
    }

    #[test]
    fn page_management_tools() {
        assert!(validate_rpc("add_page", &[], &json!({"index": -1})).is_some());
        assert!(validate_rpc("add_page", &[], &json!({"index": 0})).is_none());

        assert!(validate_rpc("delete_page", &[], &Value::Null).is_some());
        assert!(validate_rpc("delete_page", &[], &json!({"pageId": "1:1"})).is_none());

        assert!(validate_rpc("rename_page", &[], &json!({"pageId": "1:1"})).is_some());
        assert!(validate_rpc("rename_page", &[], &json!({"pageId": "1:1", "newName": "New"})).is_none());
    }

    #[test]
    fn set_effects_validates_array_and_types() {
        assert!(validate_rpc("set_effects", &ids(&["1:1"]), &Value::Null).is_some());
        assert!(validate_rpc("set_effects", &ids(&["1:1"]), &json!({"effects": "nope"})).is_some());
        assert!(validate_rpc(
            "set_effects",
            &ids(&["1:1"]),
            &json!({"effects": [{"type": "BOGUS"}]})
        )
        .is_some());
        assert!(validate_rpc(
            "set_effects",
            &ids(&["1:1"]),
            &json!({"effects": [{"type": "DROP_SHADOW"}]})
        )
        .is_none());
    }

    #[test]
    fn create_section_validates_dims() {
        assert!(validate_rpc("create_section", &[], &json!({"width": 0})).is_some());
        assert!(validate_rpc("create_section", &[], &json!({"width": 10, "height": 10})).is_none());
    }

    #[test]
    fn use_figma_requires_code() {
        assert!(validate_rpc("use_figma", &[], &Value::Null).is_some());
        assert!(validate_rpc("use_figma", &[], &json!({"code": "   "})).is_some());
        assert!(validate_rpc("use_figma", &[], &json!({"code": "return 1"})).is_none());
    }
    #[test]
    fn execute_code_requires_code() {
        assert!(validate_rpc("execute_code", &[], &Value::Null).is_some());
        assert!(validate_rpc("execute_code", &[], &json!({"code": "   "})).is_some());
        assert!(validate_rpc("execute_code", &[], &json!({"code": "return 1"})).is_none());
    }

    #[test]
    fn execute_code_rejects_out_of_range_timeout() {
        assert!(validate_rpc("execute_code", &[], &json!({"code": "return 1", "timeoutMs": 0})).is_some());
        assert!(validate_rpc("execute_code", &[], &json!({"code": "return 1", "timeoutMs": 25001})).is_some());
        assert!(validate_rpc("execute_code", &[], &json!({"code": "return 1", "timeoutMs": 1})).is_none());
        assert!(validate_rpc("execute_code", &[], &json!({"code": "return 1", "timeoutMs": 25000})).is_none());
        assert!(validate_rpc("execute_code", &[], &json!({"code": "return 1", "timeoutMs": 5000})).is_none());
    }
}
