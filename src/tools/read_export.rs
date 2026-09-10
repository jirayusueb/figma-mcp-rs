// MANIFEST: get_screenshot | GetScreenshotArgs | Export a screenshot of one or more nodes as base64-encoded image data (held in memory). Use save_screenshots instead when you want to write images directly to disk without base64 in the response.
// MANIFEST: export_frames_to_pdf | @server | Export multiple frames as a single multi-page PDF file. Each frame becomes one page in order. Ideal for pitch decks, proposals, and slide exports.
// MANIFEST: save_screenshots | @server | Export screenshots for multiple nodes and write them to the local filesystem. Returns file metadata (path, size, dimensions) — no base64 in the response. Use get_screenshot instead when you need the image data in memory.

//! Ported from figma-mcp-go internal/tools_read_export.go and the
//! save_screenshots/export_frames_to_pdf server-side helpers in internal/tools.go.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::McpError;
use base64::Engine;
use futures_util::StreamExt;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::node::Node;

// ── get_screenshot (plugin passthrough) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetScreenshotArgs {
    /// Optional node IDs to export, colon format. If empty, exports current selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_ids: Option<Vec<String>>,
    /// Export format: PNG (default), SVG, JPG, or PDF
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Export scale for raster formats (default 2)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

pub(crate) async fn get_screenshot(
    node: Arc<Node>,
    args: GetScreenshotArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "get_screenshot", &args).await
}

// ── export_frames_to_pdf (server-side merge) ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportFramesToPdfArgs {
    /// Ordered list of frame node IDs to export as PDF pages, colon format e.g. '4029:12345'
    pub node_ids: Vec<String>,
    /// File path to write the PDF to, must end in .pdf (relative to working directory or absolute)
    pub output_path: String,
}

pub(crate) async fn export_frames_to_pdf(
    node: Arc<Node>,
    args: ExportFramesToPdfArgs,
) -> Result<CallToolResult, McpError> {
    if args.output_path.is_empty() {
        return Ok(error_result("outputPath is required"));
    }

    let work_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => return Ok(error_result(format!("getwd: {e}"))),
    };
    let resolved_path = match resolve_output_path(&args.output_path, &work_dir) {
        Ok(p) => p,
        Err(e) => return Ok(error_result(e)),
    };
    let ext_is_pdf = resolved_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false);
    if !ext_is_pdf {
        return Ok(error_result("outputPath must have a .pdf extension"));
    }

    let resp = match node
        .send("export_frames_to_pdf", args.node_ids.clone(), Value::Null)
        .await
    {
        Ok(r) => r,
        Err(e) => return Ok(error_result(e)),
    };
    if !resp.error_text().is_empty() {
        return Ok(error_result(resp.error_text().to_string()));
    }

    let pages = match extract_frame_pdfs(resp.data.unwrap_or(Value::Null)) {
        Ok(p) => p,
        Err(e) => return Ok(error_result(e)),
    };

    let merged = match merge_pdf_pages(&pages) {
        Ok(m) => m,
        Err(e) => return Ok(error_result(format!("merge PDFs: {e}"))),
    };

    if let Some(parent) = resolved_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Ok(error_result(format!("mkdir: {e}")));
        }
    }
    if resolved_path.exists() {
        return Ok(error_result(format!(
            "file already exists: {}",
            resolved_path.display()
        )));
    }
    if let Err(e) = std::fs::write(&resolved_path, &merged) {
        return Ok(error_result(format!("write file: {e}")));
    }

    let out = serde_json::json!({
        "outputPath": resolved_path.display().to_string(),
        "bytesWritten": merged.len(),
        "pageCount": pages.len(),
        "success": true,
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        out.to_string(),
    )]))
}

/// Parses the plugin response `{frames:[{base64:...},...]}` and returns raw PDF bytes per frame.
fn extract_frame_pdfs(data: Value) -> Result<Vec<Vec<u8>>, String> {
    #[derive(Debug, Default, Deserialize)]
    #[serde(default)]
    struct Frame {
        base64: String,
    }
    #[derive(Debug, Default, Deserialize)]
    #[serde(default)]
    struct Wrapper {
        frames: Vec<Frame>,
    }

    let wrapper: Wrapper = serde_json::from_value(data).map_err(|e| e.to_string())?;
    if wrapper.frames.is_empty() {
        return Err("no PDF frames returned by plugin".to_string());
    }
    let mut pages = Vec::with_capacity(wrapper.frames.len());
    for (i, f) in wrapper.frames.iter().enumerate() {
        if f.base64.is_empty() {
            return Err(format!("frame {i} has empty base64"));
        }
        let raw = base64::engine::general_purpose::STANDARD
            .decode(&f.base64)
            .map_err(|e| format!("frame {i}: base64 decode: {e}"))?;
        pages.push(raw);
    }
    Ok(pages)
}

