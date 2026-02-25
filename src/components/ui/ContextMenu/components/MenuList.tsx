import { Component, For, Show, createSignal, Switch, Match } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import { ChevronRight } from 'lucide-solid';
import { cn } from '../../../../lib/utils';
import {
    ContextMenuItem,
    ActionContextMenuItem,
    SubmenuContextMenuItem,
    CustomContextMenuItem
} from '../types';

/**
 * Common properties for internal menu rendering levels.
 */
interface MenuListProps {
    /** Array of items to be displayed. */
    items: ContextMenuItem[];
    /** Callback to close the menu. */
    onClose: () => void;
    /** Current nesting level (0 for root). */
    level?: number;
}

/**
 * Recursive menu list for ContextMenu.
 */
export const MenuList: Component<MenuListProps> = props => {
    /** Index of the currently hovered/active submenu. */
    const [activeSubmenuIndex, setActiveSubmenuIndex] = createSignal<number | null>(null);
    /** Visual focus for keyboard/mouse guidance. */
    const [focusedIndex, setFocusedIndex] = createSignal(-1);

    const menuLevel = () => props.level ?? 0;

    /**
     * Internal keyboard navigation for context menu levels.
     */
    const handleKeyDown = (event: KeyboardEvent, index: number, item: ContextMenuItem) => {
        // Stop propagation to ensure only one level handles the event
        event.stopPropagation();

        const handlers: Record<string, () => void> = {
            ArrowDown: () => {
                event.preventDefault();
                setFocusedIndex(prev => (prev < props.items.length - 1 ? prev + 1 : 0));
            },
            ArrowUp: () => {
                event.preventDefault();
                setFocusedIndex(prev => (prev > 0 ? prev - 1 : props.items.length - 1));
            },
            ArrowRight: () => {
                if (item.type === 'submenu' && !item.disabled) {
                    event.preventDefault();
                    setActiveSubmenuIndex(index);
                }
            },
            Enter: () => {
                event.preventDefault();
                if (item.type === 'item' && !item.disabled) {
                    item.action();
                    props.onClose();
                } else if (item.type === 'submenu' && !item.disabled) {
                    setActiveSubmenuIndex(index);
                }
            },
            ' ': () => {
                event.preventDefault();
                if (item.type === 'item' && !item.disabled) {
                    item.action();
                    props.onClose();
                } else if (item.type === 'submenu' && !item.disabled) {
                    setActiveSubmenuIndex(index);
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
                            if (item.type === 'submenu') setActiveSubmenuIndex(index());
                        }}
                        onMouseLeave={() => {
                            if (item.type === 'submenu') setActiveSubmenuIndex(null);
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
                                            onFocus={() => setFocusedIndex(index())}
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
                                {(() => {
                                    const submenuItem = item as SubmenuContextMenuItem;
                                    return (
                                        <div
                                            class={cn(
                                                'ui-context-menu-item ui-context-menu-submenu-trigger',
                                                submenuItem.disabled &&
                                                    'ui-context-menu-item-disabled',
                                                focusedIndex() === index() &&
                                                    'ui-context-menu-item-focused'
                                            )}
                                            role="menuitem"
                                            aria-haspopup="menu"
                                            aria-expanded={activeSubmenuIndex() === index()}
                                            tabIndex={submenuItem.disabled ? -1 : 0}
                                            onKeyDown={event => handleKeyDown(event, index(), item)}
                                            onFocus={() => setFocusedIndex(index())}
                                        >
                                            <span class="ui-context-menu-item-content">
                                                <Show when={submenuItem.icon}>
                                                    <Dynamic
                                                        component={submenuItem.icon}
                                                        size={14}
                                                    />
                                                </Show>
                                                <span>{submenuItem.label}</span>
                                            </span>
                                            <ChevronRight
                                                size={14}
                                                class="ui-context-menu-chevron"
                                            />

                                            <Show when={activeSubmenuIndex() === index()}>
                                                <div class="ui-context-submenu">
                                                    <MenuList
                                                        items={submenuItem.items}
                                                        onClose={props.onClose}
                                                        level={menuLevel() + 1}
                                                    />
                                                </div>
                                            </Show>
                                        </div>
                                    );
                                })()}
                            </Match>
                        </Switch>
                    </div>
                )}
            </For>
        </div>
    );
};
