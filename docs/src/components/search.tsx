"use client";

import { useRouter } from "fumadocs-core/framework";
import { useFetchSearch } from "fumadocs-core/search/client";
import type { SearchResultRecord } from "fumadocs-core/search/client";
import type { SharedProps } from "fumadocs-ui/components/dialog/search";
import { type Components, toJsxRuntime } from "hast-util-to-jsx-runtime";
import type { ReactNode } from "react";
import { Fragment, jsx, jsxs } from "react/jsx-runtime";

import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";

function Children({ children }: { children?: ReactNode }) {
  return children;
}

function Cell({ children }: { children?: ReactNode }) {
  return <span className="min-w-0 not-last:truncate last:line-clamp-2">{children}</span>;
}

const cells: Partial<Components> = {
  table: Children,
  thead: Children,
  tbody: Children,
  tr: Children,
  th: Cell,
  td: Cell,
};

function render(hast: SearchResultRecord["hastContent"], components?: Partial<Components>) {
  return toJsxRuntime(hast, { Fragment, jsx, jsxs, components });
}

export function SearchDialog({ open, onOpenChange }: SharedProps) {
  const router = useRouter();
  const { search, onSearchChange, isLoading, data } = useFetchSearch();

  const onSelect = (url: string) => {
    // oxlint-disable-next-line no-void
    void router.push(url);
    onOpenChange(false);
  };

  const item = (record: SearchResultRecord, components?: Partial<Components>) => (
    <CommandItem
      key={record.id}
      value={record.id}
      onSelect={() => onSelect(record.url)}
      className={components && "grid grid-flow-col auto-cols-fr gap-3"}
    >
      {render(record.hastContent, components)}
    </CommandItem>
  );

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
    >
      <Command shouldFilter={false}>
        <CommandInput
          value={search}
          onValueChange={onSearchChange}
          placeholder="Search docs..."
        />
        <CommandList>
          <CommandEmpty>
            {isLoading ? "Searching..." : data ? "No results." : "Type to search."}
          </CommandEmpty>
          {data?.items.map((entry) =>
            entry.type === "table" ? (
              <CommandGroup
                key={entry.id}
                heading={
                  entry.header && (
                    <div className="grid grid-flow-col auto-cols-fr gap-3">
                      {render(entry.header, cells)}
                    </div>
                  )
                }
              >
                {entry.rows.map((row) =>
                  item(row, entry.header ? { ...cells, thead: () => null } : cells),
                )}
              </CommandGroup>
            ) : (
              item(entry)
            ),
          )}
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