/// Merges one or more single-page PDFs (each `pages[i]` a valid PDF byte slice) into one
/// multi-page PDF, in order. Ported from lopdf's `examples/merge.rs`, minus bookmarks/ToC
/// (the reference Go implementation via pdfcpu doesn't add one either).
fn merge_pdf_pages(pages: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    use std::collections::BTreeMap;

    use lopdf::{Document, Object, ObjectId};

    if pages.is_empty() {
        return Err("no pages to merge".to_string());
    }

    let mut max_id = 1u32;
    let mut documents_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut documents_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut document = Document::with_version("1.5");

    for page_bytes in pages {
        let mut doc = Document::load_mem(page_bytes).map_err(|e| e.to_string())?;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        for object_id in doc.get_pages().into_values() {
            let obj = doc
                .get_object(object_id)
                .map_err(|e| e.to_string())?
                .to_owned();
            documents_pages.insert(object_id, obj);
        }
        documents_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects.into_iter() {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                let id = catalog_object.map(|(id, _)| id).unwrap_or(object_id);
                catalog_object = Some((id, object));
            }
            b"Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, old_object)) = &pages_object {
                        if let Ok(old_dictionary) = old_object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }
                    let id = pages_object.map(|(id, _)| id).unwrap_or(object_id);
                    pages_object = Some((id, Object::Dictionary(dictionary)));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {} // Page handled below; outlines unsupported.
            _ => {
                document.objects.insert(object_id, object);
            }
        }
    }

    let (pages_id, pages_obj) = pages_object.ok_or("no Pages root found in exported PDFs")?;
    let (catalog_id, catalog_obj) =
        catalog_object.ok_or("no Catalog root found in exported PDFs")?;

    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_id);
            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = pages_obj.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set(
            "Kids",
            documents_pages
                .keys()
                .map(|id| Object::Reference(*id))
                .collect::<Vec<_>>(),
        );
        document
            .objects
            .insert(pages_id, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_obj.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_id);
        dictionary.remove(b"Outlines");
        document
            .objects
            .insert(catalog_id, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_id);
    document.max_id = document.objects.len() as u32;
    document.renumber_objects();

    let mut buf = Vec::new();
    document.save_to(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

// ── save_screenshots (server-side export + write) ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveScreenshotItemArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// File path to write the image to
    pub output_path: String,
    /// Export format: PNG, SVG, JPG, or PDF
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Export scale for raster formats
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SaveScreenshotsArgs {
    /// List of {nodeId, outputPath, format?, scale?} objects
    pub items: Vec<SaveScreenshotItemArgs>,
    /// Default export format: PNG (default), SVG, JPG, or PDF
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Default export scale for raster formats (default 2)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveResult {
    index: usize,
    node_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    node_name: String,
    output_path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    format: String,
    #[serde(skip_serializing_if = "is_zero_f64")]
    width: f64,
    #[serde(skip_serializing_if = "is_zero_f64")]
    height: f64,
    #[serde(skip_serializing_if = "is_zero_usize")]
    bytes_written: usize,
    success: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    error: String,
}

fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

fn is_zero_usize(v: &usize) -> bool {
    *v == 0
}

pub(crate) async fn save_screenshots(
    node: Arc<Node>,
    args: SaveScreenshotsArgs,
) -> Result<CallToolResult, McpError> {
    let work_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => return Ok(error_result(format!("getwd: {e}"))),
    };

    // ponytail: fixed ceiling — each in-flight item holds a full base64 image in memory
    const SAVE_CONCURRENCY: usize = 8;

    let pending: Vec<_> = args
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            save_screenshot_item(
                &node,
                item,
                index,
                &work_dir,
                args.format.as_deref(),
                args.scale,
            )
        })
        .collect();
    let results: Vec<SaveResult> = futures_util::stream::iter(pending)
        .buffered(SAVE_CONCURRENCY)
        .collect()
        .await;

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.len() - succeeded;

    let out = serde_json::json!({
        "total": results.len(),
        "succeeded": succeeded,
        "failed": failed,
        "hasErrors": failed > 0,
        "results": results,
    });
    Ok(CallToolResult::success(vec![ContentBlock::text(
        out.to_string(),
    )]))
}

