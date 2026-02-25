/**
 * Functional Dropdown Menu Item
 *
 * Standard selectable item for the dropdown menu, supporting labels, icons, and shortcuts.
 */

import { Component, Show } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { cn } from '../../../../lib/utils';
import { ActionMenuItem, DropdownContextValue } from '../types';

/**
 * Properties for a functional menu item.
 */
interface MenuItemProps {
    /** The action item definition containing label, action, icon, etc. */
    item: ActionMenuItem;
    /** Whether this item is currently focused via keyboard navigation. */
    isFocused?: boolean;
    /** Context for controlling the overall menu state. */
    context?: DropdownContextValue;
}

/**
 * Renders a standard menu item with label, optional icon, and keyboard shortcut.
 * Uses a semantic role for accessibility and handles click/selection logic.
 *
 * @param {MenuItemProps} props - Component properties.
 * @returns {JSX.Element} The rendered menu item.
 */
export const MenuItem: Component<MenuItemProps> = props => {
    /**
     * Executes the item's action and closes the menu tree.
     * Prevents event propagation to avoid triggering parent container actions.
     *
     * @param {MouseEvent} event - The click event object.
     */
    const handleClick = (event: MouseEvent) => {
        if (props.item.disabled) return;
        event.stopPropagation();

        props.item.action();
        props.context?.close();
    };

    return (
        <div
            class={cn(
                'ui-dropdown-item',
                props.isFocused && 'ui-dropdown-item-focused',
                props.item.disabled && 'ui-dropdown-item-disabled'
            )}
            role="menuitem"
            aria-disabled={props.item.disabled}
            onClick={handleClick}
        >
            <Show when={props.item.icon}>
                <Dynamic component={props.item.icon} size={14} class="ui-dropdown-item-icon" />
            </Show>

            <span class="ui-dropdown-item-label">{props.item.label}</span>

            <Show when={props.item.shortcut}>
                <span class="ui-dropdown-shortcut">{props.item.shortcut}</span>
            </Show>
        </div>
    );
};
