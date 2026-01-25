import { Component, JSX, splitProps } from "solid-js";
import { cn } from "../../lib/utils";
import "./button.css";

type ButtonVariant = "primary" | "secondary" | "ghost" | "destructive";
type ButtonSize = "sm" | "md" | "icon" | "icon-sm";

export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  children?: JSX.Element;
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ["variant", "size", "class", "children"]);
  
  const variant = local.variant || "primary";
  const size = local.size || "md";

  return (
    <button
      class={cn(
        "ui-btn",
        `ui-btn-${variant}`,
        `ui-btn-${size}`,
        local.class
      )}
      {...others}
    >
      {local.children}
    </button>
  );
};
