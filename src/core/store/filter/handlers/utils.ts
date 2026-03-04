/**
 * Helper strictly designed to check for empty values in search criteria contexts.
 *
 * @param {unknown} value - The testing variable, unknown to accept values coming dynamically from Inputs.
 * @returns {boolean} True if the value evaluates to an effective nullish sequence or an empty string.
 *
 * @example
 * ```tsx
 * const isInvalid = checkIsEmpty(""); // Returns true
 * ```
 */
export const checkIsEmpty = (value: unknown): boolean =>
    value === null || value === undefined || value === '';
