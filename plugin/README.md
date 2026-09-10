# figma-mcp-rs plugin

Figma/FigJam plugin UI (SolidJS) + plugin core, built with Vite and Bun 1.4.2.

## TypeScript 7 note

`package.json` pins `"typescript": "^7"` (TypeScript 7.0 GA, native Go compiler,
published on npm as the regular `typescript` package). The `typecheck` script
runs `bunx tsgo --noEmit`. If a future `typescript@^7` release renames or drops
the `tsgo` binary, switch the devDependency to `"@typescript/native-preview": "^7"`
and/or update the script to whatever binary that release ships (check
`node_modules/.bin` after `bun install`).

## Scripts

- `bun run dev` — watch-build both the UI (`vite.config.ts`) and plugin core
  (`vite.config.main.ts`) via `concurrently`.
- `bun run build` — one-shot build of both bundles into `dist/`.
- `bun run typecheck` — `tsgo --noEmit` over `src/`.
- `bun test` — unit tests for plugin-core handlers and serializers.
