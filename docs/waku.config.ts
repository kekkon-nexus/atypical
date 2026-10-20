import path from "node:path";

import { cloudflare } from "@cloudflare/vite-plugin";
import tailwindcss from "@tailwindcss/vite";
import { fumadocsMdx } from "fumadocs-mdx/vite";
import { defineConfig } from "waku/config";

import { wgslToGlsl } from "./plugins/wgsl-to-glsl";

export default defineConfig({
  vite: {
    environments: {
      rsc: {
        optimizeDeps: { include: ["hono/tiny"] },
        build: { rolldownOptions: { platform: "neutral" } },
      },
      ssr: {
        optimizeDeps: { include: ["waku > rsc-html-stream/server"] },
        build: { rolldownOptions: { platform: "neutral" } },
      },
    },
    optimizeDeps: {
      exclude: ["@base-ui/react"],
    },
    resolve: {
      tsconfigPaths: true,
      dedupe: ["waku"],
      alias: { "lucide-react": path.resolve(import.meta.dirname, "src/lib/lucide-shim.ts") },
    },

    plugins: [
      fumadocsMdx(),
      tailwindcss(),
      wgslToGlsl(),
      cloudflare({
        viteEnvironment: { name: "rsc", childEnvironments: ["ssr"] },
        inspectorPort: false,
      }),
    ],
  },
});