async fn save_screenshot_item(
    node: &Node,
    item: &SaveScreenshotItemArgs,
    index: usize,
    work_dir: &Path,
    default_format: Option<&str>,
    default_scale: Option<f64>,
) -> SaveResult {
    let resolved_path = match resolve_output_path(&item.output_path, work_dir) {
        Ok(p) => p,
        Err(e) => {
            return SaveResult {
                index,
                node_id: item.node_id.clone(),
                output_path: item.output_path.clone(),
                error: e,
                ..Default::default()
            };
        }
    };
    let resolved_str = resolved_path.display().to_string();

    let mut format = coalesce(item.format.as_deref(), default_format)
        .unwrap_or("")
        .to_string();
    let inferred_format = infer_format(&resolved_path);
    if format.is_empty() {
        format = inferred_format.clone();
    }
    if format.is_empty() {
        format = "PNG".to_string();
    }
    if !inferred_format.is_empty() && format != inferred_format {
        return SaveResult {
            index,
            node_id: item.node_id.clone(),
            output_path: resolved_str,
            error: format!("format {format} conflicts with file extension {inferred_format}"),
            ..Default::default()
        };
    }

    let scale = item
        .scale
        .filter(|s| *s > 0.0)
        .or_else(|| default_scale.filter(|s| *s > 0.0));

    let mut params = serde_json::json!({ "format": format });
    if let Some(s) = scale {
        params["scale"] = serde_json::json!(s);
    }

    let resp = match node
        .send("get_screenshot", vec![item.node_id.clone()], params)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return SaveResult {
                index,
                node_id: item.node_id.clone(),
                output_path: resolved_str,
                error: e,
                ..Default::default()
            };
        }
    };
    if !resp.error_text().is_empty() {
        return SaveResult {
            index,
            node_id: item.node_id.clone(),
            output_path: resolved_str,
            error: resp.error_text().to_string(),
            ..Default::default()
        };
    }

    let export = match extract_screenshot_export(resp.data.unwrap_or(Value::Null)) {
        Ok(e) => e,
        Err(e) => {
            return SaveResult {
                index,
                node_id: item.node_id.clone(),
                output_path: resolved_str,
                error: e,
                ..Default::default()
            };
        }
    };

    let bytes_written = match write_base64(&export.base64, &resolved_path) {
        Ok(n) => n,
        Err(e) => {
            return SaveResult {
                index,
                node_id: item.node_id.clone(),
                output_path: resolved_str,
                error: e,
                ..Default::default()
            };
        }
    };

    SaveResult {
        index,
        node_id: export.node_id,
        node_name: export.node_name,
        output_path: resolved_str,
        format,
        width: export.width,
        height: export.height,
        bytes_written,
        success: true,
        error: String::new(),
    }
}

fn coalesce<'a>(a: Option<&'a str>, b: Option<&'a str>) -> Option<&'a str> {
    match a {
        Some(s) if !s.is_empty() => Some(s),
        _ => b,
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct ScreenshotExport {
    node_id: String,
    node_name: String,
    base64: String,
    width: f64,
    height: f64,
}

/// Parses the plugin response `{exports:[{nodeId,nodeName,base64,width,height},...]}`.
fn extract_screenshot_export(data: Value) -> Result<ScreenshotExport, String> {
    #[derive(Debug, Default, Deserialize)]
    #[serde(default)]
    struct Wrapper {
        exports: Vec<ScreenshotExport>,
    }
    let wrapper: Wrapper = serde_json::from_value(data).map_err(|e| e.to_string())?;
    wrapper
        .exports
        .into_iter()
        .next()
        .ok_or_else(|| "no screenshot export returned by plugin".to_string())
}

/// Base64-decodes `b64` and writes it to `output_path`, refusing to overwrite an existing file
/// (mirrors Go's `os.O_EXCL`).
fn write_base64(b64: &str, output_path: &Path) -> Result<usize, String> {
    let data = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| format!("base64 decode: {e}"))?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)
    {
        Ok(mut f) => {
            use std::io::Write;
            f.write_all(&data).map_err(|e| e.to_string())?;
            Ok(data.len())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
            "file already exists at outputPath: {}",
            output_path.display()
        )),
        Err(e) => Err(e.to_string()),
    }
}

