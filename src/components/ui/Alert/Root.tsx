import { Component, splitProps, Show } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { CircleAlert, CircleCheck, TriangleAlert, Info, X as CloseIcon } from 'lucide-solid';
import { cn as concatenateClasses } from '../../../lib/utils';
import { AlertProperties, AlertVariant } from './types';
import './alert.css';

/**
 * Standard icons for each alert variant.
 */
const variantIconMap: Record<AlertVariant, Component<{ size?: number | string }>> = {
    default: Info,
    info: Info,
    success: CircleCheck,
    warning: TriangleAlert,
    destructive: CircleAlert
};

/**
 * Alert component for displaying important messages or feedback.
 * Supports multiple visual variants and optional dismissal.
 *
 * @param {AlertProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered alert component.
 *
 * @example
 * <Alert.Root variant="success" title="Success!">
 *   Your changes have been saved.
 * </Alert.Root>
 *
 * @example
 * <Alert.Root variant="destructive" isDismissible onDismiss={handleDismiss}>
 *   <Alert.Title>Error</Alert.Title>
 *   <Alert.Description>Something went wrong.</Alert.Description>
 * </Alert.Root>
 */
export const AlertRoot: Component<AlertProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'variant',
        'icon',
        'title',
        'isDismissible',
        'onDismiss',
        'children'
    ]);

    const activeVariant = () => localProperties.variant || 'default';
    const IconComponent = () => localProperties.icon || variantIconMap[activeVariant()];

    return (
        <div
            class={concatenateClasses(
                'ui-alert',
                `ui-alert-${activeVariant()}`,
                localProperties.class
            )}
            role="alert"
            {...remainingProperties}
        >
            <span class="ui-alert-icon">
                <Dynamic component={IconComponent()} />
            </span>

            <div class="ui-alert-content">
                <Show when={localProperties.title}>
                    <h5 class="ui-alert-title">{localProperties.title}</h5>
                </Show>
                <Show when={localProperties.children}>{localProperties.children}</Show>
            </div>

            <Show when={localProperties.isDismissible}>
                <button
                    type="button"
                    class="ui-alert-dismiss"
                    onClick={() => localProperties.onDismiss?.()}
                    aria-label="Dismiss alert"
                >
                    <CloseIcon size={14} />
                </button>
            </Show>
        </div>
    );
};
