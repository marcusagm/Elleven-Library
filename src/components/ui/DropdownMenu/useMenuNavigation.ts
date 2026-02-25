import { createSignal, createEffect, onCleanup, Accessor } from 'solid-js';
import { useInput, createShortcut, SCOPE_PRIORITIES } from '../../../core/input';
import { DropdownMenuItem } from './types';

/**
 * Hook to manage keyboard navigation and focus state within a dropdown menu.
 * Integrates with the core input system to ensure modal behavior and shortcut safety.
 *
 * @param items - Reactive accessor for the list of menu items.
 * @param onClose - Callback to close the menu.
 * @returns Object containing focus state and accessibility.
 */
export const useMenuNavigation = (items: Accessor<DropdownMenuItem[]>, onClose: () => void) => {
    const inputService = useInput();
    /** Index of the currently focused item in the flat list of visible items. */
    const [focusedItemIndex, setFocusedItemIndex] = createSignal<number>(-1);
    /** Whether the menu is currently being navigated via keyboard. */
    const [isKeyboardFocusActive, setIsKeyboardFocusActive] = createSignal(false);

    /**
     * Filters out non-interactive items (separators, labels, disabled items)
     * to determine which indices are reachable via navigation.
     */
    const getSelectableIndices = () => {
        return items()
            .map((item, index) => {
                const isInteractive = item.type !== 'separator' && item.type !== 'label';
                const isEnabled = !('disabled' in item && item.disabled);
                return isInteractive && isEnabled ? index : -1;
            })
            .filter(index => index !== -1);
    };

    /**
     * Moves focus to the next or previous selectable item.
     *
     * @param direction - 1 for forward, -1 for backward.
     */
    const moveFocus = (direction: 1 | -1) => {
        const selectableIndices = getSelectableIndices();
        if (selectableIndices.length === 0) return;

        const currentIndex = focusedItemIndex();
        const currentSelectablePosition = selectableIndices.indexOf(currentIndex);

        let nextPosition = currentSelectablePosition + direction;

        // Loop around if at the ends
        if (nextPosition >= selectableIndices.length) nextPosition = 0;
        if (nextPosition < 0) nextPosition = selectableIndices.length - 1;

        setFocusedItemIndex(selectableIndices[nextPosition]);
        setIsKeyboardFocusActive(true);
    };

    /**
     * Triggers the action of the currently focused item.
     */
    const triggerFocusedItem = () => {
        const index = focusedItemIndex();
        if (index === -1) return;

        const item = items()[index];
        if (!item) return;

        if (item.type === 'item') {
            item.action();
            onClose();
        } else if (item.type === 'checkbox') {
            item.onCheckedChange(!item.checked);
        }
    };

    // =============================================================================
    // Keyboard Shortcuts (Aligned with core/input)
    // =============================================================================

    createShortcut({
        keys: 'ArrowDown',
        scope: 'menu',
        priority: 1510,
        action: () => moveFocus(1),
        preventDefault: true
    });

    createShortcut({
        keys: 'ArrowUp',
        scope: 'menu',
        priority: 1510,
        action: () => moveFocus(-1),
        preventDefault: true
    });

    createShortcut({
        keys: 'Home',
        scope: 'menu',
        priority: 1510,
        action: () => {
            const firstSelectable = getSelectableIndices()[0];
            if (firstSelectable !== undefined) setFocusedItemIndex(firstSelectable);
        },
        preventDefault: true
    });

    createShortcut({
        keys: 'End',
        scope: 'menu',
        priority: 1510,
        action: () => {
            const selectableItems = getSelectableIndices();
            const lastSelectable = selectableItems[selectableItems.length - 1];
            if (lastSelectable !== undefined) setFocusedItemIndex(lastSelectable);
        },
        preventDefault: true
    });

    createShortcut({
        keys: ['Enter', 'Space'],
        scope: 'menu',
        priority: 1510,
        action: () => triggerFocusedItem(),
        preventDefault: true
    });

    createShortcut({
        keys: ['Escape', 'Tab'],
        scope: 'menu',
        priority: 1510,
        action: () => onClose(),
        preventDefault: true
    });

    // Activate the 'menu' scope when the menu is open to block global shortcuts safely.
    // Priority is set slightly higher than standard modals to ensure menu takes precedence.
    createEffect(() => {
        inputService.pushScope('menu', SCOPE_PRIORITIES.modal + 100, true);
        onCleanup(() => inputService.popScope('menu'));
    });

    return {
        focusedItemIndex,
        setFocusedItemIndex,
        isKeyboardFocusActive,
        setIsKeyboardFocusActive
    };
};
