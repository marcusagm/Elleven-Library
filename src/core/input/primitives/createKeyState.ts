/**
 * createKeyState Primitive
 * Reactive access to keyboard pressed state
 */

import { createMemo, Accessor } from 'solid-js';
import { inputStore } from '../store/inputStore';
import { canonicalizeShortcut } from '../normalizer';

/**
 * Get reactive pressed state for a specific key
 *
 * @example
 * const isShiftPressed = createKeyState('Shift');
 *
 * // In JSX
 * <Show when={isShiftPressed()}>
 *   <span>Shift is pressed!</span>
 * </Show>
 */
export function createKeyState(key: string): Accessor<boolean> {
    const normalizedKey = canonicalizeShortcut(key);

    const isKeyPressed = createMemo(() => {
        return inputStore.pressedKeys().has(normalizedKey);
    });
    return isKeyPressed;
}

/**
 * Get all currently pressed keys
 *
 * @example
 * const pressed = createPressedKeys();
 *
 * // Check if a modifier is pressed
 * const hasModifier = () =>
 *   pressed().has('Meta') || pressed().has('Ctrl') || pressed().has('Shift');
 */
export function createPressedKeys(): Accessor<Set<string>> {
    const pressedKeysMemo = createMemo(() => inputStore.pressedKeys());
    return pressedKeysMemo;
}

/**
 * Check if some of the given keys are pressed
 *
 * @example
 * const isModifierPressed = createAnyKeyPressed(['Meta', 'Ctrl']);
 */
export function createAnyKeyPressed(keys: string[]): Accessor<boolean> {
    const normalizedKeys = keys.map(keyName => canonicalizeShortcut(keyName));

    const anyPressed = createMemo(() => {
        const pressed = inputStore.pressedKeys();
        return normalizedKeys.some(keyName => pressed.has(keyName));
    });
    return anyPressed;
}

/**
 * Check if all of the given keys are pressed
 *
 * @example
 * const isCtrlShiftPressed = createAllKeysPressed(['Ctrl', 'Shift']);
 */
export function createAllKeysPressed(keys: string[]): Accessor<boolean> {
    const normalizedKeys = keys.map(keyName => canonicalizeShortcut(keyName));

    const allPressed = createMemo(() => {
        const pressed = inputStore.pressedKeys();
        return normalizedKeys.every(keyName => pressed.has(keyName));
    });
    return allPressed;
}
