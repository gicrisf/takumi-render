use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use takumi_render::{
    DEFAULT_HEIGHT, DEFAULT_WIDTH, load_fonts, read_manifest, render_file, render_html, write_png,
};

#[derive(Debug, Parser)]
#[command(
    name = "takumi-render",
    version,
    about = "Render HTML and CSS to PNG with Takumi"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Render one HTML file to PNG.
    Render(RenderArgs),
    /// Render a JSON manifest of HTML files to PNG.
    Batch(BatchArgs),
}

#[derive(Debug, Args)]
struct RenderArgs {
    /// HTML file to read, or - for standard input.
    #[arg(long, value_name = "FILE", default_value = "-")]
    html: String,

    /// PNG file to write.
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,

    /// Image width in pixels.
    #[arg(long, default_value_t = DEFAULT_WIDTH)]
    width: u32,

    /// Image height in pixels.
    #[arg(long, default_value_t = DEFAULT_HEIGHT)]
    height: u32,

    /// Font directory. Repeat for multiple directories.
    #[arg(long = "fonts-dir", value_name = "DIR")]
    fonts_dirs: Vec<PathBuf>,
}

#[derive(Debug, Args)]
struct BatchArgs {
    /// JSON manifest of cards to render.
    manifest: PathBuf,

    /// Additional font directory. Repeat for multiple directories.
    #[arg(long = "fonts-dir", value_name = "DIR")]
    fonts_dirs: Vec<PathBuf>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("takumi-render: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Render(args) => render(args),
        Command::Batch(args) => batch(args),
    }
}

fn render(args: RenderArgs) -> Result<()> {
    let fonts = load_fonts(&args.fonts_dirs)?;
    let html = if args.html == "-" {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .context("read HTML from standard input")?;
        source
    } else {
        fs::read_to_string(&args.html).with_context(|| format!("read HTML {}", args.html))?
    };

    let png = render_html(&html, args.width, args.height, &fonts)?;
    write_png(&args.output, &png)
}

fn batch(args: BatchArgs) -> Result<()> {
    let manifest = read_manifest(&args.manifest)?;
    let mut directories = args.fonts_dirs;
    directories.extend(manifest.fonts);
    let fonts = load_fonts(&directories)?;

    for card in manifest.cards {
        render_file(
            &card.input,
            &card.output,
            card.width.unwrap_or(DEFAULT_WIDTH),
            card.height.unwrap_or(DEFAULT_HEIGHT),
            &fonts,
        )?;
    }

    Ok(())
}
