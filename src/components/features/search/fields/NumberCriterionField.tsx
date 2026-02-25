import { Component, Show } from 'solid-js';
import { NumberInput } from '../../../ui/NumberInput';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders an input group for generic numeric search criteria.
 * Supports single values and between-range values.
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the number field renderer.
 * @returns {JSX.Element} The rendered numeric input group.
 */
export const NumberCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    /** Checks if the current comparison logic expects a range of two numbers. */
    const isRangeMode = () => properties.comparisonOperator === 'between';

    return (
        <div class="number-input-group">
            <NumberInput
                size={properties.size || 'md'}
                value={(properties.value as number) ?? undefined}
                onChange={value => properties.setValue(value ?? null)}
                placeholder={isRangeMode() ? 'From...' : 'Value...'}
                error={!!properties.errors.value}
                errorMessage={properties.errors.value}
            />
            <Show when={isRangeMode() && properties.setValue2}>
                <span class="range-separator">to</span>
                <NumberInput
                    size={properties.size || 'md'}
                    value={(properties.value2 as number) ?? undefined}
                    onChange={value => properties.setValue2?.(value ?? null)}
                    placeholder="To..."
                    error={!!properties.errors.value2}
                    errorMessage={properties.errors.value2}
                />
            </Show>
        </div>
    );
};

/**
 * Handler implementation for generic numeric search criteria.
 */
export const numberHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the numeric inputs. */
    component: NumberCriterionField,

    /**
     * Validates that numbers are provided and range logic is sound.
     *
     * @param value - Primary numeric selection.
     * @param value2 - Secondary numeric selection (for range).
     * @param operator - Logic operator (e.g., 'equal', 'between').
     * @returns A map of validation error messages.
     */
    validate: (value, value2, operator) => {
        const validationErrors: Record<string, string> = {};
        if (value === null || value === '') {
            validationErrors.value = 'Value is required';
        }

        if (operator === 'between') {
            if (value2 === null || value2 === '') {
                validationErrors.value2 = 'End value is required';
            } else if (value !== null && value !== '') {
                if (Number(value) > Number(value2)) {
                    validationErrors.value2 = 'End value must be greater than start';
                }
            }
        }
        return validationErrors;
    },

    /**
     * Processes numeric values for storage/query logic.
     *
     * @param value - Primary raw input.
     * @param value2 - Secondary raw input.
     * @param operator - Logic operator.
     * @returns The final query value or range array.
     */
    process: (value, value2, operator) => {
        if (operator === 'between') {
            return { finalValue: [value, value2] };
        }
        return { finalValue: value };
    }
};
