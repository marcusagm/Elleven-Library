import { Component, JSX, splitProps, createMemo, Show } from 'solid-js';
import { cn } from '../../lib/utils';
import { createControllableSignal } from '../../lib/primitives';
import { createId } from '../../lib/primitives/createId';
import { Check, Minus } from 'lucide-solid';
import './checkbox.css';

/**
 * Defines the available sizes for the Checkbox component.
 */
export type CheckboxSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the Checkbox component, extending standard HTML input attributes.
 */
export interface CheckboxProps extends Omit<
    JSX.InputHTMLAttributes<HTMLInputElement>,
    'onChange' | 'type'
> {
    /**
     * The checked state of the checkbox.
     * @default false
     */
    checked?: boolean;
    /**
     * The default checked state of the checkbox when not explicitly controlled.
     * @default false
     */
    defaultChecked?: boolean;
    /**
     * Whether the checkbox should be in an indeterminate state (partial selection).
     * @default false
     */
    indeterminate?: boolean;
    /**
     * Callback function invoked when the checked state changes.
     * @param checked - The new checked state.
     */
    onCheckedChange?: (checked: boolean) => void;
    /**
     * The label text to display next to the checkbox.
     */
    label?: string;
    /**
     * A description to display below the label for additional context.
     */
    description?: string;
    /**
     * The size variant of the checkbox.
     * @default 'md'
     */
    size?: CheckboxSize;
}

/**
 * Checkbox component for selecting multiple options.
 * Supports indeterminate state for partial selections.
 *
 * @example
 * <Checkbox label="Accept terms" />
 * <Checkbox checked indeterminate label="Select all" />
 */
export const Checkbox: Component<CheckboxProps> = props => {
    const [local, others] = splitProps(props, [
        'class',
        'checked',
        'defaultChecked',
        'indeterminate',
        'onCheckedChange',
        'label',
        'description',
        'size',
        'id',
        'disabled'
    ]);

    /**
     * Generates a unique identifier for the checkbox.
     */
    const id = createMemo(() => local.id || createId('checkbox'));
    /**
     * Generates a unique identifier for the checkbox description.
     */
    const descriptionId = createMemo(() => (local.description ? `${id()}-desc` : undefined));

    /**
     * Creates a controllable signal for the checkbox state.
     */
    const { value: isChecked, setValue: setChecked } = createControllableSignal({
        value: () => local.checked,
        defaultValue: local.defaultChecked ?? false,
        onChange: (checked: boolean) => local.onCheckedChange?.(checked)
    });

    /**
     * Handles the click event of the checkbox.
     */
    const handleClick = () => {
        if (local.disabled) return;
        setChecked(!isChecked());
    };

    /**
     * Handles the keydown event of the checkbox.
     */
    const handleKeyDown = (event: KeyboardEvent) => {
        if (local.disabled) return;
        if (event.key === ' ' || event.key === 'Enter') {
            event.preventDefault();
            setChecked(!isChecked());
        }
    };

    /**
     * Determines whether the check icon should be displayed.
     */
    const showCheck = () => isChecked() && !local.indeterminate;

    /**
     * Determines whether the indeterminate icon should be displayed.
     */
    const showIndeterminate = () => local.indeterminate;

    return (
        <label
            class={cn('ui-checkbox-wrapper', local.disabled && 'ui-checkbox-disabled', local.class)}
            for={id()}
        >
            <button
                type="button"
                role="checkbox"
                id={id()}
                class={cn(
                    'ui-checkbox',
                    `ui-checkbox-${local.size || 'md'}`,
                    (isChecked() || local.indeterminate) && 'ui-checkbox-checked'
                )}
                aria-checked={local.indeterminate ? 'mixed' : isChecked()}
                aria-disabled={local.disabled}
                aria-describedby={descriptionId()}
                disabled={local.disabled}
                onClick={handleClick}
                onKeyDown={handleKeyDown}
            >
                <Show when={showCheck()}>
                    <Check size={12} class="ui-checkbox-icon" />
                </Show>
                <Show when={showIndeterminate()}>
                    <Minus size={12} class="ui-checkbox-icon" />
                </Show>
            </button>

            <Show when={local.label || local.description}>
                <div class="ui-checkbox-content">
                    <Show when={local.label}>
                        <span
                            class={cn(
                                'ui-checkbox-label',
                                `ui-checkbox-label-${local.size || 'md'}`
                            )}
                        >
                            {local.label}
                        </span>
                    </Show>
                    <Show when={local.description}>
                        <span
                            id={descriptionId()}
                            class={cn(
                                'ui-checkbox-description',
                                `ui-checkbox-description-${local.size || 'md'}`
                            )}
                        >
                            {local.description}
                        </span>
                    </Show>
                </div>
            </Show>

            {/* Hidden input for form submission */}
            <input
                type="checkbox"
                checked={isChecked()}
                disabled={local.disabled}
                class="ui-checkbox-input"
                tabindex={-1}
                aria-hidden="true"
                {...others}
            />
        </label>
    );
};
