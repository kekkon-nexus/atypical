import oxfmt from "@kekkon-nexus/config/oxfmt";
import oxlint from "@kekkon-nexus/config/oxlint";
import jsdoc from "@kekkon-nexus/config/oxlint/base/jsdoc";
import vp from "@kekkon-nexus/config/oxlint/vite-plus";
import { defineConfig } from "vite-plus";

export default defineConfig({
  fmt: {
    ...oxfmt,
    ignorePatterns: ["*.toml", "benches/*.md"],
  },
  lint: {
    extends: [oxlint, vp],
    env: {
      node: true,
    },
    options: {
      typeAware: true,
      typeCheck: true,
    },
    overrides: [
      {
        files: ["*.js"],
        ...jsdoc,
      },
    ],
  },
  staged: {
    "*.rs": () => [
      "cargo fmt --all",
      "cargo clippy --workspace --all-targets --all-features --allow-staged --allow-dirty --fix -- -D warnings",
    ],
    "*.toml": ["tombi format", "tombi lint"],
    "*.{json,yml,yaml}": ["vp fmt", "v8r"],
    "*.{js,ts}": "vp check --fix",
  },
});
