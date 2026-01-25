import { Component, JSX, splitProps, Show } from "solid-js";
import { cn } from "../../lib/utils";
import "./input.css";

export interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  leftIcon?: JSX.Element;
}

export const Input: Component<InputProps> = (props) => {
  const [local, others] = splitProps(props, ["leftIcon", "class"]);

  return (
    <div class="ui-input-wrapper">
      <Show when={local.leftIcon}>
        <div class="ui-input-icon-left">
          {local.leftIcon}
        </div>
      </Show>
      <input
        class={cn(
          "ui-input",
          !!local.leftIcon && "ui-input-has-left-icon",
          local.class
        )}
        {...others}
      />
    </div>
  );
};
