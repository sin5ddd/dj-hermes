# Third-party notices

## This project

dj-hermes source code is dual-licensed **MIT OR Apache-2.0**.
See [`LICENSE-MIT`](./LICENSE-MIT) and [`LICENSE-APACHE`](./LICENSE-APACHE).

Bundled WAV files under `samples/` are **CC0 1.0**. See [`samples/LICENSE.md`](./samples/LICENSE.md).

GitHub Releases do **not** bundle Whisper, Piper, or Hermes.

## Direct Rust dependencies (SPDX)

| Crate | License |
| --- | --- |
| `cpal` | Apache-2.0 |
| `crossbeam` | MIT OR Apache-2.0 |
| `crossterm` | MIT |
| `rustyline` | MIT |
| `axum` | MIT |
| `tokio` | MIT |
| `serde` | MIT OR Apache-2.0 |
| `serde_json` | MIT OR Apache-2.0 |
| `async-stream` | MIT |
| `tokio-stream` | MIT |
| `reqwest` | MIT OR Apache-2.0 |
| `midir` | MIT |

## TLS and selected transitives

`reqwest` is built with `rustls-tls` (no OpenSSL, no aws-lc).

| Crate | License | Notes |
| --- | --- | --- |
| `ring` 0.17.14 | Apache-2.0 AND ISC | rustls crypto |
| `webpki-roots` 1.0.9 | CDLA-Permissive-2.0 | TLS roots |
| `matchit` | MIT AND BSD-3-Clause | axum router |

## System libraries (Linux)

On Linux, `cpal` dynamically links **libasound** (ALSA), which is LGPL-2.1.
The library is not vendored in this repository.
