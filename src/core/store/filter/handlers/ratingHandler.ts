import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';

/**
 * Dedicated Handler for numerical scores with tailored display formatting.
 *
 * @module ratingHandler
 * @description
 * Provides direct format validations stringifying output explicitly as "Stars" units.
 */
export const ratingHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the rating criterion.
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
     * Processes the rating criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: value => ({ finalValue: value }),

    /**
     * Formats the rating criterion into a displayable string.
     *
     * @param {unknown} ratingCount - The primary value to format.
     * @returns {string} The formatted string.
     */
    formatDisplay: ratingCount => `${ratingCount} Stars`
};
