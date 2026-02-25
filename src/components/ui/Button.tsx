import { Component, JSX, splitProps, Show } from 'solid-js';
import { cn } from '../../lib/utils';
import './button.css';

/**
 * Defines the visual style variants for the Button component.
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
 */
export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg' | 'icon' | 'icon-sm' | 'icon-xs';

/**
 * Properties for the Button component, extending standard HTML button attributes.
 */
export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
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
     * When true, the button is disabled and may show a spinner.
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
 * Button component with multiple variants, sizes, and states.
 *
 * @example
 * <Button>Click me</Button>
 *
 * @example
 * <Button variant="destructive" loading>
 *   Deleting...
 * </Button>
 *
 * @example
 * <Button variant="ghost" leftIcon={<Plus size={16} />}>
 *   Add item
 * </Button>
 */
export const Button: Component<ButtonProps> = props => {
    const [local, others] = splitProps(props, [
        'variant',
        'size',
        'loading',
        'leftIcon',
        'rightIcon',
        'class',
        'children',
        'disabled'
    ]);

    const variant = () => local.variant || 'primary';
    const size = () => local.size || 'md';
    const isDisabled = () => local.disabled || local.loading;

    return (
        <button
            class={cn(
                'ui-btn',
                `ui-btn-${variant()}`,
                `ui-btn-${size()}`,
                local.loading && 'ui-btn-loading',
                local.class
            )}
            disabled={isDisabled()}
            aria-busy={local.loading || undefined}
            {...others}
        >
            <Show when={local.loading}>
                <span class="ui-btn-spinner" aria-hidden="true" />
            </Show>

            <Show when={local.leftIcon && !local.loading}>
                <span class="ui-btn-icon ui-btn-icon-left" aria-hidden="true">
                    {local.leftIcon}
                </span>
            </Show>

            <Show when={local.children}>
                <span class="ui-btn-content">{local.children}</span>
            </Show>

            <Show when={local.rightIcon}>
                <span class="ui-btn-icon ui-btn-icon-right" aria-hidden="true">
                    {local.rightIcon}
                </span>
            </Show>
        </button>
    );
};
