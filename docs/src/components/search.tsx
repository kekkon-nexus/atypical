"use client";

import { useRouter } from "fumadocs-core/framework";
import { useDocsSearch } from "fumadocs-core/search/client";
import { fetchClient } from "fumadocs-core/search/client/fetch";
import type { SharedProps } from "fumadocs-ui/components/dialog/search";

import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";

export function SearchDialog({ open, onOpenChange }: SharedProps) {
  const router = useRouter();
  const { search, setSearch, query } = useDocsSearch({
    client: fetchClient(),
  });
  const items = query.data === "empty" ? [] : query.data;

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
    >
      <Command shouldFilter={false}>
        <CommandInput
          value={search}
          onValueChange={setSearch}
          placeholder="Search docs..."
        />
        <CommandList>
          <CommandEmpty>
            {query.isLoading
              ? "Searching..."
              : query.data === "empty"
                ? "Type to search."
                : "No results."}
          </CommandEmpty>
          {items.map((item) => (
            <CommandItem
              key={item.id}
              value={item.id}
              onSelect={() => {
                // oxlint-disable-next-line no-void
                void router.push(item.url);
                onOpenChange(false);
              }}
            >
              {item.content}
            </CommandItem>
          ))}
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
