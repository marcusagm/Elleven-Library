import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';

/**
 * Abstract Select Handler usually focused on enumerators like File Extensions logic.
 *
 * @module selectHandler
 * @description
 * Takes metadata mapping from extension parameters array rendering it into visual aliases.
 */
export const selectHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the select criterion.
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
     * Processes the select criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: value => ({ finalValue: value }),

    /**
     * Formats the select criterion into a displayable string.
     *
     * @param {unknown} val - The primary value to format.
     * @param {unknown} _v2 - The secondary value to format.
     * @param {string} _op - The operator to use for formatting.
     * @param {string} _unit - The unit to use for formatting.
     * @param {Object} metadata - The metadata to use for formatting.
     * @returns {string} The formatted string.
     */
    formatDisplay: (val, _v2, _op, _unit, metadata) => {
        const found = metadata?.supportedFormats?.find(f => f.extensions.includes(String(val)));
        return found ? `.${String(val).toUpperCase()} (${found.name})` : String(val);
    }
};
