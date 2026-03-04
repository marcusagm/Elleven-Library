import { StoreMetadata } from '../../../../components/features/search/fields/types';

/**
 * Basic search value types including string, number, null, or Date.
 */
export type SearchValue = string | number | null | Date;

/**
 * Interface defining the business logic for validating, processing, and formatting
 * a specific search field type. This core handler is decoupled from any UI component.
 */
export interface CoreCriterionHandler {
    /**
     * Validates the inputs for the search criterion.
     *
     * @param {SearchValue} value - The primary value of the search field.
     * @param {SearchValue} value2 - The secondary value (e.g., end boundary for "between" operations).
     * @param {string} operator - The logical operator (e.g., "eq", "between", "contains").
     * @param {string} [unitMultiplier] - The unit associated with the value, if applicable.
     * @returns {Record<string, string>} A map of field names to error messages. Returns an empty object if valid.
     */
    validate: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unitMultiplier?: string
    ) => Record<string, string>;

    /**
     * Processes raw search inputs into database-friendly types.
     *
     * @param {SearchValue} value - The primary value to process.
     * @param {SearchValue} value2 - The secondary value to process.
     * @param {string} operator - The operational context (e.g., "between" results in an array).
     * @param {string} [unitMultiplier] - The unit multiplier, if applicable.
     * @returns {Object} An object containing the highly structured `finalValue` and optional processed `unitMultiplier`.
     */
    process: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unitMultiplier?: string
    ) => {
        finalValue: unknown;
        unitMultiplier?: string;
    };

    /**
     * Optional method to format highly technical values back into human-readable strings.
     *
     * @param {unknown} value - The primary processed value.
     * @param {unknown} value2 - The secondary processed value.
     * @param {string} operator - The operator acting upon the values.
     * @param {string} [unitMultiplier] - The localized/unit multiplier token to represent the unit.
     * @param {StoreMetadata} [metadata] - Extra metadata required for formatting (e.g. tag dictionary).
     * @returns {string} The fully readable string representation of the criterion criteria.
     */
    formatDisplay?: (
        value: unknown,
        value2: unknown,
        operator: string,
        unitMultiplier?: string,
        metadata?: StoreMetadata
    ) => string;
}
