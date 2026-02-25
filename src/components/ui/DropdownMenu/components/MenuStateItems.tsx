/**
 * Dropdown Stateful Items
 *
 * Specialized menu item components for handling selectable states (checkboxes and radios).
 */

import { Component, Show, createMemo } from 'solid-js';
import { Check } from 'lucide-solid';
import { cn } from '../../../../lib/utils';
import { CheckboxMenuItem, RadioMenuItem, DropdownContextValue } from '../types';

/**
 * Common indicator for checkbox and radio items.
 *
 * @param props - Includes the item type and active status.
 */
const MenuIndicator: Component<{
    type: 'checkbox' | 'radio';
    isActive: boolean;
}> = props => {
    return (
        <span class="ui-dropdown-indicator">
            <Show when={props.isActive}>
                <Show when={props.type === 'checkbox'}>
                    <Check size={12} class="ui-dropdown-indicator-check" />
                </Show>

                <Show when={props.type === 'radio'}>
                    <div class="ui-dropdown-radio-dot" />
                </Show>
            </Show>
        </span>
    );
};

/**
 * Properties for a checkbox menu item.
 */
interface MenuCheckboxItemProps {
    item: CheckboxMenuItem;
    isFocused?: boolean;
}

/**
 * Checkbox menu item implementation.
 */
export const MenuCheckboxItem: Component<MenuCheckboxItemProps> = props => {
    const handleToggle = (event: MouseEvent) => {
        if (props.item.disabled) return;
        event.stopPropagation();

        props.item.onCheckedChange(!props.item.checked);
    };

    return (
        <div
            class={cn(
                'ui-dropdown-item ui-dropdown-checkbox',
                props.isFocused && 'ui-dropdown-item-focused',
                props.item.disabled && 'ui-dropdown-item-disabled'
            )}
            role="menuitemcheckbox"
            aria-checked={props.item.checked}
            aria-disabled={props.item.disabled}
            onClick={handleToggle}
        >
            <MenuIndicator type="checkbox" isActive={props.item.checked} />
            <span class="ui-dropdown-item-label">{props.item.label}</span>
        </div>
    );
};

/**
 * Properties for a radio menu item.
 */
interface MenuRadioItemProps {
    item: RadioMenuItem;
    isFocused?: boolean;
    context?: DropdownContextValue;
}

/**
 * Radio menu item implementation with context coordination.
 */
export const MenuRadioItem: Component<MenuRadioItemProps> = props => {
    const isSelected = createMemo(() => props.context?.radioValue?.() === props.item.value);

    const handleSelect = (event: MouseEvent) => {
        if (props.item.disabled) return;
        event.stopPropagation();

        props.context?.onRadioChange?.(props.item.value);
    };

    return (
        <div
            class={cn(
                'ui-dropdown-item ui-dropdown-radio',
                props.isFocused && 'ui-dropdown-item-focused',
                props.item.disabled && 'ui-dropdown-item-disabled'
            )}
            role="menuitemradio"
            aria-checked={isSelected()}
            aria-disabled={props.item.disabled}
            onClick={handleSelect}
        >
            <MenuIndicator type="radio" isActive={isSelected()} />
            <span class="ui-dropdown-item-label">{props.item.label}</span>
        </div>
    );
};
