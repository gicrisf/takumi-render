# takumi-render

Native HTML/CSS to PNG rendering with [Takumi](https://takumi.kane.tw), without Node.js.

This is a small, independent command-line front end for Takumi's Rust renderer. It reads an HTML
document or a JSON batch manifest, registers the fonts you provide, and writes PNG files. It makes
no network requests and needs no Node.js, npm, or WebAssembly runtime.

## Why

Takumi publishes an excellent Rust engine and npm bindings, but not a general-purpose PNG CLI. This
project fills that gap for static-site builds, social-preview generation, and other workflows that
already have HTML and CSS.

## Install

### Prebuilt binary

Install the latest Linux x86_64 binary with the installer:

```sh
curl -fsSL https://raw.githubusercontent.com/gicrisf/takumi-render/main/install.sh | bash
```

This installs to `~/.local/bin`. Set `TAKUMI_RENDER_INSTALL_DIR` to choose another directory, or
`TAKUMI_RENDER_VERSION` to pin a release. Manual downloads are available from the
[releases page](https://github.com/gicrisf/takumi-render/releases).

### Cargo

```sh
cargo install --git https://github.com/gicrisf/takumi-render --locked
```

The build requires Rust 1.91 or newer.

## Usage

```sh
# One HTML file
takumi-render render \
  --html card.html \
  --output card.png \
  --width 1200 \
  --height 630 \
  --fonts-dir path/to/fonts

# HTML from standard input
cat card.html | takumi-render render --output card.png --fonts-dir path/to/fonts
```

`--fonts-dir` is repeatable and recursive. It accepts TTF, OTF, TTC, WOFF, and WOFF2 files. At least
one font is required because Takumi does not load system fonts.

Use data URLs in the HTML for local images, so the renderer never needs to fetch files:

```html
<img src="data:image/svg+xml;base64,...">
<div style="background-image:url('data:image/jpeg;base64,...')"></div>
```

## Batch mode

Batch mode loads the font set once and renders every card in order.

```sh
takumi-render batch cards.json --fonts-dir path/to/fonts
```

```json
{
  "fonts": ["path/to/more/fonts"],
  "cards": [
    {
      "input": "html/home.html",
      "output": "static/og/home.png",
      "width": 1200,
      "height": 630
    },
    {
      "input": "html/post.html",
      "output": "static/og/post.png"
    }
  ]
}
```

Paths are resolved from the current working directory. `width` and `height` default to 1200 and 630.

## Scope

This release renders one PNG per input. It does not include a template engine, fetch remote images,
generate social metadata, or know anything about a particular static site.

## Credits

This project exists because of [Takumi](https://github.com/kane50613/takumi). It is independent and
unofficial, and is not maintained by or affiliated with the Takumi project.

## License

MIT.
