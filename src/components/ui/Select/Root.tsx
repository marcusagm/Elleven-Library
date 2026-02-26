import { Component, createSignal, createMemo, createEffect, onCleanup } from 'solid-js';
import { createControllableSignal, createClickOutside } from '../../../lib/primitives';
import { SelectContext } from './context';
import { SelectRootProperties } from './types';

/**
 * Select.Root component manages the internal state and context for the Select suite.
 *
 * @param {SelectRootProperties} properties - Properties for state management.
 * @returns {JSX.Element} The logic wrapper providing state to children.
 */
export const Root: Component<SelectRootProperties> = properties => {
    const [isOpen, setIsOpen] = createSignal(false);
    const [searchQuery, setSearchQuery] = createSignal('');
    const [highlightedIndex, setHighlightedIndex] = createSignal(-1);
    const [contentPosition, setContentPosition] = createSignal({ top: 0, left: 0, width: 0 });

    const [triggerReference, setTriggerReference] = createSignal<HTMLButtonElement>();
    const [contentReference, setContentReference] = createSignal<HTMLDivElement>();

    /**
     * Internal signal for managing the controlled and uncontrolled selection state.
     */
    const { value: selectedValue, setValue: setSelectedValue } = createControllableSignal({
        value: () => properties.value,
        defaultValue: properties.defaultValue ?? '',
        onChange: (value: string) => properties.onValueChange?.(value)
    });

    /**
     * Logic to position the dropdown content based on the trigger's coordinates.
     */
    const recalculatePosition = () => {
        const trigger = triggerReference();
        const content = contentReference();
        if (!trigger || !content || !isOpen()) {
            return;
        }

        const triggerRect = trigger.getBoundingClientRect();
        const contentRect = content.getBoundingClientRect();
        const viewportHeight = window.innerHeight;
        const verticalOffset = 4;

        let top = triggerRect.bottom + verticalOffset;
        const left = triggerRect.left;

        // Check for viewport overflow (bottom overflow)
        if (top + contentRect.height > viewportHeight - 10) {
            const spaceAbove = triggerRect.top - verticalOffset;
            if (spaceAbove > contentRect.height) {
                // If there's space above, flip the content
                top = triggerRect.top - contentRect.height - verticalOffset;
            } else {
                // Otherwise, stick to the bottom
                top = Math.max(10, viewportHeight - contentRect.height - 10);
            }
        }

        setContentPosition({
            top,
            left,
            width: triggerRect.width
        });
    };

    /**
     * Closes the dropdown when a click outside the component occurs.
     */
    createClickOutside(
        () => [triggerReference(), contentReference()].filter(Boolean) as HTMLElement[],
        () => {
            if (isOpen()) {
                setIsOpen(false);
            }
        }
    );

    /**
     * Listens for scroll and resize events while the dropdown is open.
     */
    createEffect(() => {
        if (isOpen()) {
            // Initial positioning
            requestAnimationFrame(recalculatePosition);

            const handleLayoutChange = () => requestAnimationFrame(recalculatePosition);
            window.addEventListener('scroll', handleLayoutChange, true);
            window.addEventListener('resize', handleLayoutChange);

            onCleanup(() => {
                window.removeEventListener('scroll', handleLayoutChange, true);
                window.removeEventListener('resize', handleLayoutChange);
            });
        }
    });

    /**
     * Derived options accessor for simple usage.
     */
    const allOptions = createMemo(() => properties.options || []);

    const contextValue = {
        value: selectedValue,
        setValue: setSelectedValue,
        isOpen,
        setIsOpen,
        disabled: () => properties.disabled,
        searchQuery,
        setSearchQuery,
        highlightedIndex,
        setHighlightedIndex,
        contentPosition,
        setContentPosition,
        triggerElement: triggerReference,
        setTriggerElement: (element: HTMLButtonElement) => setTriggerReference(element),
        contentElement: contentReference,
        setContentElement: (element: HTMLDivElement) => setContentReference(element),
        options: allOptions
    };

    return (
        <SelectContext.Provider value={contextValue}>
            {typeof properties.children === 'function'
                ? properties.children(contextValue)
                : properties.children}
        </SelectContext.Provider>
    );
};
