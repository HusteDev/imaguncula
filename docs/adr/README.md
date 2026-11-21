# Architecture decision records

| ADR | Status | Decision |
| --- | --- | --- |
| [0001](0001-frontend-stack.md) | Accepted | Svelte 5, TypeScript, SvelteKit static SPA, Vite, pnpm |
| [0002](0002-rust-library-spikes.md) | Accepted; hosted CI pending | Rust image-processing libraries and spike evidence |
| [0003](0003-svg-intermediate.md) | Accepted | Retain one sanitized `usvg` tree |
| [0004](0004-new-project-defaults.md) | Accepted | Enable Windows desktop and Web/PWA by default |
| [0005](0005-combined-zip-layout.md) | Accepted | Use preset-ID directories in combined ZIPs |
| [0006](0006-initial-release-platforms.md) | Accepted | Publish Windows and Linux first; build unsigned macOS artifacts |
| [0007](0007-preset-delivery.md) | Accepted | Bundle presets with v1 releases |
| [0008](0008-icns-scope.md) | Accepted | Implement ICNS in Phase 4 |
| [0009](0009-project-license.md) | Accepted | Dual-license under MIT or Apache-2.0 |
| [0010](0010-png-color-profile-policy.md) | Accepted | Detect ICC profiles, treat decoded v1 channel values as sRGB, and tag generated PNGs as sRGB |
| [0011](0011-transactional-directory-export.md) | Accepted | Stage, verify, and atomically commit directory exports with recoverable overwrite |
