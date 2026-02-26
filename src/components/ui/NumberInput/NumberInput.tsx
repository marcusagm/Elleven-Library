import { Component, splitProps, mergeProps } from 'solid-js';
import { Input } from '../Input/Input';
import { Minus, Plus } from 'lucide-solid';
import { cn } from '../../../lib/utils';
import { createControllableSignal } from '../../../lib/primitives';
import { NumberInputProps } from './types';
import './number-input.css';

/**
 * A specialized Input component for numeric values.
 * Provides controls for incrementing/decrementing and enforces numeric validation.
 *
 * @param props - Properties for the NumberInput component.
 * @returns The rendered NumberInput component.
 *
 * @example
 * <NumberInput value={count()} onChange={setCount} min={0} max={100} step={5} />
 */
export const NumberInput: Component<NumberInputProps> = props => {
    // Merge default properties to ensure defined boundaries.
    const mergedProperties = mergeProps(
        {
            step: 1,
            min: -Infinity,
            max: Infinity
        },
        props
    );

    // Separate specialized number logic properties from standard input attributes.
    // We avoid abbreviations to maintain descriptive code.
    const [localComponentProperties, remainingHtmlAttributes] = splitProps(mergedProperties, [
        'value',
        'defaultValue',
        'min',
        'max',
        'step',
        'onChange',
        'format',
        'class',
        'disabled',
        'leftIcon',
        'rightIcon'
    ]);

    // Manage a controllable signal that supports both controlled and uncontrolled states.
    const { value: numericValue, setValue: setNumericValue } = createControllableSignal<
        number | undefined
    >({
        value: () => localComponentProperties.value,
        defaultValue: localComponentProperties.defaultValue,
        onChange: value => localComponentProperties.onChange?.(value)
    });

    /**
     * Handles the input event to parse and validate the numeric value.
     *
     * @param event - The input event from the HTML input element.
     */
    const handleInput = (event: InputEvent & { currentTarget: HTMLInputElement }) => {
        const inputValue = event.currentTarget.value;

        if (inputValue === '') {
            setNumericValue(undefined);
            return;
        }

        const parsedNumber = parseFloat(inputValue);
        if (!isNaN(parsedNumber)) {
            setNumericValue(parsedNumber);
        }
    };

    /**
     * Filters keystrokes to ensure only valid numeric characters are entered.
     * Integrates with the core system through the Base Input's event handling.
     *
     * @param event - The keyboard event object.
     */
    const handleKeyDown = (event: KeyboardEvent) => {
        // List of allowed navigation and control keys.
        const allowedNavigationKeys = [
            'Backspace',
            'Delete',
            'Tab',
            'Escape',
            'Enter',
            'ArrowLeft',
            'ArrowRight',
            'ArrowUp',
            'ArrowDown',
            'Home',
            'End',
            '-',
            '.'
        ];

        if (allowedNavigationKeys.includes(event.key)) {
            return;
        }

        // Allow system shortcuts like Copy/Paste
        if (event.ctrlKey || event.metaKey) {
            return;
        }

        // Block any character that is not a digit.
        if (!/^[0-9]$/.test(event.key)) {
            event.preventDefault();
        }
    };

    /**
     * Increases the current value by the defined step, respecting the maximum limit.
     */
    const handleIncrement = () => {
        if (localComponentProperties.disabled) {
            return;
        }

        const currentValue = numericValue() ?? 0;
        const nextValue = Math.min(
            localComponentProperties.max,
            currentValue + localComponentProperties.step
        );
        setNumericValue(nextValue);
    };

    /**
     * Decreases the current value by the defined step, respecting the minimum limit.
     */
    const handleDecrement = () => {
        if (localComponentProperties.disabled) {
            return;
        }

        const currentValue = numericValue() ?? 0;
        const nextValue = Math.max(
            localComponentProperties.min,
            currentValue - localComponentProperties.step
        );
        setNumericValue(nextValue);
    };

    /**
     * Resolves the display value, applying the optional format function if provided.
     */
    const resolvedDisplayValue = () => {
        const value = numericValue();
        if (value === undefined) {
            return '';
        }
        return localComponentProperties.format ? localComponentProperties.format(value) : value;
    };

    return (
        <Input
            type="number"
            class={cn('ui-number-input', localComponentProperties.class)}
            value={resolvedDisplayValue()}
            min={localComponentProperties.min}
            max={localComponentProperties.max}
            step={localComponentProperties.step}
            disabled={localComponentProperties.disabled}
            onInput={handleInput}
            onKeyDown={handleKeyDown}
            leftIcon={
                <button
                    type="button"
                    class="ui-number-input-button"
                    onClick={handleDecrement}
                    disabled={
                        localComponentProperties.disabled ||
                        (numericValue() !== undefined &&
                            numericValue()! <= localComponentProperties.min)
                    }
                    tabIndex={-1}
                    aria-label="Decrease value"
                >
                    <Minus size={14} />
                </button>
            }
            rightIcon={
                <button
                    type="button"
                    class="ui-number-input-button"
                    onClick={handleIncrement}
                    disabled={
                        localComponentProperties.disabled ||
                        (numericValue() !== undefined &&
                            numericValue()! >= localComponentProperties.max)
                    }
                    tabIndex={-1}
                    aria-label="Increase value"
                >
                    <Plus size={14} />
                </button>
            }
            {...remainingHtmlAttributes}
        />
    );
};
