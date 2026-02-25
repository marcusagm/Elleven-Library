/**
 * Context Menu Item List
 *
 * Internal component for rendering the recursive list of items within a context menu.
 * High-level orchestrator for item types and keyboard flow.
 */

import { Component, For, Show, createSignal, Switch, Match } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { cn } from '../../../../lib/utils';
import {
    ContextMenuItem,
    ActionContextMenuItem,
    SubmenuContextMenuItem,
    CustomContextMenuItem
} from '../types';
import { SubmenuItem } from './SubmenuItem';

/**
 * Common properties for internal menu rendering levels.
 */
interface MenuListProps {
    /** Array of items to be displayed in the current list level. */
    items: ContextMenuItem[];
    /** Callback to close the entire menu tree. */
    onClose: () => void;
    /** Current nesting depth (0 for root context menu). */
    level?: number;
}

/**
 * Recursive menu list for ContextMenu.
 * Handles selection state, hover tracking, and integration with specialized item components.
 *
 * @param {MenuListProps} props - Component properties.
 * @returns {JSX.Element} The rendered menu list.
 */
export const MenuList: Component<MenuListProps> = props => {
    /** Index of the currently hovered/active submenu. */
    const [activeSubmenuIndex, setActiveSubmenuIndex] = createSignal<number | null>(null);
    /** Visual focus for keyboard/mouse guidance. */
    const [focusedIndex, setFocusedIndex] = createSignal(-1);

    /** Resolves current menu level (0-indexed). */
    const menuLevel = () => props.level ?? 0;

    /**
     * Internal keyboard navigation for context menu levels.
     * Manages circular navigation and sub-menu activation.
     *
     * @param {KeyboardEvent} event - The native keyboard event.
     * @param {number} itemIndex - The index of the item that received the event.
     * @param {ContextMenuItem} item - The structural definition of the item.
     */
    const handleKeyDown = (event: KeyboardEvent, itemIndex: number, item: ContextMenuItem) => {
        // Stop propagation to ensure only one level handles the navigation event
        event.stopPropagation();

        const handlers: Record<string, () => void> = {
            ArrowDown: () => {
                event.preventDefault();
                setFocusedIndex(previousIndex =>
                    previousIndex < props.items.length - 1 ? previousIndex + 1 : 0
                );
            },
            ArrowUp: () => {
                event.preventDefault();
                setFocusedIndex(previousIndex =>
                    previousIndex > 0 ? previousIndex - 1 : props.items.length - 1
                );
            },
            ArrowRight: () => {
                if (item.type === 'submenu' && !item.disabled) {
                    event.preventDefault();
                    setActiveSubmenuIndex(itemIndex);
                }
            },
            Enter: () => {
                event.preventDefault();
                if (item.type === 'item' && !item.disabled) {
                    item.action();
                    props.onClose();
                } else if (item.type === 'submenu' && !item.disabled) {
                    setActiveSubmenuIndex(itemIndex);
                }
            },
            ' ': () => {
                event.preventDefault();
                if (item.type === 'item' && !item.disabled) {
                    item.action();
                    props.onClose();
                } else if (item.type === 'submenu' && !item.disabled) {
                    setActiveSubmenuIndex(itemIndex);
                }
            },
            Escape: () => {
                event.preventDefault();
                props.onClose();
            }
        };

        const handler = handlers[event.key];
        if (handler) {
            handler();
        }
    };

    return (
        <div class="ui-context-menu-list" role="menu">
            <For each={props.items}>
                {(item, index) => (
                    <div
                        class="ui-context-menu-item-wrapper"
                        onMouseEnter={() => {
                            setFocusedIndex(index());

                            if (item.type === 'submenu') {
                                setActiveSubmenuIndex(index());
                            } else {
                                // Close any active submenu when hovering another item
                                setActiveSubmenuIndex(null);
                            }
                        }}
                        onMouseLeave={event => {
                            /**
                             * If the mouse moves out of a submenu item, we only close it
                             * if it's NOT moving into its own submenu content.
                             * However, since we now have physical overlap, this is handled
                             * more naturally by the pointer events chain.
                             */
                            if (item.type === 'submenu' && activeSubmenuIndex() === index()) {
                                const relatedTarget = event.relatedTarget as HTMLElement;
                                // If moving to something that isn't the submenu content, close it
                                if (
                                    !relatedTarget ||
                                    !relatedTarget.closest('.ui-context-submenu')
                                ) {
                                    setActiveSubmenuIndex(null);
                                }
                            }
                        }}
                    >
                        <Switch>
                            <Match when={item.type === 'separator'}>
                                <div class="ui-context-menu-separator" role="separator" />
                            </Match>

                            <Match when={item.type === 'custom'}>
                                <div class="ui-context-menu-custom">
                                    {(item as CustomContextMenuItem).content}
                                </div>
                            </Match>

                            <Match when={item.type === 'item'}>
                                {(() => {
                                    const actionItem = item as ActionContextMenuItem;
                                    return (
                                        <button
                                            type="button"
                                            class={cn(
                                                'ui-context-menu-item',
                                                actionItem.danger && 'ui-context-menu-item-danger',
                                                actionItem.disabled &&
                                                    'ui-context-menu-item-disabled',
                                                focusedIndex() === index() &&
                                                    'ui-context-menu-item-focused'
                                            )}
                                            role="menuitem"
                                            disabled={actionItem.disabled}
                                            onClick={event => {
                                                event.stopPropagation();
                                                actionItem.action();
                                                props.onClose();
                                            }}
                                            onKeyDown={event => handleKeyDown(event, index(), item)}
                                            onFocus={() => {
                                                setFocusedIndex(index());
                                            }}
                                        >
                                            <span class="ui-context-menu-item-content">
                                                <Show when={actionItem.icon}>
                                                    <Dynamic
                                                        component={actionItem.icon}
                                                        size={14}
                                                    />
                                                </Show>
                                                <span>{actionItem.label}</span>
                                            </span>
                                            <Show when={actionItem.shortcut}>
                                                <span class="ui-context-menu-shortcut">
                                                    {actionItem.shortcut}
                                                </span>
                                            </Show>
                                        </button>
                                    );
                                })()}
                            </Match>

                            <Match when={item.type === 'submenu'}>
                                <SubmenuItem
                                    item={item as SubmenuContextMenuItem}
                                    onClose={props.onClose}
                                    isFocused={focusedIndex() === index()}
                                    isActive={activeSubmenuIndex() === index()}
                                    onKeyDown={handleKeyDown}
                                    index={index()}
                                    onFocus={() => {
                                        setFocusedIndex(index());
                                    }}
                                    level={menuLevel()}
                                />
                            </Match>
                        </Switch>
                    </div>
                )}
            </For>
        </div>
    );
};
