import { Component, createSignal, For } from 'solid-js';
import { Portal } from 'solid-js/web';
import { cn as concatenateClasses } from '../../../lib/utils';
import { ToasterProperties } from './types';
import { getActiveToastState } from './state';
import { ToastItem } from './ToastItem';
import './sonner.css';

/**
 * Toaster component acts as a high-performance portal container for toast notifications.
 * It manages the stacking, interaction states (expansion on hover), and placement.
 *
 * @param {ToasterProperties} properties - Configuration for the toaster placement and appearance.
 * @returns {JSX.Element} A portal-based container for toast notifications.
 *
 * @example
 * // Place this at the root of your application (e.g., App.tsx)
 * <Toaster position="bottom-right" useRichColors={true} />
 *
 * // Trigger with:
 * toast.success("Operation successful!");
 */
export const Toaster: Component<ToasterProperties> = properties => {
    /**
     * Resolves the current placement of the toaster, defaulting to 'bottom-right'.
     */
    const activePosition = () => properties.position || 'bottom-right';

    /**
     * State tracking whether the user is currently hovering the toaster stack.
     * When expanded, toasts shift to show full descriptions and multiple items.
     */
    const [isStackExpanded, setIsStackExpanded] = createSignal(false);

    return (
        <Portal>
            <div
                class={concatenateClasses(
                    'ui-toaster',
                    `ui-toaster-${activePosition()}`,
                    properties.isExpandedByDefault && 'ui-toaster-expand',
                    properties.useRichColors && 'ui-toaster-rich'
                )}
                data-expanded={isStackExpanded()}
                onMouseEnter={() => setIsStackExpanded(true)}
                onMouseLeave={() => setIsStackExpanded(false)}
            >
                <For each={getActiveToastState().activeToasts}>
                    {(toast, indexReference) => (
                        <ToastItem
                            toast={toast}
                            index={indexReference()}
                            totalCount={getActiveToastState().activeToasts.length}
                            isStackExpanded={isStackExpanded()}
                        />
                    )}
                </For>
            </div>
        </Portal>
    );
};
