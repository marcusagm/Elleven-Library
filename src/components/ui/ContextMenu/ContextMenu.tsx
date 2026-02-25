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
 * @param props - Properties for the context menu.
 * @returns The rendered context menu.
 */
export const ContextMenu: Component<ContextMenuProps> = props => {
    /** Target ref for the container for positioning and clicking outside. */
    let containerRef: HTMLDivElement | undefined;
    /** Current calculated placement. */
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

        // Initially hide to measure
        containerRef.style.top = `${props.y}px`;
        containerRef.style.left = `${props.x}px`;

        requestAnimationFrame(() => {
            if (!containerRef) return;

            const rect = containerRef.getBoundingClientRect();
            const viewportWidth = window.innerWidth;
            const viewportHeight = window.innerHeight;

            let top = props.y;
            let left = props.x;

            // Collision detection for right boundary
            if (left + rect.width > viewportWidth) {
                left = Math.max(0, viewportWidth - rect.width - 8);
            }
            // Collision detection for bottom boundary
            if (top + rect.height > viewportHeight) {
                top = Math.max(0, viewportHeight - rect.height - 8);
            }

            setCoordinates({ top, left });
            setIsVisible(true);
        });
    });

    /**
     * Click outside detection.
     */
    createClickOutside(
        () => containerRef,
        () => props.onClose()
    );

    /**
     * Listen for Escape key at the document level while open.
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
