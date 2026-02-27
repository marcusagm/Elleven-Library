import { Component, Show } from 'solid-js';
import { NumberInput } from '../../../ui/NumberInput';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProperties } from './types';
import { SIZE_UNITS } from '../../../../core/store/filter/constants';

/**
 * Renders a specialized input group for file size criteria.
 * Includes one or two number inputs (depending on the operator) and a unit selector (B, KB, MB, GB).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the size field renderer.
 * @returns {JSX.Element} The rendered size input group.
 */
export const SizeCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    /** Checks if the current comparison logic expects a range of two sizes. */
    const isRangeMode = () => properties.comparisonOperator === 'between';

    return (
        <div class="number-input-group">
            <NumberInput
                size={properties.size || 'md'}
                value={(properties.value as number) ?? undefined}
                onChange={value => properties.setValue(value ?? null)}
                placeholder={isRangeMode() ? 'From Size...' : 'Size Value...'}
                error={!!properties.errors.value}
                errorMessage={properties.errors.value}
            />
            <Show when={isRangeMode() && properties.setValue2}>
                <span class="range-separator">to</span>
                <NumberInput
                    size={properties.size || 'md'}
                    value={(properties.value2 as number) ?? undefined}
                    onChange={value => properties.setValue2?.(value ?? null)}
                    placeholder="To Size..."
                    error={!!properties.errors.value2}
                    errorMessage={properties.errors.value2}
                />
            </Show>
            <Show when={properties.setUnitMultiplier}>
                <Select
                    size={properties.size || 'md'}
                    options={SIZE_UNITS}
                    value={properties.unitMultiplier || '1048576'}
                    onValueChange={value => properties.setUnitMultiplier?.(value)}
                    error={!!properties.errors.unit}
                    errorMessage={properties.errors.unit}
                    class="unit-select"
                />
            </Show>
        </div>
    );
};

/**
 * Checks if a given input value is considered empty (null, undefined, or empty string).
 * @param value - The value to check.
 */
const checkIsEmpty = (value: unknown) => value === null || value === undefined || value === '';

/**
 * Handler implementation for file size-based search criteria.
 * Handles validation of numeric inputs, conversion between display units and bytes,
 * and formatting for human-readable output.
 */
export const sizeHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the size inputs and unit selector. */
    component: SizeCriterionField,

    /**
     * Validates that size values are provided and range consistency is maintained.
     *
     * @param value - Primary size value selection.
     * @param value2 - Secondary size value selection.
     * @param operator - The comparison logic being used.
     * @param unitMultiplier - The selected unit multiplier string.
     * @returns A record of validation error messages.
     */
    validate: (value, value2, operator, unitMultiplier) => {
        const validationErrors: Record<string, string> = {};

        if (checkIsEmpty(value)) {
            validationErrors.value = 'Value is required';
        }

        if (operator === 'between') {
            if (checkIsEmpty(value2)) {
                validationErrors.value2 = 'End value is required';
            } else if (!checkIsEmpty(value) && Number(value) > Number(value2)) {
                validationErrors.value2 = 'End value must be greater than start';
            }
        }

        if (
            !SIZE_UNITS.find(
                (option: { value: string; label: string }) => option.value === unitMultiplier
            )
        ) {
            validationErrors.unit = 'Unit is required';
        }

        return validationErrors;
    },

    /**
     * Converts the UI-level numeric inputs and units into raw byte counts for database queries.
     *
     * @param value - Primary raw size input.
     * @param value2 - Secondary raw size input.
     * @param operator - Current comparison operator.
     * @param unitMultiplier - The selected unit multiplier.
     * @returns The final processed byte value (or range) and the unit used.
     */
    process: (value, value2, operator, unitMultiplier) => {
        const numericMultiplier = Number(unitMultiplier);
        let finalValue: unknown;

        if (operator === 'between') {
            const startBytes = Math.round(Number(value) * numericMultiplier);
            const endBytes = Math.round(Number(value2) * numericMultiplier);
            finalValue = [startBytes, endBytes];
        } else {
            finalValue = Math.round(Number(value) * numericMultiplier);
        }

        return { finalValue, unitMultiplier };
    },

    /**
     * Formats the byte values back into display-friendly strings based on the selected unit.
     *
     * @param value1 - Primary byte count (or UI display value before multiplier).
     * @param value2 - Secondary byte count (or UI display value before multiplier).
     * @param operator - Comparison logic used.
     * @param unitMultiplier - The unit multiplier for resolving the correct label.
     * @returns A friendly string (e.g., "500 MB to 1 GB").
     */
    formatDisplay: (value1, value2, operator, unitMultiplier) => {
        const unitLabel =
            SIZE_UNITS.find(
                (option: { value: string; label: string }) => option.value === unitMultiplier
            )?.label || 'bytes';
        if (operator === 'between') {
            return `${value1} ${unitLabel} to ${value2} ${unitLabel}`;
        }
        return `${value1} ${unitLabel}`;
    }
};
