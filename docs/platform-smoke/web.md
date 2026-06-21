# Web Platform Smoke Artifact

Status: `validated-bounded` for the shared showcase in a local browser smoke.

The Web path builds the shared showcase for `wasm32-unknown-unknown`, generates a
`wasm-bindgen` package, serves it from loopback, opens it in Chromium, captures
visible screenshots, checks resize behavior, and records browser/runtime logs.
This proves the bounded showcase path, not every browser, GPU, WebGPU/WebGL
adapter, hosting setup, or CSP policy.

## Reproducible Checks

```bash
cargo check --manifest-path platform/web/Cargo.toml
cargo build --manifest-path platform/web/Cargo.toml --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir /tmp/egui-expressive-web/pkg \
  <target>/wasm32-unknown-unknown/release/egui_expressive_web_showcase.wasm
python3 -m http.server 8765 --bind 127.0.0.1 --directory /tmp/egui-expressive-web/site
chromium --headless --disable-gpu --screenshot=web-showcase.png http://127.0.0.1:8765/
```

## Results

| Check | Result | Notes |
| --- | --- | --- |
| Web crate check | passed | `platform/web/Cargo.toml` compiles locally. |
| wasm release build | passed | `wasm32-unknown-unknown` release artifact builds. |
| wasm-bindgen output | passed | JS/WASM package generated for web loading. |
| Browser launch | passed | Chromium loopback smoke reached a running DOM status. |
| Visible render | passed | 1280x800 and 640x480 screenshots were non-black with non-zero variance. |
| Resize path | passed | Smaller viewport smoke remained visible and running. |
| Browser/app logs | passed | No app panic/runtime failure markers were found in the captured stderr/log slice. |

## Boundaries

- Chromium loopback proof does not certify every browser, WebView, GPU, WebGPU or
  WebGL adapter, hosting provider, CSP, service worker, or mobile browser.
- Web support claims must name the tested browser/runtime path and remain scoped
  to the committed Web harness until broader browser-matrix evidence is added.
