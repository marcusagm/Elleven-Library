import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';

/**
 * Structural Handler strictly focused on system folder references mapping.
 *
 * @module folderHandler
 * @description
 * Intersects pure references of directories resolving visual outputs by reading
 * cross-store metadata references.
 */
export const folderHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the folder criterion.
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
     * Processes the folder criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: value => ({ finalValue: value }),

    /**
     * Formats the folder criterion into a displayable string.
     *
     * @param {unknown} folderId - The primary value to format.
     * @param {unknown} _v2 - The secondary value to format.
     * @param {string} _op - The operator to use for formatting.
     * @param {string} _unit - The unit to use for formatting.
     * @param {Object} metadata - The metadata to use for formatting.
     * @returns {string} The formatted string.
     */
    formatDisplay: (folderId, _v2, _op, _unit, metadata) => {
        const matchedFolder = metadata?.locations.find(l => String(l.id) === String(folderId));
        return matchedFolder?.name || String(folderId);
    }
};
