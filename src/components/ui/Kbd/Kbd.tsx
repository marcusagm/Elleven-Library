import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import './kbd.css';

/**
 * Properties for the Kbd (Keyboard key) component.
 */
export interface KbdProperties extends JSX.HTMLAttributes<HTMLElement> {
    /** The content to be displayed within the keyboard key indicator. */
    children: JSX.Element;
}

/**
 * Keyboard key indicator component.
 * Displays keyboard shortcuts and individual keys in a visually distinct, semantic style.
 *
 * @param {KbdProperties} properties - Properties for the Kbd component.
 * @returns {JSX.Element} The rendered kbd element.
 *
 * @example
 * <Kbd>⌘</Kbd>
 *
 * @example
 * <span>Press <Kbd>Ctrl</Kbd> + <Kbd>C</Kbd> to copy.</span>
 */
export const Kbd: Component<KbdProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, ['class', 'children']);

    return (
        <kbd class={cn('ui-kbd', localProperties.class)} {...remainingProperties}>
            {localProperties.children}
        </kbd>
    );
};
