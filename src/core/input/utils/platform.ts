import type { Platform } from '../types';

let cachedPlatform: Platform | null = null;

/**
 * Detects the current operating system platform.
 *
 * @returns The detected platform ('mac', 'windows', or 'linux').
 */
export function detectPlatform(): Platform {
    if (cachedPlatform) return cachedPlatform;

    if (typeof navigator === 'undefined') {
        cachedPlatform = 'windows';
        return cachedPlatform;
    }

    const userAgent = navigator.userAgent.toLowerCase();

    if (userAgent.includes('mac')) {
        cachedPlatform = 'mac';
    } else if (userAgent.includes('linux')) {
        cachedPlatform = 'linux';
    } else {
        cachedPlatform = 'windows';
    }

    return cachedPlatform;
}

/**
 * Checks if the current platform is macOS.
 */
export function isMac(): boolean {
    return detectPlatform() === 'mac';
}
