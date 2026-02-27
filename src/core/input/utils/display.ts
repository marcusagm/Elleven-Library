import type { Platform } from '../types';
import { MAC_SYMBOLS, WINDOWS_SYMBOLS } from './keys';

/**
 * Formats a key name for user-facing display based on the platform.
 *
 * @param key - The normalized key code.
 * @param platform - The target platform for formatting.
 * @returns The formatted key string or symbol.
 */
export function formatKeyForDisplay(key: string, platform: Platform): string {
    const symbols = platform === 'mac' ? MAC_SYMBOLS : WINDOWS_SYMBOLS;

    if (symbols[key]) {
        return symbols[key];
    }

    // Convert KeyX to X, DigitX to X
    if (/^Key([A-Z])$/.test(key)) {
        return key.slice(3);
    }
    if (/^Digit([0-9])$/.test(key)) {
        return key.slice(5);
    }

    // Function keys
    if (/^F[0-9]+$/.test(key)) {
        return key;
    }

    // Special cases
    if (key === 'Equal') return '+';
    if (key === 'Minus') return '-';

    return key;
}
