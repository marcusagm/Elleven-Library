import type { ShortcutDefinition } from '../types';

/**
 * Default shortcut definitions for the application.
 */
export const DEFAULT_SHORTCUTS: ShortcutDefinition[] = [
    // Global scope
    {
        name: 'Focus Search',
        description: 'Focus the search input',
        keys: 'Meta+KeyK',
        scope: 'global',
        command: 'app:focus-search',
        category: 'Navigation',
        ignoreInputs: false,
        preventDefault: true
    },
    {
        name: 'Select All',
        description: 'Select all items',
        keys: 'Meta+KeyA',
        scope: 'global',
        command: 'app:select-all',
        category: 'Selection',
        ignoreInputs: true
    },
    {
        name: 'Deselect All',
        description: 'Clear selection',
        keys: 'Escape',
        scope: 'global',
        command: 'app:deselect-all',
        category: 'Selection',
        ignoreInputs: false
    },
    {
        name: 'Settings',
        description: 'Open application settings',
        keys: 'Meta+Comma',
        scope: 'global',
        command: 'app:settings',
        category: 'Application'
    },
    {
        name: 'Clear Search / Blur',
        description: 'Clear search query or blur input',
        keys: 'Escape',
        scope: 'search',
        command: 'search:clear',
        category: 'Search',
        ignoreInputs: false
    },

    // Viewport Interaction
    {
        name: 'Move Up',
        description: 'Navigate up in grid/list',
        keys: 'ArrowUp',
        scope: 'viewport',
        command: 'viewport:move-up',
        category: 'Navigation'
    },
    {
        name: 'Move Down',
        description: 'Navigate down in grid/list',
        keys: 'ArrowDown',
        scope: 'viewport',
        command: 'viewport:move-down',
        category: 'Navigation'
    },
    {
        name: 'Move Left',
        description: 'Navigate left in grid',
        keys: 'ArrowLeft',
        scope: 'viewport',
        command: 'viewport:move-left',
        category: 'Navigation'
    },
    {
        name: 'Move Right',
        description: 'Navigate right in grid',
        keys: 'ArrowRight',
        scope: 'viewport',
        command: 'viewport:move-right',
        category: 'Navigation'
    },
    {
        name: 'Go to Start',
        description: 'Navigate to first item',
        keys: 'Home',
        scope: 'viewport',
        command: 'viewport:home',
        category: 'Navigation'
    },
    {
        name: 'Go to End',
        description: 'Navigate to last item',
        keys: 'End',
        scope: 'viewport',
        command: 'viewport:end',
        category: 'Navigation'
    },
    {
        name: 'Toggle Selection',
        description: 'Select/deselect focused item',
        keys: 'Space',
        scope: 'viewport',
        command: 'viewport:toggle-select',
        category: 'Selection'
    },
    {
        name: 'Open Item',
        description: 'Open focused item',
        keys: 'Enter',
        scope: 'viewport',
        command: 'viewport:open',
        category: 'Navigation'
    },
    {
        name: 'Add to Selection',
        description: 'Add item to current selection',
        keys: 'Shift+Space',
        scope: 'viewport',
        command: 'viewport:select-add',
        category: 'Selection'
    },
    {
        name: 'Select All Items',
        description: 'Select all visible items',
        keys: 'Meta+KeyA',
        scope: 'viewport',
        command: 'viewport:select-all',
        category: 'Selection',
        ignoreInputs: true
    },

    // Image Viewer scope
    {
        name: 'Close Viewer',
        description: 'Close the image viewer',
        keys: 'Escape',
        scope: 'image-viewer',
        command: 'viewer:close',
        category: 'Viewer',
        ignoreInputs: false
    },
    {
        name: 'Zoom In',
        description: 'Increase zoom level',
        keys: 'Equal',
        scope: 'image-viewer',
        command: 'viewer:zoom-in',
        category: 'Viewer'
    },
    {
        name: 'Zoom Out',
        description: 'Decrease zoom level',
        keys: 'Minus',
        scope: 'image-viewer',
        command: 'viewer:zoom-out',
        category: 'Viewer'
    },
    {
        name: 'Fit to Screen',
        description: 'Fit image to screen',
        keys: 'Meta+Digit0',
        scope: 'image-viewer',
        command: 'viewer:fit-screen',
        category: 'Viewer'
    },
    {
        name: 'Original Size',
        description: 'Show image at 100% zoom',
        keys: 'Meta+Digit1',
        scope: 'image-viewer',
        command: 'viewer:original-size',
        category: 'Viewer'
    },
    {
        name: 'Pan Tool',
        description: 'Activate pan tool',
        keys: 'KeyH',
        scope: 'image-viewer',
        command: 'viewer:tool-pan',
        category: 'Viewer'
    },
    {
        name: 'Rotate Tool',
        description: 'Activate rotate tool',
        keys: 'KeyR',
        scope: 'image-viewer',
        command: 'viewer:tool-rotate',
        category: 'Viewer'
    },
    {
        name: 'Previous Item',
        description: 'Go to previous item',
        keys: 'ArrowLeft',
        scope: 'image-viewer',
        command: 'viewer:previous',
        category: 'Viewer'
    },
    {
        name: 'Next Item',
        description: 'Go to next item',
        keys: 'ArrowRight',
        scope: 'image-viewer',
        command: 'viewer:next',
        category: 'Viewer'
    },
    {
        name: 'Play/Pause Slideshow',
        description: 'Toggle slideshow playback',
        keys: 'Space',
        scope: 'image-viewer',
        command: 'viewer:slideshow-toggle',
        category: 'Viewer'
    },
    {
        name: 'Flip Horizontal',
        description: 'Flip image horizontally',
        keys: 'Shift+KeyH',
        scope: 'image-viewer',
        command: 'viewer:flip-h',
        category: 'Viewer'
    },
    {
        name: 'Flip Vertical',
        description: 'Flip image vertically',
        keys: 'Shift+KeyV',
        scope: 'image-viewer',
        command: 'viewer:flip-v',
        category: 'Viewer'
    },

    // Search Scope
    {
        name: 'Close Search',
        description: 'Close search or clear input',
        keys: 'Escape',
        scope: 'search',
        command: 'search:close',
        category: 'Search',
        ignoreInputs: false
    },
    {
        name: 'Next Result',
        description: 'Select next search result',
        keys: 'ArrowDown',
        scope: 'search',
        command: 'search:next',
        category: 'Search',
        ignoreInputs: false
    },
    {
        name: 'Previous Result',
        description: 'Select previous search result',
        keys: 'ArrowUp',
        scope: 'search',
        command: 'search:prev',
        category: 'Search',
        ignoreInputs: false
    },
    {
        name: 'Execute Result',
        description: 'Open selected result',
        keys: 'Enter',
        scope: 'search',
        command: 'search:exec',
        category: 'Search',
        ignoreInputs: false
    },

    // Modal scope
    {
        name: 'Close Modal',
        description: 'Close the active modal',
        keys: 'Escape',
        scope: 'modal',
        command: 'modal:close',
        category: 'Modal',
        ignoreInputs: false
    },
    {
        name: 'Confirm Modal',
        description: 'Confirm/Submit active modal',
        keys: 'Enter',
        scope: 'modal',
        command: 'modal:confirm',
        category: 'Modal',
        ignoreInputs: false
    }
];
