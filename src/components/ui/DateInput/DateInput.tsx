import { Component, createEffect, createSignal, splitProps, mergeProps, Show, JSX } from 'solid-js';
import { Calendar as CalendarIcon } from 'lucide-solid';
import { DatePicker } from '../DatePicker';
import { Popover } from '../Popover';
import { cn } from '../../../lib/utils';
import { DateInputProperties } from './types';
import { useInputEvents } from '../Input/useInputEvents';
import { formatDateToDisplay, parseDisplayDate } from '../../../utils/format';
import './date-input.css';

/**
 * A specialized Input component for date selection.
 * Combines a masked text input with a calendar dropdown for an enhanced user experience.
 *
 * @param properties - The reactive properties for the DateInput component.
 * @returns The rendered DateInput component interface.
 *
 * @example
 * <DateInput label="Birth Date" value={birthDate()} onChange={setBirthDate} />
 */
export const DateInput: Component<DateInputProperties> = properties => {
    // Provide default values and split properties for cleaner handling within the component.
    const mergedProperties = mergeProps({ size: 'md' as const }, properties);
    const [localProperties, htmlAttributes] = splitProps(mergedProperties, [
        'value',
        'defaultValue',
        'onChange',
        'class',
        'wrapperClass',
        'label',
        'error',
        'errorMessage',
        'size'
    ]);

    // Internal state for the masked string in the input field.
    const [inputValue, setInputValue] = createSignal('');
    const [isPopoverOpen, setIsPopoverOpen] = createSignal(false);

    // Integrates with the global input system to block shortcuts during editing.
    const { handleFocus, handleBlur, handleKeyDown } = useInputEvents(htmlAttributes);

    // Synchronize the input string with the reactive date value providing real-time feedback.
    createEffect(() => {
        const currentDate = localProperties.value ?? localProperties.defaultValue;
        if (currentDate instanceof Date && !isNaN(currentDate.getTime())) {
            setInputValue(formatDateToDisplay(currentDate));
        } else if (localProperties.value === null) {
            setInputValue('');
        }
    });

    /**
     * Handles the input event, applying the mask and updating the value accordingly.
     *
     * @param event - The generic input event from the text field.
     */
    const handleInput = (event: InputEvent & { currentTarget: HTMLInputElement }) => {
        const rawValue = event.currentTarget.value;
        const maskedValue = applyMask(rawValue);

        // Update the input field value directly to maintain mask effect visually for the user.
        if (maskedValue !== rawValue) {
            event.currentTarget.value = maskedValue;
        }
        setInputValue(maskedValue);

        const parsedDate = parseDisplayDate(maskedValue);
        if (parsedDate) {
            localProperties.onChange?.(parsedDate);
        } else if (maskedValue === '') {
            localProperties.onChange?.(null);
        }
    };

    /**
     * Handles date selection from the calendar picker and closes the popover.
     *
     * @param date - The Date object selected from the calendar.
     */
    const handleCalendarSelect = (date: Date) => {
        setInputValue(formatDateToDisplay(date));
        localProperties.onChange?.(date);
        setIsPopoverOpen(false);
    };
    return (
        <div class={cn('ui-date-input-wrapper', localProperties.wrapperClass)}>
            {localProperties.label && (
                <label class="ui-date-input-label">{localProperties.label}</label>
            )}

            <div
                class={cn(
                    'ui-date-input-container',
                    `ui-date-input-${localProperties.size}`,
                    localProperties.error && 'ui-date-input-error',
                    htmlAttributes.disabled && 'ui-date-input-disabled',
                    'ui-date-input-has-right',
                    localProperties.class
                )}
            >
                <input
                    type="text"
                    class="ui-date-input-field"
                    value={inputValue()}
                    onInput={handleInput}
                    onFocus={handleFocus}
                    onBlur={handleBlur}
                    onKeyDown={handleKeyDown}
                    disabled={htmlAttributes.disabled}
                    placeholder={htmlAttributes.placeholder || 'DD/MM/YYYY'}
                    aria-invalid={localProperties.error || undefined}
                    {...htmlAttributes}
                />

                <div class="ui-date-input-icon-right">
                    <DateInputTrigger
                        disabled={htmlAttributes.disabled}
                        isOpen={isPopoverOpen()}
                        onToggle={() => setIsPopoverOpen(!isPopoverOpen())}
                        onClose={() => setIsPopoverOpen(false)}
                        anchor={
                            <DatePicker
                                value={
                                    parseDisplayDate(inputValue()) ||
                                    (localProperties.value ?? new Date())
                                }
                                onChange={handleCalendarSelect}
                            />
                        }
                    />
                </div>
            </div>

            {localProperties.error && localProperties.errorMessage && (
                <span class="ui-date-input-error-message" role="alert">
                    {localProperties.errorMessage}
                </span>
            )}
        </div>
    );
};

/**
 * Applies a numeric mask (DD/MM/YYYY) to the raw input string.
 *
 * @param rawString - The raw value from the input.
 * @returns The masked value.
 */
function applyMask(rawString: string): string {
    const cleanString = rawString.replace(/\D/g, '');
    const maskPattern = '99/99/9999';
    let resultString = '';
    let cleanStringIndex = 0;

    for (
        let patternIndex = 0;
        patternIndex < maskPattern.length && cleanStringIndex < cleanString.length;
        patternIndex++
    ) {
        if (maskPattern[patternIndex] === '9') {
            resultString += cleanString[cleanStringIndex];
            cleanStringIndex++;
        } else {
            resultString += maskPattern[patternIndex];
        }
    }
    return resultString;
}

/**
 * Properties for the internal DateInputTrigger component.
 */
interface DateInputTriggerProperties {
    disabled?: boolean;
    isOpen: boolean;
    onToggle: () => void;
    onClose: () => void;
    anchor: JSX.Element;
}

/**
 * Internal component for the calendar trigger button and popover.
 *
 * @param properties - The reactive properties for the DateInputTrigger.
 */
const DateInputTrigger: Component<DateInputTriggerProperties> = properties => {
    return (
        <Show
            when={!properties.disabled}
            fallback={
                <button
                    type="button"
                    class="ui-date-input-trigger"
                    disabled={true}
                    aria-label="Open calendar"
                    tabIndex={-1}
                >
                    <CalendarIcon size={16} />
                </button>
            }
        >
            <Popover
                isOpen={properties.isOpen}
                onClose={() => properties.onClose()}
                trigger={
                    <button
                        type="button"
                        class="ui-date-input-trigger"
                        onClick={event => {
                            event.stopPropagation();
                            properties.onToggle();
                        }}
                        disabled={false}
                        aria-label="Open calendar"
                        tabIndex={-1}
                    >
                        <CalendarIcon size={16} />
                    </button>
                }
                class="ui-date-input-popover"
                align="end"
            >
                {properties.anchor}
            </Popover>
        </Show>
    );
};
