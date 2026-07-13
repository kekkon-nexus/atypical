import base from "@kekkon-nexus/config/oxlint";
import { defineConfig } from "oxlint";

const config: ReturnType<typeof defineConfig> = defineConfig({
  ...base,
});

export default config;
