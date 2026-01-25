import { Component, JSX, splitProps } from "solid-js";
import { cn } from "../../lib/utils";
import "./badge.css";

type BadgeVariant = "default" | "outline" | "secondary";

interface BadgeProps extends JSX.HTMLAttributes<HTMLDivElement> {
  variant?: BadgeVariant;
}

export const Badge: Component<BadgeProps> = (props) => {
  const [local, others] = splitProps(props, ["variant", "class", "children"]);
  const variant = local.variant || "default";

  return (
    <div
      class={cn("ui-badge", `ui-badge-${variant}`, local.class)}
      {...others}
    >
      {local.children}
    </div>
  );
};
