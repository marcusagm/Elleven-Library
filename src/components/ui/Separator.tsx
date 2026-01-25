import { Component, JSX, splitProps } from "solid-js";
import { cn } from "../../lib/utils";

interface SeparatorProps extends JSX.HTMLAttributes<HTMLDivElement> {
  orientation?: "horizontal" | "vertical";
}

export const Separator: Component<SeparatorProps> = (props) => {
  const [local, others] = splitProps(props, ["orientation", "class"]);
  const orientation = local.orientation || "horizontal";

  return (
    <div
      class={cn(
        "ui-separator",
        local.class
      )}
      style={{
        "background-color": "var(--border-subtle)",
        ...(orientation === "horizontal" 
          ? { height: "1px", width: "100%" } 
          : { width: "1px", height: "100%" })
      }}
      {...others}
    />
  );
};
