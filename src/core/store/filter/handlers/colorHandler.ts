import { CoreCriterionHandler } from './types';
import { checkIsEmpty } from './utils';

/** Maximum ΔE for strict color matching. */
const DELTA_E_EXACT = 2.3;
/** Maximum ΔE for broad color family matching. */
const DELTA_E_BROAD = 50;

/**
 * Maps a tolerance percentage to a strict ΔE threshold value.
 *
 * @param {number} percentage - The visual percentage limit (0 to 100).
 * @returns {number} The corresponding mathematical ΔE constraint.
 */
function calculateProximityThreshold(percentage: number): number {
    return DELTA_E_EXACT + (percentage / 100) * (DELTA_E_BROAD - DELTA_E_EXACT);
}

/**
 * Translates a given mapped percentage back into a human-readable match level.
 *
 * @param {number} percentage - Proximity percentage value from the state.
 * @returns {string} Standardized semantic label representing proximity format.
 */
function getMatchLabel(percentage: number): string {
    if (percentage === 0) return 'Exact';
    if (percentage <= 25) return 'Very Similar';
    if (percentage <= 50) return 'Similar';
    if (percentage <= 75) return 'Related';
    return 'Broad';
}

/**
 * Advanced Handler strictly focused on mathematical Color (LAB) validation and processing.
 *
 * @module colorHandler
 * @description
 * Contains explicit rules on dealing with delta correlations (∆E), ensuring JSON
 * payloads representing a color hex + proximities are properly converted to metric bounds.
 */
export const colorHandler: CoreCriterionHandler = {
    /**
     * Validates the inputs for the color criterion.
     *
     * @param {unknown} value - The primary value to validate.
     * @returns {Record<string, string>} A map of field names to error messages. Returns an empty object if valid.
     */
    validate: value => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) {
            errors.value = 'Color is required';
            return errors;
        }
        let parsed: { hex?: string };
        try {
            parsed = typeof value === 'string' ? JSON.parse(value) : {};
        } catch {
            errors.value = 'Invalid color value';
            return errors;
        }
        if (!parsed.hex || !/^#[0-9A-Fa-f]{6}$/.test(parsed.hex)) {
            errors.value = 'Invalid hex color';
        }
        return errors;
    },

    /**
     * Processes the color criterion into a final value.
     *
     * @param {unknown} value - The primary value to process.
     * @param {unknown} _value2 - The secondary value to process.
     * @param {string} operator - The operator to use for processing.
     * @returns {Object} An object containing the final value and optional unit multiplier.
     */
    process: (value, _value2, operator) => {
        if (typeof value !== 'string') {
            return { finalValue: { hex: '#000000', threshold: 25 } };
        }
        let parsed: { hex: string; proximity: number };
        try {
            parsed = JSON.parse(value);
        } catch {
            return { finalValue: { hex: '#000000', threshold: 25 } };
        }
        const threshold =
            operator === 'exact'
                ? DELTA_E_EXACT
                : calculateProximityThreshold(parsed.proximity ?? 50);
        return {
            finalValue: {
                hex: parsed.hex,
                threshold: Math.round(threshold * 10) / 10
            }
        };
    },

    /**
     * Formats the color criterion into a displayable string.
     *
     * @param {unknown} value - The primary value to format.
     * @param {unknown} _value2 - The secondary value to format.
     * @param {string} operator - The operator to use for formatting.
     * @returns {string} The formatted string.
     */
    formatDisplay: (value, _value2, operator) => {
        try {
            const parsed =
                typeof value === 'string'
                    ? JSON.parse(value as string)
                    : (value as Record<string, unknown>);
            const hex = (parsed.hex as string) ?? '#000000';

            if (operator === 'exact') {
                return `${hex} (Exact)`;
            }

            const threshold = (parsed.threshold as number) ?? 25;
            const proximity = Math.max(
                0,
                Math.min(
                    100,
                    Math.round(
                        ((threshold - DELTA_E_EXACT) / (DELTA_E_BROAD - DELTA_E_EXACT)) * 100
                    )
                )
            );
            const label = getMatchLabel(proximity);
            return `${hex} (Tolerance: ${proximity}% - ${label})`;
        } catch {
            return String(value);
        }
    }
};
