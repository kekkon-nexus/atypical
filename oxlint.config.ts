import oxlint from "@kekkon-nexus/config/oxlint";
import { defineConfig } from "oxlint";

export default defineConfig({
  extends: [oxlint],
  env: {
    node: true,
  },
  options: {
    typeAware: true,
    typeCheck: true,
  },
});
