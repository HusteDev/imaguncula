# ADR 0001: Frontend stack

Status: accepted

Date: 2026-07-31

## Context

The desktop interface needs typed state, accessible semantic markup, a large output review table, and static assets that Tauri can bundle without a server.

## Decision

Use Svelte 5 with TypeScript in strict mode, SvelteKit's static adapter in SPA mode, Vite, and pnpm with a committed lockfile. SvelteKit supplies routing and static application structure; it does not add a runtime server.

## Consequences

The release application remains a static frontend. Tauri IPC must not be called during server rendering. Frontend dependency updates pass `svelte-check` and the production frontend build before adoption.

