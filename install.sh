#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'takumi-render installer: %s\n' "$*" >&2
  exit 1
}

case "$(uname -s)" in
  Linux) ;;
  *) fail "prebuilt binaries are only available for Linux x86_64; use Cargo" ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) ;;
  *) fail "prebuilt binaries are only available for Linux x86_64; use Cargo" ;;
esac

for tool in curl grep install mktemp sha256sum tar; do
  command -v "$tool" >/dev/null 2>&1 || fail "missing required command: $tool"
done

version="${TAKUMI_RENDER_VERSION:-}"
if [[ -z "$version" ]]; then
  latest="$(curl -fsSL https://api.github.com/repos/gicrisf/takumi-render/releases/latest)" ||
    fail "could not query the latest release; set TAKUMI_RENDER_VERSION to pin one"
  version="$(printf '%s\n' "$latest" | sed -n 's/.*"tag_name":[[:space:]]*"v\([^"]*\)".*/\1/p' | head -n 1)"
  [[ -n "$version" ]] || fail "could not determine the latest release; set TAKUMI_RENDER_VERSION to pin one"
fi
version="${version#v}"

archive="takumi-render-v${version}-x86_64-unknown-linux-musl.tar.gz"
base="https://github.com/gicrisf/takumi-render/releases/download/v${version}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

curl -fsSL "${base}/${archive}" -o "${tmp}/${archive}" ||
  fail "could not download ${archive}"
curl -fsSL "${base}/SHA256SUMS" -o "${tmp}/SHA256SUMS" ||
  fail "could not download SHA256SUMS"

(cd "$tmp" && grep "  ${archive}$" SHA256SUMS | sha256sum -c -) ||
  fail "checksum verification failed"

tar -xzf "${tmp}/${archive}" -C "$tmp"

bindir="${TAKUMI_RENDER_INSTALL_DIR:-${HOME}/.local/bin}"
mkdir -p "$bindir"
install -m 0755 "${tmp}/takumi-render-v${version}-x86_64-unknown-linux-musl/takumi-render" "${bindir}/takumi-render"

"${bindir}/takumi-render" --version

case ":${PATH}:" in
  *":${bindir}:"*) ;;
  *) printf 'Add %s to PATH to run takumi-render from any directory.\n' "$bindir" >&2 ;;
esac
