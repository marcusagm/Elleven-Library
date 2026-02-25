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
 * Combines a masked text input with a calendar dropdown.
 *
 * @param props - Properties for the DateInput component.
 * @returns The rendered DateInput component.
 *
 * @example
 * <DateInput label="Birth Date" value={birthDate()} onChange={setBirthDate} />
 */
export const DateInput: Component<DateInputProperties> = props => {
    // Provide default values and split props for cleaner handling.
    const mergedProperties = mergeProps({ size: 'md' as const }, props);
    const [local, htmlAttributes] = splitProps(mergedProperties, [
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

    // Synchronize the input string with the reactive date value.
    createEffect(() => {
        const currentDate = local.value ?? local.defaultValue;
        if (currentDate instanceof Date && !isNaN(currentDate.getTime())) {
            setInputValue(formatDateToDisplay(currentDate));
        } else if (local.value === null) {
            setInputValue('');
        }
    });

    /**
     * Handles the input event, applying the mask and updating the value.
     */
    const handleInput = (event: InputEvent & { currentTarget: HTMLInputElement }) => {
        const rawValue = event.currentTarget.value;
        const maskedValue = applyMask(rawValue);

        // Update the input field value directly to maintain mask effect
        if (maskedValue !== rawValue) {
            event.currentTarget.value = maskedValue;
        }
        setInputValue(maskedValue);

        const parsedDate = parseDisplayDate(maskedValue);
        if (parsedDate) {
            local.onChange?.(parsedDate);
        } else if (maskedValue === '') {
            local.onChange?.(null);
        }
    };

    /**
     * Handles date selection from the calendar picker.
     */
    const handleCalendarSelect = (date: Date) => {
        setInputValue(formatDateToDisplay(date));
        local.onChange?.(date);
        setIsPopoverOpen(false);
    };
    return (
        <div class={cn('ui-date-input-wrapper', local.wrapperClass)}>
            {local.label && <label class="ui-date-input-label">{local.label}</label>}

            <div
                class={cn(
                    'ui-date-input-container',
                    `ui-date-input-${local.size}`,
                    local.error && 'ui-date-input-error',
                    htmlAttributes.disabled && 'ui-date-input-disabled',
                    'ui-date-input-has-right',
                    local.class
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
                    aria-invalid={local.error || undefined}
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
                                    parseDisplayDate(inputValue()) || (local.value ?? new Date())
                                }
                                onChange={handleCalendarSelect}
                            />
                        }
                    />
                </div>
            </div>

            {local.error && local.errorMessage && (
                <span class="ui-date-input-error-message" role="alert">
                    {local.errorMessage}
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

    for (let i = 0; i < maskPattern.length && cleanStringIndex < cleanString.length; i++) {
        if (maskPattern[i] === '9') {
            resultString += cleanString[cleanStringIndex];
            cleanStringIndex++;
        } else {
            resultString += maskPattern[i];
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
 */
const DateInputTrigger: Component<DateInputTriggerProperties> = props => {
    return (
        <Show
            when={!props.disabled}
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
                isOpen={props.isOpen}
                onClose={() => props.onClose()}
                trigger={
                    <button
                        type="button"
                        class="ui-date-input-trigger"
                        onClick={event => {
                            event.stopPropagation();
                            props.onToggle();
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
                {props.anchor}
            </Popover>
        </Show>
    );
};
