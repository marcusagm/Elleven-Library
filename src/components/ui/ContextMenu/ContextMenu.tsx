/**
 * Context Menu Component
 *
 * Provides a coordinate-based context menu that appears at a specific screen position.
 * Handles viewport collisions to ensure the menu remains visible.
 */

import { Component, createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { createClickOutside } from '../../../lib/primitives';
import { ContextMenuProps } from './types';
import { MenuList } from './components/MenuList';
import './context-menu.css';

/**
 * ContextMenu component for right-click displays.
 * Supports standard items, submenus, and custom layouts.
 * Uses a coordinate-based positioning system.
 *
 * @param {ContextMenuProps} props - Properties for the context menu.
 * @returns {JSX.Element} The rendered context menu.
 *
 * @example
 * <ContextMenu
 *   coordinateX={100}
 *   coordinateY={200}
 *   isOpen={true}
 *   items={menuItems}
 *   onClose={() => setOpen(false)}
 * />
 */
export const ContextMenu: Component<ContextMenuProps> = props => {
    /** Target ref for the container for positioning and clicking outside. */
    let containerRef: HTMLDivElement | undefined;
    /** Current calculated placement coordinates. */
    const [coordinates, setCoordinates] = createSignal({ top: 0, left: 0 });
    /** Controls opacity to avoid flicker before positioning. */
    const [isVisible, setIsVisible] = createSignal(false);

    /**
     * Re-calculates placement when menu opens or coordinates change.
     */
    createEffect(() => {
        if (!props.isOpen || !containerRef) {
            setIsVisible(false);
            return;
        }

        // Initially move to requested position to allow measurement
        containerRef.style.top = `${props.coordinateY}px`;
        containerRef.style.left = `${props.coordinateX}px`;

        requestAnimationFrame(() => {
            if (!containerRef) return;

            const menuBoundingRect = containerRef.getBoundingClientRect();
            const viewportWidth = window.innerWidth;
            const viewportHeight = window.innerHeight;

            let topCoordinate = props.coordinateY;
            let leftCoordinate = props.coordinateX;

            // Collision detection for right boundary
            if (leftCoordinate + menuBoundingRect.width > viewportWidth) {
                leftCoordinate = Math.max(0, viewportWidth - menuBoundingRect.width - 8);
            }
            // Collision detection for bottom boundary
            if (topCoordinate + menuBoundingRect.height > viewportHeight) {
                topCoordinate = Math.max(0, viewportHeight - menuBoundingRect.height - 8);
            }

            setCoordinates({ top: topCoordinate, left: leftCoordinate });
            setIsVisible(true);
        });
    });

    /**
     * Detection for clicks outside the menu container.
     */
    createClickOutside(
        () => containerRef,
        () => props.onClose()
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
                {/* Backdrop to capture clicks and prevent context menu on itself */}
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
                    ref={containerRef}
                    class="ui-context-menu-container"
                    style={{
                        top: `${coordinates().top}px`,
                        left: `${coordinates().left}px`,
                        opacity: isVisible() ? 1 : 0,
                        'transition-property': 'opacity',
                        'transition-duration': '150ms'
                    }}
                    onContextMenu={event => event.preventDefault()}
                >
                    <MenuList items={props.items} onClose={() => props.onClose()} />
                </div>
            </Portal>
        </Show>
    );
};
