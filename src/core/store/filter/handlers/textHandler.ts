import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';

/**
 * Handler focused on arbitrary string search matching.
 *
 * @module textHandler
 * @description
 * Default generic handler bypassing heavy logic straight strictly to text evaluation.
 */
export const textHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the text criterion.
     *
     * @param {unknown} value - The primary value to validate.
     * @returns {Record<string, string>} A map of field names to error messages. Returns an empty object if valid.
     */
    validate: value => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        return errors;
    },

    /**
     * Processes the text criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @param {unknown} _value2 - The secondary value to process.
     * @param {string} operator - The operator to use for processing.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: value => ({ finalValue: value })
};
