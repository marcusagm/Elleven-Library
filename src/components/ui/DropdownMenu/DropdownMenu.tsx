import { Component, splitProps, createSignal, Show, createContext } from 'solid-js';
import { Portal } from 'solid-js/web';
import { cn } from '../../../lib/utils';
import { createClickOutside } from '../../../lib/primitives';
import { DropdownMenuProps, DropdownContextValue } from './types';
import { useMenuPositioning } from './useMenuPositioning';
import { MenuList } from './components/MenuList';
import './dropdown-menu.css';

/**
 * Context for the dropdown tree to share essential state like 'close' and radio values.
 */
export const DropdownMenuContext = createContext<DropdownContextValue>();

/**
 * Professional Dropdown Menu component.
 * Features:
 * - Robust positioning with @floating-ui/dom (collision detection, flipping, shifting).
 * - Full keyboard navigation support (Arrows, Enter, Esc, Home/End).
 * - Modal input scope integration to prevent global shortcut conflicts.
 * - Recursion support for nested submenus.
 * - Discriminated union based items for full type safety.
 *
 * @param props - Properties for the dropdown menu.
 * @returns The rendered dropdown component.
 */
export const DropdownMenu: Component<DropdownMenuProps> = props => {
    /**
     * Separate custom component properties from others.
     * We don't use abbreviations like 'pos' to comply with naming guidelines.
     */
    const [local] = splitProps(props, [
        'trigger',
        'items',
        'align',
        'side',
        'radioValue',
        'onRadioChange',
        'class',
        'contentClass'
    ]);

    /** State of the dropdown visibility. */
    const [isMenuOpen, setIsMenuOpen] = createSignal(false);

    /** Trigger and floating content refs managed by the positioning hook. */
    const { setTriggerReference, setFloatingElement, coordinates } = useMenuPositioning(
        () => local.align,
        () => local.side
    );

    /** Ref for the actual content container for click-outside detection. */
    let triggerElementContainer: HTMLDivElement | undefined;
    let floatingContentContainer: HTMLDivElement | undefined;

    /**
     * Closes the menu and resets any navigation state.
     */
    const handleClose = () => setIsMenuOpen(false);

    /**
     * Toggles the menu open/closed.
     */
    const handleToggle = () => setIsMenuOpen(!isMenuOpen());

    /**
     * Handles clicks outside the trigger and content to close the menu.
     */
    createClickOutside(
        () => [triggerElementContainer, floatingContentContainer].filter(Boolean) as HTMLElement[],
        () => {
            if (isMenuOpen()) handleClose();
        }
    );

    /**
     * Context value for children components.
     */
    const contextValue: DropdownContextValue = {
        close: handleClose,
        radioValue: () => local.radioValue || '',
        onRadioChange: (value: string) => local.onRadioChange?.(value)
    };

    return (
        <DropdownMenuContext.Provider value={contextValue}>
            <div class={cn('ui-dropdown', local.class)}>
                <div
                    ref={el => {
                        triggerElementContainer = el;
                        setTriggerReference(el);
                    }}
                    class="ui-dropdown-trigger"
                    onClick={handleToggle}
                >
                    {local.trigger}
                </div>

                <Show when={isMenuOpen()}>
                    <Portal>
                        <div
                            ref={el => {
                                floatingContentContainer = el;
                                setFloatingElement(el);
                            }}
                            class={cn('ui-dropdown-content', local.contentClass)}
                            style={{
                                position: 'fixed',
                                top: `${coordinates().top}px`,
                                left: `${coordinates().left}px`,
                                'z-index': 9999
                            }}
                        >
                            <MenuList items={local.items} context={contextValue} />
                        </div>
                    </Portal>
                </Show>
            </div>
        </DropdownMenuContext.Provider>
    );
};
