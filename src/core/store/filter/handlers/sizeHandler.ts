import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';
import { SIZE_UNITS } from '../constants';

/**
 * System memory units Handler strictly focused on file weighting representations.
 *
 * @module sizeHandler
 * @description
 * Performs advanced validation dictating specific logic based on byte boundaries and formatting.
 */
export const sizeHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the size criterion.
     *
     * @param {unknown} value - The primary value to validate.
     * @param {unknown} value2 - The secondary value to validate.
     * @param {string} operator - The operator to use for validation.
     * @param {string} unitMultiplier - The unit multiplier to use for validation.
     * @returns {Record<string, string>} A map of field names to error messages. Returns an empty object if valid.
     */
    validate: (value, value2, operator, unitMultiplier) => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        if (operator === 'between') {
            if (checkIsEmpty(value2)) errors.value2 = 'End value is required';
            else if (!checkIsEmpty(value) && Number(value) > Number(value2)) {
                errors.value2 = 'End value must be greater than start';
            }
        }
        if (
            !SIZE_UNITS.find(
                (unitItem: { value: string; label: string }) => unitItem.value === unitMultiplier
            )
        ) {
            errors.unit = 'Unit is required';
        }
        return errors;
    },

    /**
     * Processes the size criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @param {unknown} value2 - The secondary value to process.
     * @param {string} operator - The operator to use for processing.
     * @param {string} unitMultiplier - The unit multiplier to use for processing.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: (value, value2, operator, unitMultiplier) => {
        const mathematicalMultiplier = Number(unitMultiplier);
        if (operator === 'between') {
            return {
                finalValue: [
                    Math.round(Number(value) * mathematicalMultiplier),
                    Math.round(Number(value2) * mathematicalMultiplier)
                ],
                unitMultiplier
            };
        }
        return { finalValue: Math.round(Number(value) * mathematicalMultiplier), unitMultiplier };
    },

    /**
     * Formats the size criterion into a displayable string.
     *
     * @param {unknown} startValue - The primary value to format.
     * @param {unknown} endValue - The secondary value to format.
     * @param {string} operator - The operator to use for formatting.
     * @param {string} unitMultiplier - The unit multiplier to use for formatting.
     * @returns {string} The formatted string.
     */
    formatDisplay: (startValue, endValue, operator, unitMultiplier) => {
        const mathematicalMultiplier = Number(unitMultiplier || '1048576');
        const visualLabel =
            SIZE_UNITS.find(
                (unitItem: { value: string; label: string }) => unitItem.value === unitMultiplier
            )?.label || 'bytes';

        const displayStartValue = Number(startValue) / mathematicalMultiplier;
        const displayEndValue = endValue ? Number(endValue) / mathematicalMultiplier : undefined;

        return operator === 'between'
            ? `${displayStartValue} ${visualLabel} to ${displayEndValue} ${visualLabel}`
            : `${displayStartValue} ${visualLabel}`;
    }
};
