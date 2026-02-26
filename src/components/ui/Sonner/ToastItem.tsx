import { Component, createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { X, CheckCircle2, AlertCircle, AlertTriangle, Info } from 'lucide-solid';
import { cn as concatenateClasses } from '../../../lib/utils';
import { ToastItemProperties, ToastIconMap } from './types';
import { removeToastReference } from './state';

/**
 * Maps toast variants to their respective icons from the lucide-solid library.
 * @internal
 */
const toastVariantIcons: ToastIconMap = {
    default: Info,
    success: CheckCircle2,
    error: AlertCircle,
    warning: AlertTriangle,
    info: Info
};

/**
 * ToastItem represents a single notification card within the stacked container.
 * It manages its own auto-dismiss timer and exit animation state.
 *
 * @param {ToastItemProperties} properties - Properties for rendering the individual toast.
 * @returns {JSX.Element} The rendered toast item markup.
 * @internal
 */
export const ToastItem: Component<ToastItemProperties> = properties => {
    const [isExitingAnimationActive, setIsExitingAnimationActive] = createSignal(false);

    /**
     * Triggers the exit animation and removes the toast after completion.
     */
    const handleDismissAction = () => {
        setIsExitingAnimationActive(true);
        setTimeout(() => removeToastReference(properties.toast.identifier), 200);
    };

    /**
     * Automatically dismisses the toast after the specified duration.
     * Effect is re-evaluated when the duration property changes.
     */
    createEffect(() => {
        if (properties.toast.duration && properties.toast.duration > 0) {
            const autoDismissTimer = setTimeout(handleDismissAction, properties.toast.duration);
            onCleanup(() => clearTimeout(autoDismissTimer));
        }
    });

    /**
     * Calculates the zero-based index of the toast from the front of the stack.
     * @returns {number} The reverse index for styling.
     */
    const calculateReverseIndex = () => properties.totalCount - 1 - properties.index;

    return (
        <div
            class={concatenateClasses(
                'ui-toast',
                `ui-toast-${properties.toast.variant}`,
                isExitingAnimationActive() && 'ui-toast-exiting'
            )}
            role="alert"
            aria-live="polite"
            data-index={calculateReverseIndex()}
            data-expanded={properties.isStackExpanded}
            style={{
                '--index': calculateReverseIndex(),
                'z-index': properties.index,
                opacity: calculateReverseIndex() >= 3 && !properties.isStackExpanded ? 0 : 1,
                'pointer-events':
                    calculateReverseIndex() > 0 && !properties.isStackExpanded ? 'none' : 'auto'
            }}
        >
            <div class="ui-toast-icon">
                <Dynamic component={toastVariantIcons[properties.toast.variant]} />
            </div>

            <div class="ui-toast-content">
                <div class="ui-toast-title">{properties.toast.title}</div>
                <Show when={properties.toast.description}>
                    <div class="ui-toast-description">{properties.toast.description}</div>
                </Show>
            </div>

            <div class="ui-toast-actions">
                <Show when={properties.toast.action}>
                    <button
                        class="ui-toast-action-btn"
                        onClick={() => {
                            properties.toast.action!.onClick();
                            handleDismissAction();
                        }}
                    >
                        {properties.toast.action!.label}
                    </button>
                </Show>

                <Show when={properties.toast.isDismissible}>
                    <button
                        class="ui-toast-close"
                        onClick={handleDismissAction}
                        aria-label="Dismiss Notification"
                    >
                        <X size={14} />
                    </button>
                </Show>
            </div>
        </div>
    );
};
