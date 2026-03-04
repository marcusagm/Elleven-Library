import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';
import { formatToISO, formatToDisplay } from '../../../../utils/format';

/**
 * Standard Handler strictly focused on Temporal properties.
 *
 * @module dateHandler
 * @description
 * Deals with ISO formatting and time series range conversions logic.
 */
export const dateHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the date criterion.
     *
     * @param {unknown} value - The primary value to validate.
     * @param {unknown} value2 - The secondary value to validate.
     * @param {string} operator - The operator to use for validation.
     * @returns {Record<string, string>} A map of field names to error messages. Returns an empty object if valid.
     */
    validate: (value, value2, operator) => {
        const validationErrors: Record<string, string> = {};
        if (checkIsEmpty(value)) {
            validationErrors.value = 'Date is required';
        }

        if (operator === 'between') {
            if (checkIsEmpty(value2)) {
                validationErrors.value2 = 'End date is required';
            } else if (!checkIsEmpty(value)) {
                const startDateObject = new Date(value as string | Date);
                const endDateObject = new Date(value2 as string | Date);
                if (startDateObject > endDateObject) {
                    validationErrors.value2 = 'End date must be after start date';
                }
            }
        }
        return validationErrors;
    },

    /**
     * Processes the date criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @param {unknown} value2 - The secondary value to process.
     * @param {string} operator - The operator to use for processing.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: (value, value2, operator) => {
        if (operator === 'between') {
            return {
                finalValue: [
                    formatToISO(value as Date | string),
                    formatToISO(value2 as Date | string)
                ]
            };
        }
        return { finalValue: formatToISO(value as Date | string) };
    },

    /**
     * Formats the date criterion into a displayable string.
     *
     * @param {unknown} value1 - The primary value to format.
     * @param {unknown} value2 - The secondary value to format.
     * @param {string} operator - The operator to use for formatting.
     * @returns {string} The formatted string.
     */
    formatDisplay: (value1, value2, operator) => {
        if (operator === 'between') {
            return `${formatToDisplay(value1 as string | Date)} to ${formatToDisplay(value2 as string | Date)}`;
        }
        return formatToDisplay(value1 as string | Date);
    }
};
