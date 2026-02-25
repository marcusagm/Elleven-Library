import { JSX, onCleanup } from 'solid-js';
import { useInput, SCOPE_PRIORITIES } from '../../../core/input';

/**
 * Hook to manage input-specific events such as focus, blur, and keyboard navigation.
 * Handles integration with the global input scope system to prevent shortcut conflicts.
 *
 * @param htmlAttributes - The standard HTML attributes passed to the input component.
 * @returns An object containing event handlers for the input element.
 */
export const useInputEvents = (htmlAttributes: JSX.InputHTMLAttributes<HTMLInputElement>) => {
    const inputService = useInput();
    let isEditingScopeActive = false;

    /**
     * Handles the focus event on the input element.
     * Activates the 'editing' input scope to prevent global shortcuts from firing.
     *
     * @param event - The focus event object.
     */
    const handleFocus = (event: FocusEvent) => {
        if (!isEditingScopeActive) {
            inputService.pushScope('editing', SCOPE_PRIORITIES.editing, true);
            isEditingScopeActive = true;
        }

        const onFocusHandler = htmlAttributes.onFocus;
        if (typeof onFocusHandler === 'function') {
            // @ts-expect-error - SolidJS synthetic events have complex internal types that are difficult to match perfectly in a generic handler.
            // The runtime execution is safe as we are passing the native event object.
            onFocusHandler(event);
        }
    };

    /**
     * Handles the blur event on the input element.
     * Deactivates the 'editing' input scope.
     *
     * @param event - The blur event object.
     */
    const handleBlur = (event: FocusEvent) => {
        if (isEditingScopeActive) {
            inputService.popScope('editing');
            isEditingScopeActive = false;
        }

        const onBlurHandler = htmlAttributes.onBlur;
        if (typeof onBlurHandler === 'function') {
            // @ts-expect-error - SolidJS synthetic events have complex internal types
            onBlurHandler(event);
        }
    };

    /**
     * Handles keydown events to manage propagation and prevent system shortcut conflicts.
     *
     * @param event - The keyboard event object.
     */
    const handleKeyDown = (event: KeyboardEvent) => {
        // List of keys that are common shortcuts in the application
        // that must be trapped within the input field.
        const blockedKeysList = [
            'Enter',
            'ArrowUp',
            'ArrowDown',
            'ArrowLeft',
            'ArrowRight',
            ' ',
            'Home',
            'End'
        ];

        // Check for standard text editing shortcuts (Cmd/Ctrl + A, C, V, X, Z, etc.)
        // These should be handled by the browser's native input behavior and blocked
        // from bubbling up to the application's global shortcut system.
        const isModifierPressed = event.metaKey || event.ctrlKey;
        const isStandardEditingShortcut =
            isModifierPressed && ['a', 'c', 'v', 'x', 'z'].includes(event.key.toLowerCase());

        if (blockedKeysList.includes(event.key) || isStandardEditingShortcut) {
            event.stopPropagation();

            // Prevent default form submission or other browser behaviors for Enter
            if (event.key === 'Enter') {
                event.preventDefault();
            }
        }

        const onKeyDownHandler = htmlAttributes.onKeyDown;
        if (typeof onKeyDownHandler === 'function') {
            // @ts-expect-error - SolidJS synthetic events have complex internal types
            onKeyDownHandler(event);
        }
    };

    // Ensure the scope is popped if the component is unmounted while focused.
    onCleanup(() => {
        if (isEditingScopeActive) {
            inputService.popScope('editing');
        }
    });

    return {
        handleFocus,
        handleBlur,
        handleKeyDown
    };
};
