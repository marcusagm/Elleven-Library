/**
 * Context Menu Component
 *
 * Provides a coordinate-based context menu that appears at a specific screen position.
 * Uses @floating-ui/dom for robust positioning and boundary detection.
 */

import { Component, createEffect, onCleanup, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { createClickOutside } from '../../../lib/primitives';
import { ContextMenuProps } from './types';
import { MenuList } from './components/MenuList';
import { useMenuPositioning } from './useMenuPositioning';
import './context-menu.css';

/**
 * ContextMenu component for right-click interactions.
 * Features:
 * - Viewport-aware positioning using Floating UI.
 * - Portal-based rendering to avoid parent container clipping.
 * - Backdrop to handle clicks and disable standard browser menu.
 *
 * @param {ContextMenuProps} props - Component properties.
 * @returns {JSX.Element} The rendered context menu.
 */
export const ContextMenu: Component<ContextMenuProps> = props => {
    /**
     * Virtual element to represent the mouse coordinates for Floating UI.
     */
    const virtualReference = () => ({
        getBoundingClientRect: () =>
            ({
                width: 0,
                height: 0,
                x: props.coordinateX,
                y: props.coordinateY,
                top: props.coordinateY,
                left: props.coordinateX,
                right: props.coordinateX,
                bottom: props.coordinateY
            }) as DOMRect
    });

    /**
     * Handles positioning relative to the virtual coordinate point.
     * We only provide the reference when the menu is open.
     */
    const { setFloatingElement, coordinates } = useMenuPositioning(
        () => (props.isOpen ? virtualReference() : null),
        'bottom-start'
    );

    /** Ref for click-outside detection. */
    let contentElement: HTMLDivElement | undefined;

    /**
     * Detection for clicks outside the menu container.
     */
    createClickOutside(
        () => contentElement,
        event => {
            if (!props.isOpen) return;

            // Check if the click target is within unknown context menu container or submenu.
            // This is necessary because submenus are rendered in Portals and thus
            // are not children of the root menu container.
            const target = event.target as HTMLElement;
            if (target && target.closest('.ui-context-menu-container, .ui-context-submenu')) {
                return;
            }

            props.onClose();
        }
    );

    /**
     * Listen for Escape key at the document level while the menu is open.
     */
    createEffect(() => {
        if (!props.isOpen) return;

        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                props.onClose();
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        onCleanup(() => document.removeEventListener('keydown', handleKeyDown));
    });

    return (
        <Show when={props.isOpen}>
            <Portal>
                {/* Backdrop to capture clicks and prevent nested context menus */}
                <div
                    class="ui-context-menu-backdrop"
                    onContextMenu={event => {
                        event.preventDefault();
                        props.onClose();
                    }}
                    onClick={() => props.onClose()}
                    role="presentation"
                />

                <div
                    ref={element => {
                        contentElement = element;
                        setFloatingElement(element);
                    }}
                    class="ui-context-menu-container"
                    style={{
                        position: 'fixed',
                        top: `${coordinates().top}px`,
                        left: `${coordinates().left}px`,
                        // Slight delay or check for (0,0) might be needed,
                        // but autoUpdate usually handles the first calculation quickly.
                        opacity: coordinates().top === 0 && coordinates().left === 0 ? 0 : 1,
                        'z-index': 9999
                    }}
                    onContextMenu={event => event.preventDefault()}
                >
                    <MenuList items={props.items} onClose={() => props.onClose()} />
                </div>
            </Portal>
        </Show>
    );
};
