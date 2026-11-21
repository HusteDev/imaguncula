## Scope

Describe the user-visible or architectural change and identify the execution-plan task or issue it implements.

## Evidence

- [ ] `just lint`
- [ ] `just test`
- [ ] `just build`
- [ ] Platform-specific or manual checks are documented below

List exact commands and results. Mark checks not run and explain why.

## Security and privacy

- [ ] Processing remains local-only
- [ ] Filesystem operations stay within user-selected or staged paths
- [ ] Untrusted image, preset, and project input fails closed
- [ ] Logs, fixtures, screenshots, and commits contain no secrets or confidential artwork
- [ ] New dependencies pass the repository dependency policy

## Product and documentation

- [ ] Output dimensions, names, package paths, and validation behavior have tests where changed
- [ ] Relevant user and architecture documentation is updated
- [ ] `PLAN.md`, `EXECUTION-PLAN.md`, and ADRs are updated when requirements or accepted decisions changed

## Visual evidence

Add screenshots for visible changes. Use synthetic artwork and include keyboard/focus behavior when relevant.
