import { Component, splitProps, Show } from 'solid-js';
import { cn } from '../../../lib/utils';
import { ButtonProperties } from './types';
import './button.css';

/**
 * Button component with multiple variants, sizes, and states.
 * Follows the Mundam design system guidelines for visual excellence.
 *
 * @param {ButtonProperties} properties - The properties for the Button component.
 * @returns {JSX.Element} The rendered button element.
 *
 * @example
 * <Button variant="primary" size="md" onClick={() => console.log('Clicked')}>
 *   Click Me
 * </Button>
 *
 * @example
 * <Button variant="ghost" leftIcon={<Plus size={16} />}>
 *   Add Item
 * </Button>
 *
 * @example
 * <Button variant="destructive" loading>
 *   Deleting...
 * </Button>
 */
export const Button: Component<ButtonProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'variant',
        'size',
        'loading',
        'leftIcon',
        'rightIcon',
        'class',
        'children',
        'disabled'
    ]);

    const activeVariant = () => localProperties.variant || 'primary';
    const activeSize = () => localProperties.size || 'md';
    const isDisabled = () => localProperties.disabled || localProperties.loading;

    return (
        <button
            class={cn(
                'ui-btn',
                `ui-btn-${activeVariant()}`,
                `ui-btn-${activeSize()}`,
                localProperties.loading && 'ui-btn-loading',
                localProperties.class
            )}
            disabled={isDisabled()}
            aria-busy={localProperties.loading || undefined}
            {...remainingProperties}
        >
            <Show when={localProperties.loading}>
                <span class="ui-btn-spinner" aria-hidden="true" />
            </Show>

            <Show when={localProperties.leftIcon && !localProperties.loading}>
                <span class="ui-btn-icon ui-btn-icon-left" aria-hidden="true">
                    {localProperties.leftIcon}
                </span>
            </Show>

            <Show when={localProperties.children}>
                <span class="ui-btn-content">{localProperties.children}</span>
            </Show>

            <Show when={localProperties.rightIcon}>
                <span class="ui-btn-icon ui-btn-icon-right" aria-hidden="true">
                    {localProperties.rightIcon}
                </span>
            </Show>
        </button>
    );
};
