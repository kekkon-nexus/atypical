"use client";

import { type ReactNode, useState } from "react";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

/** Toggles between panels rendered on the server; hidden panels stay in the markup. */
export function Switcher({
  options,
  className,
}: {
  options: { value: string; label: ReactNode; panel: ReactNode }[];
  className?: string;
}) {
  const [active, setActive] = useState(options[0]?.value);

  return (
    <div className={className}>
      <ToggleGroup
        variant="outline"
        size="sm"
        value={active ? [active] : []}
        onValueChange={(value: string[]) => {
          if (value[0]) setActive(value[0]);
        }}
      >
        {options.map(({ value, label }) => (
          <ToggleGroupItem
            key={value}
            value={value}
          >
            {label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      {options.map(({ value, panel }) => (
        <div
          key={value}
          hidden={value !== active}
        >
          {panel}
        </div>
      ))}
    </div>
  );
}
