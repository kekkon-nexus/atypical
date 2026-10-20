import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { searchTrigger } from "@/components/search-trigger";
import { ThemeSwitch } from "@/components/theme-switch";

import { appName, gitConfig } from "./shared";

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <span className="rounded-md bg-primary-foreground px-1.5 py-0.5 font-medium text-primary">
          {appName}
        </span>
      ),
    },
    slots: { searchTrigger, themeSwitch: ThemeSwitch },
    links: [{ text: "Documentation", url: "/docs", active: "nested-url" }],
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  };
}