fn infer_format(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("png") => "PNG".to_string(),
        Some("svg") => "SVG".to_string(),
        Some("jpg") | Some("jpeg") => "JPG".to_string(),
        Some("pdf") => "PDF".to_string(),
        _ => String::new(),
    }
}

// ── shared path helpers ──────────────────────────────────────────────────────

/// Resolves `output_path` (relative or absolute) against `work_dir` and confines it inside
/// `work_dir` — mirrors Go's `resolveOutputPath`/`mustBeInsideDir`.
fn resolve_output_path(output_path: &str, work_dir: &Path) -> Result<PathBuf, String> {
    let candidate = Path::new(output_path);
    let joined = if candidate.is_absolute() {
        lexical_clean(candidate)
    } else {
        lexical_clean(&work_dir.join(candidate))
    };
    must_be_inside_dir(&joined, work_dir)
}

fn must_be_inside_dir(resolved: &Path, work_dir: &Path) -> Result<PathBuf, String> {
    let work_clean = lexical_clean(work_dir);
    if resolved.strip_prefix(&work_clean).is_ok() {
        Ok(resolved.to_path_buf())
    } else {
        Err(format!(
            "outputPath must be inside the working directory: {}",
            work_dir.display()
        ))
    }
}

/// Lexically normalizes `.`/`..` components without touching the filesystem, like Go's
/// `filepath.Clean`. `..` past the root/prefix is dropped rather than erroring.
fn lexical_clean(path: &Path) -> PathBuf {
    use std::path::Component::*;
    let mut out: Vec<std::path::Component> = Vec::new();
    for comp in path.components() {
        match comp {
            CurDir => {}
            ParentDir => match out.last() {
                Some(Normal(_)) => {
                    out.pop();
                }
                Some(RootDir) | Some(Prefix(_)) => {}
                _ => out.push(comp),
            },
            other => out.push(other),
        }
    }
    out.into_iter().collect()
}

fn error_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(msg.into())])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_output_path_accepts_relative_path_inside_work_dir() {
        let work_dir = Path::new("/tmp/work");
        let resolved = resolve_output_path("out/a.png", work_dir).unwrap();
        assert_eq!(resolved, Path::new("/tmp/work/out/a.png"));
    }

    #[test]
    fn resolve_output_path_rejects_parent_dir_escape() {
        let work_dir = Path::new("/tmp/work");
        assert!(resolve_output_path("../x.png", work_dir).is_err());
    }

    #[test]
    fn resolve_output_path_rejects_absolute_path_outside_work_dir() {
        let work_dir = Path::new("/tmp/work");
        assert!(resolve_output_path("/etc/passwd", work_dir).is_err());
    }

    #[test]
    fn resolve_output_path_accepts_absolute_path_inside_work_dir() {
        let work_dir = Path::new("/tmp/work");
        let resolved = resolve_output_path("/tmp/work/sub/a.png", work_dir).unwrap();
        assert_eq!(resolved, Path::new("/tmp/work/sub/a.png"));
    }

    #[test]
    fn infer_format_maps_known_extensions() {
        assert_eq!(infer_format(Path::new("a.png")), "PNG");
        assert_eq!(infer_format(Path::new("a.SVG")), "SVG");
        assert_eq!(infer_format(Path::new("a.jpg")), "JPG");
        assert_eq!(infer_format(Path::new("a.jpeg")), "JPG");
        assert_eq!(infer_format(Path::new("a.pdf")), "PDF");
        assert_eq!(infer_format(Path::new("a.txt")), "");
        assert_eq!(infer_format(Path::new("a")), "");
    }

    /// Builds a minimal single-page PDF (Catalog -> Pages -> Page) for merge testing.
    fn build_test_pdf() -> Vec<u8> {
        use lopdf::{dictionary, Document, Object};

        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);

        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();
        buf
    }

    #[test]
    fn merge_pdf_pages_combines_page_count() {
        let pages = vec![build_test_pdf(), build_test_pdf()];
        let merged = merge_pdf_pages(&pages).unwrap();
        let doc = lopdf::Document::load_mem(&merged).unwrap();
        assert_eq!(doc.get_pages().len(), 2);
    }

    #[test]
    fn merge_pdf_pages_rejects_empty_input() {
        assert!(merge_pdf_pages(&[]).is_err());
    }
}
