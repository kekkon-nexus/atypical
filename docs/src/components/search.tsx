"use client";

import { useRouter } from "fumadocs-core/framework";
import {
  useFetchSearch,
  type SearchResultItem,
  type SearchResultRecord,
} from "fumadocs-core/search/client";
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
  CommandSeparator,
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

const row = "grid auto-cols-fr grid-flow-col gap-3";

function render(
  hast: SearchResultRecord["hastContent"],
  components?: Partial<Components>,
): ReactNode {
  // hast-util-to-jsx-runtime types its result as the global `JSX.Element`, which React 19 removed
  return toJsxRuntime(hast, { Fragment, jsx, jsxs, components }) as ReactNode;
}

interface Group {
  page?: SearchResultRecord;
  matches: SearchResultItem[];
}

function toGroups(items: SearchResultItem[], query: string) {
  const groups: Group[] = [];
  for (const item of items) {
    const last = groups.at(-1);
    if (item.type === "page" || !last) {
      groups.push(item.type === "page" ? { page: item, matches: [] } : { matches: [item] });
    } else {
      last.matches.push(item);
    }
  }
  const term = query.toLowerCase();
  const inTitle = (group: Group) =>
    Number(group.page?.content.toLowerCase().includes(term) ?? false);
  return groups.toSorted((a, b) => inTitle(b) - inTitle(a));
}

export function SearchDialog({ open, onOpenChange }: SharedProps) {
  const router = useRouter();
  const { search, onSearchChange, isLoading, data } = useFetchSearch();

  const onSelect = (url: string) => {
    // oxlint-disable-next-line no-void
    void router.push(url);
    onOpenChange(false);
  };

  const item = (
    record: SearchResultRecord,
    className?: string,
    components?: Partial<Components>,
  ) => (
    <CommandItem
      key={record.id}
      value={record.id}
      onSelect={() => onSelect(record.url)}
      className={className}
    >
      {render(record.hastContent, components)}
    </CommandItem>
  );

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      className="top-1/2 -translate-y-1/2 sm:max-w-2xl"
    >
      <Command shouldFilter={false}>
        <CommandInput
          value={search}
          onValueChange={onSearchChange}
          placeholder="Search docs..."
        />
        <CommandList className="max-h-[50vh]">
          <CommandEmpty>
            {isLoading ? "Searching..." : data ? "No results." : "Type to search."}
          </CommandEmpty>
          {data &&
            toGroups(data.items, data.query).map((group, i) => (
              <Fragment key={group.page?.id ?? i}>
                {i > 0 && <CommandSeparator alwaysRender />}
                <CommandGroup>
                  {group.page && item(group.page, "font-semibold")}
                  {group.matches.map((match) =>
                    match.type === "table" ? (
                      <Fragment key={match.id}>
                        {match.header && (
                          <div className={`${row} px-2 py-1.5 text-xs text-muted-foreground`}>
                            {render(match.header, cells)}
                          </div>
                        )}
                        {match.rows.map((r) =>
                          item(
                            r,
                            `${row} *:[svg]:hidden`,
                            match.header ? { ...cells, thead: () => undefined } : cells,
                          ),
                        )}
                      </Fragment>
                    ) : (
                      item(match)
                    ),
                  )}
                </CommandGroup>
              </Fragment>
            ))}
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
