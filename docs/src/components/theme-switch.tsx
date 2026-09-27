"use client";

import { RiComputerLine, RiMoonLine, RiSunLine } from "@remixicon/react";
import type { ThemeSwitchProps } from "fumadocs-ui/layouts/shared/slots/theme-switch";
import { useTheme } from "next-themes";
import { useSyncExternalStore } from "react";
import { flushSync } from "react-dom";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";

const noop = () => () => {};

export function ThemeSwitch({ className, mode = "light-dark" }: ThemeSwitchProps) {
  const { setTheme, theme, resolvedTheme } = useTheme();
  const mounted = useSyncExternalStore(
    noop,
    () => true,
    () => false,
  );
  const current = mode === "light-dark" ? resolvedTheme : theme;

  const changeTheme = (value: string) => {
    document.startViewTransition(() => flushSync(() => setTheme(value)));
  };

  return (
    <ToggleGroup
      className={cn(className, "px-0.5")}
      data-theme-toggle=""
      value={mounted && current ? [current] : []}
      onValueChange={([value]) => value && changeTheme(value)}
      variant="outline"
      spacing={0}
    >
      <ToggleGroupItem
        value="light"
        aria-label="Light"
      >
        <RiSunLine />
      </ToggleGroupItem>
      <ToggleGroupItem
        value="dark"
        aria-label="Dark"
      >
        <RiMoonLine />
      </ToggleGroupItem>
      {mode === "light-dark-system" && (
        <ToggleGroupItem
          value="system"
          aria-label="System"
        >
          <RiComputerLine />
        </ToggleGroupItem>
      )}
    </ToggleGroup>
  );
}
