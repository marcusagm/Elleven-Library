/**
 * Context Submenu Item Component
 *
 * Handles a submenu trigger and positions the nested menu using Floating UI.
 * Renders the submenu in a Portal to ensure it isn't clipped by parent containers.
 */

import { Component, createSignal, Show } from 'solid-js';
import { Portal, Dynamic } from 'solid-js/web';
import { ChevronRight } from 'lucide-solid';
import { cn } from '../../../../lib/utils';
import { SubmenuContextMenuItem, ContextMenuItem } from '../types';
import { useMenuPositioning } from '../useMenuPositioning';
import { MenuList } from './MenuList';

/**
 * Properties for a submenu item.
 */
interface SubmenuItemProps {
    /** The submenu item definition. */
    item: SubmenuContextMenuItem;
    /** Callback to close the entire menu tree. */
    onClose: () => void;
    /** Whether this item is currently visual focused. */
    isFocused: boolean;
    /** Whether the submenu content should be visible. */
    isActive: boolean;
    /** Keyboard event handler from the parent list. */
    onKeyDown: (event: KeyboardEvent, index: number, item: ContextMenuItem) => void;
    /** Current index in the list. */
    index: number;
    /** Callback to set focus to this item. */
    onFocus: () => void;
    /** Current nesting level. */
    level: number;
}

/**
 * Renders a menu item that opens a nested submenu.
 * Uses useMenuPositioning to ensure the submenu stays within viewport boundaries.
 *
 * @param {SubmenuItemProps} props - Component properties.
 * @returns {JSX.Element} The rendered submenu item.
 */
export const SubmenuItem: Component<SubmenuItemProps> = props => {
    /** Trigger element ref for positioning the floating submenu. */
    const [triggerElement, setTriggerElement] = createSignal<HTMLElement | null>(null);

    /** Positioning hook for the submenu content. */
    const { setFloatingElement, coordinates } = useMenuPositioning(
        () => (props.isActive ? triggerElement() : null),
        'right-start'
    );

    return (
        <div
            ref={setTriggerElement}
            class={cn(
                'ui-context-menu-item ui-context-menu-submenu-trigger',
                props.item.disabled && 'ui-context-menu-item-disabled',
                props.isFocused && 'ui-context-menu-item-focused'
            )}
            role="menuitem"
            aria-haspopup="menu"
            aria-expanded={props.isActive}
            tabIndex={props.item.disabled ? -1 : 0}
            onKeyDown={event => props.onKeyDown(event, props.index, props.item)}
            onFocus={() => props.onFocus()}
        >
            <span class="ui-context-menu-item-content">
                <Show when={props.item.icon}>
                    <Dynamic component={props.item.icon} size={14} />
                </Show>
                <span>{props.item.label}</span>
            </span>

            <ChevronRight size={14} class="ui-context-menu-chevron" />

            {/* Submenu render with Portal for boundary safety */}
            <Show when={props.isActive}>
                <Portal>
                    <div
                        ref={setFloatingElement}
                        class="ui-context-submenu"
                        style={{
                            position: 'fixed',
                            top: `${coordinates().top}px`,
                            left: `${coordinates().left}px`,
                            width: 'fit-content',
                            opacity: coordinates().top === 0 && coordinates().left === 0 ? 0 : 1,
                            'z-index': 10000 + props.level // Ensure submenus are on top of parent levels
                        }}
                    >
                        <MenuList
                            items={props.item.items}
                            onClose={props.onClose}
                            level={props.level + 1}
                        />
                    </div>
                </Portal>
            </Show>
        </div>
    );
};
