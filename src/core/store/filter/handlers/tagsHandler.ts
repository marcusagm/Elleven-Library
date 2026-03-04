import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';

/**
 * Structural Handler focused on specific dynamic tagging associations.
 *
 * @module tagsHandler
 * @description
 * Intersects abstract tags keys resolving literal names via application shared metadata.
 */
export const tagsHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the tags criterion.
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
     * Processes the tags criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: value => ({ finalValue: value }),

    /**
     * Formats the tags criterion into a displayable string.
     *
     * @param {unknown} tagId - The primary value to format.
     * @param {unknown} _v2 - The secondary value to format.
     * @param {string} _op - The operator to use for formatting.
     * @param {string} _unit - The unit to use for formatting.
     * @param {Object} metadata - The metadata to use for formatting.
     * @returns {string} The formatted string.
     */
    formatDisplay: (tagId, _v2, _op, _unit, metadata) => {
        const matched = metadata?.tags.find(tagItem => String(tagItem.id) === String(tagId));
        return matched?.name || String(tagId);
    }
};
