import { Component, Show } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { cn } from '../../../../lib/utils';
import { ActionMenuItem, DropdownContextValue } from '../types';

/**
 * Properties for a functional menu item.
 */
interface MenuItemProps {
    /** The action item definition. */
    item: ActionMenuItem;
    /** Whether this item is currently focused via keyboard. */
    isFocused?: boolean;
    /** Context for menu control. */
    context?: DropdownContextValue;
}

/**
 * Renders a standard menu item with label, optional icon, and shortcut.
 * Uses a semantic role for accessibility.
 */
export const MenuItem: Component<MenuItemProps> = props => {
    /**
     * Executes the item's action and closes the menu tree.
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
