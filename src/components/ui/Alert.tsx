import { Component, JSX, splitProps, Show } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { cn as concatenateClasses } from '../../lib/utils';
import { CircleAlert, CircleCheck, TriangleAlert, Info, X as CloseIcon } from 'lucide-solid';
import './alert.css';

type AlertVariant = 'default' | 'info' | 'success' | 'warning' | 'destructive';

/**
 * Properties for the Alert component.
 */
export interface AlertProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** The visual variant of the alert */
    variant?: AlertVariant;
    /** Optional custom icon to display */
    icon?: Component<{ size?: number | string }>;
    /** Title of the alert */
    title?: string;
    /** Whether the alert can be dismissed by the user */
    dismissible?: boolean;
    /** Callback triggered when the alert is dismissed */
    onDismiss?: () => void;
    /** Content to display inside the alert */
    children?: JSX.Element;
}

const variantIcons: Record<AlertVariant, Component<{ size?: number | string }>> = {
    default: Info,
    info: Info,
    success: CircleCheck,
    warning: TriangleAlert,
    destructive: CircleAlert
};

/**
 * Alert component for displaying important messages.
 * Supports multiple variants and optional dismissal.
 *
 * @param {AlertProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered solid-js component.
 *
 * @example
 * <Alert variant="success" title="Success!">
 *   Your changes have been saved.
 * </Alert>
 *
 * <Alert variant="destructive" dismissible onDismiss={() => {}}>
 *   Something went wrong.
 * </Alert>
 */
export const Alert: Component<AlertProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'variant',
        'icon',
        'title',
        'dismissible',
        'onDismiss',
        'children'
    ]);

    const variant = () => localProperties.variant || 'default';
    const IconComponent = () => localProperties.icon || variantIcons[variant()];

    return (
        <div
            class={concatenateClasses('ui-alert', `ui-alert-${variant()}`, localProperties.class)}
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
                <Show when={localProperties.children}>
                    <div class="ui-alert-description">{localProperties.children}</div>
                </Show>
            </div>

            <Show when={localProperties.dismissible}>
                <button
                    type="button"
                    class="ui-alert-dismiss"
                    onClick={() => localProperties.onDismiss?.()}
                    aria-label="Dismiss"
                >
                    <CloseIcon size={14} />
                </button>
            </Show>
        </div>
    );
};

/**
 * Component for the title of an alert.
 *
 * @param {Object} properties - Component properties.
 * @param {JSX.Element} properties.children - The title content.
 * @returns {JSX.Element} The rendered title component.
 */
export const AlertTitle: Component<{ children: JSX.Element }> = properties => (
    <h5 class="ui-alert-title">{properties.children}</h5>
);

/**
 * Component for the description or body of an alert.
 *
 * @param {Object} properties - Component properties.
 * @param {JSX.Element} properties.children - The description content.
 * @returns {JSX.Element} The rendered description component.
 */
export const AlertDescription: Component<{ children: JSX.Element }> = properties => (
    <div class="ui-alert-description">{properties.children}</div>
);
