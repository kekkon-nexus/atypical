import tailwindcss from "@tailwindcss/vite";
import { fumadocsMdx } from "fumadocs-mdx/vite";
import { defineConfig } from "waku/config";

export default defineConfig({
  vite: {
    resolve: {
      tsconfigPaths: true,
      dedupe: ["waku"],
    },

    plugins: [tailwindcss(), fumadocsMdx()],
  },
});
