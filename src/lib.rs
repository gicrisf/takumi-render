//! Render HTML and CSS to PNG with Takumi.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use takumi::prelude::{
    FontResource, Fonts, FromHtmlOptions, OutputFormat, RenderOptions, Viewport,
};
use takumi::{from_html, render, write_image};
use walkdir::WalkDir;

/// Default social-preview width in pixels.
pub const DEFAULT_WIDTH: u32 = 1200;

/// Default social-preview height in pixels.
pub const DEFAULT_HEIGHT: u32 = 630;

/// A batch of HTML files to render.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    /// Additional font directories, resolved from the current working directory.
    #[serde(default)]
    pub fonts: Vec<PathBuf>,
    /// Cards to render in order.
    pub cards: Vec<Card>,
}

/// One HTML input and PNG output pair.
#[derive(Debug, Deserialize)]
pub struct Card {
    /// HTML file to read.
    pub input: PathBuf,
    /// PNG file to write.
    pub output: PathBuf,
    /// Optional width. Defaults to [`DEFAULT_WIDTH`].
    #[serde(default)]
    pub width: Option<u32>,
    /// Optional height. Defaults to [`DEFAULT_HEIGHT`].
    #[serde(default)]
    pub height: Option<u32>,
}

/// Load a JSON batch manifest.
pub fn read_manifest(path: &Path) -> Result<Manifest> {
    let source =
        fs::read_to_string(path).with_context(|| format!("read manifest {}", path.display()))?;
    serde_json::from_str(&source).with_context(|| format!("parse manifest {}", path.display()))
}

/// Register every supported font file in the given directories.
///
/// Directories are read recursively in path order. At least one font file is
/// required because Takumi does not load system fonts.
pub fn load_fonts(directories: &[PathBuf]) -> Result<Fonts> {
    if directories.is_empty() {
        bail!("provide at least one --fonts-dir");
    }

    let mut fonts = Fonts::default();
    let mut registered = 0usize;

    for directory in directories {
        for entry in WalkDir::new(directory).sort_by_file_name() {
            let entry = entry.with_context(|| format!("walk {}", directory.display()))?;
            if !entry.file_type().is_file() || !is_font(entry.path()) {
                continue;
            }

            let path = entry.path();
            let bytes = fs::read(path).with_context(|| format!("read font {}", path.display()))?;
            fonts
                .register(FontResource::new(bytes))
                .with_context(|| format!("register font {}", path.display()))?;
            registered += 1;
        }
    }

    if registered == 0 {
        bail!("no .ttf, .otf, .ttc, .woff, or .woff2 fonts found");
    }

    Ok(fonts)
}

/// Render one HTML string to PNG bytes.
pub fn render_html(html: &str, width: u32, height: u32, fonts: &Fonts) -> Result<Vec<u8>> {
    let node = from_html(html, FromHtmlOptions::default()).context("parse HTML")?;
    let options = RenderOptions::builder()
        .viewport(Viewport::new((width, height)))
        .node(node)
        .fonts(fonts)
        .build();
    let image = render(options).context("render HTML")?;
    let mut png = Vec::new();
    write_image(&image, &mut png, OutputFormat::Png).context("encode PNG")?;
    Ok(png)
}

/// Read an HTML file, render it, and write a PNG.
pub fn render_file(
    input: &Path,
    output: &Path,
    width: u32,
    height: u32,
    fonts: &Fonts,
) -> Result<()> {
    let html =
        fs::read_to_string(input).with_context(|| format!("read HTML {}", input.display()))?;
    let png = render_html(&html, width, height, fonts)
        .with_context(|| format!("render {}", input.display()))?;
    write_png(output, &png)
}

/// Write PNG bytes, creating parent directories as needed.
pub fn write_png(output: &Path, png: &[u8]) -> Result<()> {
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }

    fs::write(output, png).with_context(|| format!("write {}", output.display()))
}

fn is_font(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("ttf" | "otf" | "ttc" | "woff" | "woff2")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_png_without_text() {
        let fonts = Fonts::default();
        let png = render_html(
            r#"<div style="width:100%;height:100%;background:#ff0000"></div>"#,
            10,
            10,
            &fonts,
        )
        .unwrap();

        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 10);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 10);
    }

    #[test]
    fn parses_a_manifest() {
        let manifest: Manifest = serde_json::from_str(
            r#"{
                "fonts": ["fonts"],
                "cards": [
                    {"input": "in.html", "output": "out.png"},
                    {"input": "wide.html", "output": "wide.png", "width": 800, "height": 400}
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(manifest.fonts, [PathBuf::from("fonts")]);
        assert_eq!(manifest.cards.len(), 2);
        assert_eq!(manifest.cards[0].width, None);
        assert_eq!(manifest.cards[1].width, Some(800));
    }

    #[test]
    fn rejects_an_empty_font_directory() {
        let directory = tempfile::tempdir().unwrap();
        let error = load_fonts(&[directory.path().to_path_buf()]).err().unwrap();
        assert!(error.to_string().contains("no .ttf"));
    }

    #[test]
    fn requires_a_font_directory() {
        let error = load_fonts(&[]).err().unwrap();
        assert!(error.to_string().contains("--fonts-dir"));
    }
}
