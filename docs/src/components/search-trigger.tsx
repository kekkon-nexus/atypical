"use client";

import { RiSearchLine } from "@remixicon/react";
import { useSearchContext } from "fumadocs-ui/contexts/search";
import type {
  SearchTriggerProps,
  FullSearchTriggerProps,
} from "fumadocs-ui/layouts/shared/slots/search-trigger";

import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupText } from "@/components/ui/input-group";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { cn } from "@/lib/utils";

function SearchTrigger({ hideIfDisabled, variant, ...props }: SearchTriggerProps) {
  const { enabled, setOpenSearch } = useSearchContext();
  if (hideIfDisabled && !enabled) return;

  if (variant === "primary") {
    variant = "default";
  }

  return (
    <Button
      variant="ghost"
      size="icon-sm"
      data-search=""
      aria-label="Open Search"
      onClick={() => setOpenSearch(true)}
      {...props}
    >
      <RiSearchLine />
    </Button>
  );
}

function FullSearchTrigger({ hideIfDisabled, className, ...props }: FullSearchTriggerProps) {
  const { enabled, hotKey, setOpenSearch } = useSearchContext();
  if (hideIfDisabled && !enabled) return;

  return (
    <Button
      nativeButton={false}
      render={<InputGroup />}
      variant="outline"
      data-search-full=""
      onClick={() => setOpenSearch(true)}
      className={cn(className, "px-[unset]")}
      {...props}
    >
      <InputGroupText className="flex-auto">Search...</InputGroupText>
      <InputGroupAddon>
        <RiSearchLine />
      </InputGroupAddon>
      <InputGroupAddon align="inline-end">
        <KbdGroup>
          {hotKey.map((k, i) => (
            <Kbd key={i}>{k.display}</Kbd>
          ))}
        </KbdGroup>
      </InputGroupAddon>
    </Button>
  );
}

export const searchTrigger = { sm: SearchTrigger, full: FullSearchTrigger };
