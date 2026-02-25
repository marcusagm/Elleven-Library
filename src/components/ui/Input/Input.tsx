import { Component, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { InputProps } from './types';
import { useInputEvents } from './useInputEvents';
import { InputLabel } from './InputLabel';
import { InputIcon } from './InputIcon';
import { InputErrorMessage } from './InputErrorMessage';
import './input.css';

/**
 * A highly customizable Input component that follows the Mundam design system.
 * Supports labels, icons, various sizes, and validation error states.
 * Integrates with the core input system for keyboard shortcut safety.
 *
 * @param props - Properties for the Input component.
 * @returns The rendered Input component.
 *
 * @example
 * // Basic usage with placeholder
 * <Input placeholder="Type something..." />
 *
 * @example
 * // With label and left icon
 * <Input label="Search" leftIcon={<SearchIcon />} placeholder="Find items..." />
 *
 * @example
 * // With error state
 * <Input error errorMessage="Field is required" />
 */
export const Input: Component<InputProps> = props => {
    // Separate specialized component properties from standard HTML input attributes.
    // We avoid abbreviations to comply with naming guidelines.
    const [inputComponentProperties, inputHtmlAttributes] = splitProps(props, [
        'label',
        'leftIcon',
        'rightIcon',
        'size',
        'error',
        'errorMessage',
        'wrapperClass',
        'class'
    ]);

    // Use our custom hook to manage specialized input events and scopes.
    const { handleFocus, handleBlur, handleKeyDown } = useInputEvents(inputHtmlAttributes);

    /**
     * Resolves the size variant, defaulting to 'md' if not specified.
     */
    const resolvedSize = () => inputComponentProperties.size || 'md';

    return (
        <div class={cn('ui-input-wrapper', inputComponentProperties.wrapperClass)}>
            <InputLabel text={inputComponentProperties.label} />

            <div
                class={cn(
                    'ui-input-container',
                    `ui-input-${resolvedSize()}`,
                    inputComponentProperties.error && 'ui-input-error',
                    inputHtmlAttributes.disabled && 'ui-input-disabled',
                    !!inputComponentProperties.leftIcon && 'ui-input-has-left',
                    !!inputComponentProperties.rightIcon && 'ui-input-has-right'
                )}
            >
                <InputIcon icon={inputComponentProperties.leftIcon} position="left" />

                <input
                    class={cn('ui-input', inputComponentProperties.class)}
                    aria-invalid={inputComponentProperties.error || undefined}
                    aria-describedby={
                        inputComponentProperties.error && inputComponentProperties.errorMessage
                            ? `${inputHtmlAttributes.id}-error`
                            : undefined
                    }
                    {...inputHtmlAttributes}
                    onFocus={handleFocus}
                    onBlur={handleBlur}
                    onKeyDown={handleKeyDown}
                />

                <InputIcon icon={inputComponentProperties.rightIcon} position="right" />
            </div>

            <InputErrorMessage
                show={!!inputComponentProperties.error}
                message={inputComponentProperties.errorMessage}
                inputId={inputHtmlAttributes.id}
            />
        </div>
    );
};
