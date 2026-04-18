use std::process::Command;

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PdfBlock {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    Image { data: String, ext: String, width: u32, height: u32 },
}

#[derive(Debug, Deserialize)]
pub struct PdfPage {
    pub blocks: Vec<PdfBlock>,
}

#[derive(Debug, Deserialize)]
pub struct PdfContent {
    pub pages: Vec<PdfPage>,
}

#[derive(Debug, Deserialize)]
pub struct PdfMeta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub page_count: Option<u32>,
}

/// Resolve python binary: `RL_PYTHON` env var, then `python3`.
fn python_bin() -> String {
    std::env::var("RL_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

/// Resolve script path: `RL_PDF_SCRIPT` → `<exe_dir>/scripts/pdf_extract.py`
/// → `./server/scripts/pdf_extract.py`.
fn script_path() -> String {
    if let Ok(p) = std::env::var("RL_PDF_SCRIPT") {
        return p;
    }
    if let Ok(exe) = std::env::current_exe() {
        let candidate = exe.parent().unwrap_or(std::path::Path::new("."))
            .join("scripts/pdf_extract.py");
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    // Dev fallback: run from repo root via `cargo run`.
    "server/scripts/pdf_extract.py".to_string()
}

fn run(args: &[&str]) -> anyhow::Result<Vec<u8>> {
    let out = Command::new(python_bin())
        .args(args)
        .output()
        .context("failed to spawn python3")?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("pdf_extract.py failed: {stderr}");
    }
    Ok(out.stdout)
}

/// Synchronous — callers must wrap in `tokio::task::spawn_blocking`.
pub fn extract_content(file_path: &str) -> anyhow::Result<PdfContent> {
    let script = script_path();
    let stdout = run(&[&script, "--mode=content", file_path])?;
    serde_json::from_slice(&stdout).context("invalid JSON from pdf_extract.py --mode=content")
}

/// Synchronous — callers must wrap in `tokio::task::spawn_blocking`.
pub fn extract_meta(file_path: &str) -> anyhow::Result<PdfMeta> {
    let script = script_path();
    let stdout = run(&[&script, "--mode=meta", file_path])?;
    serde_json::from_slice(&stdout).context("invalid JSON from pdf_extract.py --mode=meta")
}

/// Non-fatal startup probe. Returns `Err` if python3 or PyMuPDF is missing.
pub fn self_test() -> anyhow::Result<()> {
    let script = script_path();
    let out = Command::new(python_bin())
        .args([&script, "--selftest"])
        .output()
        .context("failed to spawn python3")?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("{stderr}")
    }
}
