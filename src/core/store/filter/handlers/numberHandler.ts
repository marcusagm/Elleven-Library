import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';

/**
 * Native Handler focused on fundamental arithmetic search fields.
 *
 * @module numberHandler
 * @description
 * Deals with validating boundaries for plain numerical values arrays constraints.
 */
export const numberHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the number criterion.
     *
     * @param {unknown} value - The primary value to validate.
     * @param {unknown} value2 - The secondary value to validate.
     * @param {string} operator - The operator to use for validation.
     * @returns {Record<string, string>} A map of field names to error messages. Returns an empty object if valid.
     */
    validate: (value, value2, operator) => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        if (operator === 'between') {
            if (checkIsEmpty(value2)) errors.value2 = 'End value is required';
            else if (!checkIsEmpty(value) && Number(value) > Number(value2)) {
                errors.value2 = 'End value must be greater than start';
            }
        }
        return errors;
    },

    /**
     * Processes the number criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @param {unknown} value2 - The secondary value to process.
     * @param {string} operator - The operator to use for processing.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: (value, value2, operator) => {
        if (operator === 'between') return { finalValue: [value, value2] };
        return { finalValue: value };
    },

    /**
     * Formats the number criterion into a displayable string.
     *
     * @param {unknown} startValue - The primary value to format.
     * @param {unknown} endValue - The secondary value to format.
     * @param {string} operator - The operator to use for formatting.
     * @returns {string} The formatted string.
     */
    formatDisplay: (startValue, endValue, operator) => {
        if (operator === 'between') return `${startValue} to ${endValue}`;
        return String(startValue);
    }
};
