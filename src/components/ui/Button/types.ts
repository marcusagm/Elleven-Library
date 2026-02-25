import { JSX } from 'solid-js';

/**
 * Defines the visual style variants for the Button component.
 * - 'primary': The main action button style.
 * - 'secondary': An alternative action style.
 * - 'ghost': A transparent button that only shows on hover.
 * - 'ghost-destructive': A transparent button indicating a destructive action.
 * - 'destructive': A solid red button for dangerous actions.
 * - 'outline': A button with a border and transparent background.
 */
export type ButtonVariant =
    | 'primary'
    | 'secondary'
    | 'ghost'
    | 'ghost-destructive'
    | 'destructive'
    | 'outline';

/**
 * Defines the available sizes for the Button component.
 * - 'xs': Extra small button.
 * - 'sm': Small button.
 * - 'md': Medium button (default).
 * - 'lg': Large button.
 * - 'icon': Square button for icons.
 * - 'icon-sm': Small square button for icons.
 * - 'icon-xs': Extra small square button for icons.
 */
export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg' | 'icon' | 'icon-sm' | 'icon-xs';

/**
 * Properties for the Button component, extending standard HTML button attributes.
 */
export interface ButtonProperties extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
    /**
     * The visual style variant of the button.
     * @default 'primary'
     */
    variant?: ButtonVariant;
    /**
     * The size variant of the button.
     * @default 'md'
     */
    size?: ButtonSize;
    /**
     * Whether the button is in a loading state.
     * When true, the button is disabled and shows a spinner.
     */
    loading?: boolean;
    /**
     * An optional icon element to display before the button content.
     */
    leftIcon?: JSX.Element;
    /**
     * An optional icon element to display after the button content.
     */
    rightIcon?: JSX.Element;
    /**
     * The content to be rendered inside the button.
     */
    children?: JSX.Element;
}

/**
 * Defines the orientation options for the ButtonGroup component.
 */
export type ButtonGroupOrientation = 'horizontal' | 'vertical';

/**
 * Defines the size options for the ButtonGroup component.
 */
export type ButtonGroupSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the ButtonGroup component.
 */
export interface ButtonGroupProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /**
     * The orientation of the buttons within the group.
     * @default 'horizontal'
     */
    orientation?: ButtonGroupOrientation;
    /**
     * The size variant of the buttons within the group.
     * @default 'md'
     */
    size?: ButtonGroupSize;
    /**
     * Whether the buttons should appear attached to each other.
     * @default false
     */
    attached?: boolean;
    /**
     * The content to be rendered inside the button group, typically Button components.
     */
    children: JSX.Element;
}
