# docs

The atypical documentation site, built with [Fumadocs](https://fumadocs.dev) on [Waku](https://waku.gg) and deployed to Cloudflare Workers.

Install from the repository root, since dependencies come from the workspace catalogs:

```bash
bun install
```

Then, from `docs/`:

| Script               | Does                                        |
| -------------------- | ------------------------------------------- |
| `bun run dev`        | Start the dev server                        |
| `bun run build`      | Build for Workers into `dist/`              |
| `bun run preview`    | Production build served locally by wrangler |
| `bun run deploy`     | Build and deploy with wrangler              |
| `bun run cf-typegen` | Generate types for the Worker bindings      |

Pages live in `content/docs`. The home page and benchmarks read `../benches/result-*.md` and the version reads `../Cargo.toml`, both bundled at build time, so a new bench snapshot or version bump needs a rebuild.
