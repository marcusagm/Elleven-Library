/**
 * Facade Module to wrap deep interaction against raw Search Field handlers routines.
 *
 * @module criterionHelpers
 * @description
 * It standardizes entry-points mapping dynamic UI/event constraints automatically against
 * proper domain-separated logic inside `handlers`. Injects ambient Singletons implicitly
 * as MetaData to alleviate direct dependencie needs externally.
 */

import { SEARCH_FIELDS } from './constants';
import { type SearchCriterion } from './schemas';
import { coreCriterionHandlerRegistry, textHandler, type SearchValue } from './handlers';
import { metadataState } from '../metadata';
import { supportedFormats } from '../systemStore';

export const criterionHelpers = {
    /**
     * Resolves mathematical validation boundaries depending on exact field mapping rules.
     *
     * @param {string} key - The primitive database field target path.
     * @param {string} operator - The logical matching statement (contains, is, between).
     * @param {unknown} value - Generic target value start.
     * @param {unknown} [value2] - Optional threshold end value for "between" logic gaps.
     * @param {string} [unitMultiplier] - Custom unit definitions acting dynamically on byte scales.
     * @returns {Record<string, string>} Hashmap object holding explicit field error messages.
     *
     * @example
     * ```ts
     * const errors = criterionHelpers.validateCriterion("size", "between", 10, 50, "1024");
     * ```
     */
    validateCriterion: (
        key: string,
        operator: string,
        value: unknown,
        value2?: unknown,
        unitMultiplier?: string
    ): Record<string, string> => {
        const field = SEARCH_FIELDS.find(f => f.value === key);
        if (!field) return { value: 'Invalid field' };

        const logic = coreCriterionHandlerRegistry[field.type] || textHandler;
        return logic.validate(
            value as SearchValue,
            value2 as SearchValue,
            operator,
            unitMultiplier
        );
    },

    /**
     * Composes readable user visual interfaces parsing database primitives into labels.
     *
     * @param {Omit<SearchCriterion, 'id'>} criterion - State object minus identification primitives.
     * @returns {string} Fully readable localized translation string for User Interface matching.
     *
     * @example
     * ```ts
     * const readableLabel = criterionHelpers.formatCriterionDisplay({ key: "size", operator: "gt", value: 4096, unitMultiplier: "1024" });
     * ```
     */
    formatCriterionDisplay: (criterion: Omit<SearchCriterion, 'id'>): string => {
        const field = SEARCH_FIELDS.find(f => f.value === criterion.key);
        if (!field) return String(criterion.value);

        const logic = coreCriterionHandlerRegistry[field.type] || textHandler;
        if (logic.formatDisplay) {
            const rawValue = criterion.value;
            const value1 = Array.isArray(rawValue) ? rawValue[0] : rawValue;
            const value2 = Array.isArray(rawValue) ? rawValue[1] : undefined;

            return logic.formatDisplay(
                value1,
                value2,
                criterion.operator,
                criterion.unitMultiplier,
                {
                    locations: metadataState.locations,
                    tags: metadataState.tags,
                    supportedFormats: supportedFormats()
                }
            );
        }

        return String(criterion.value);
    },

    /**
     * Translates human-defined variables into solid Database-Ready primitive mappings.
     *
     * @param {string} key - Primary mapped parameter field string.
     * @param {string} operator - Targeted logical condition to dictate gap generation.
     * @param {unknown} value - Main criterion initial user boundary block.
     * @param {unknown} [value2] - Secondary bounds specific generally for "between" commands.
     * @param {string} [unitMultiplier] - Mathematical reference constants string values.
     * @returns {Object} Containing mapped array-ified/processed properties and normalized multi-values.
     *
     * @example
     * ```ts
     * const { finalValue } = criterionHelpers.processCriterion("size", "between", 10, 50, "1024");
     * ```
     */
    processCriterion: (
        key: string,
        operator: string,
        value: unknown,
        value2?: unknown,
        unitMultiplier?: string
    ) => {
        const field = SEARCH_FIELDS.find(f => f.value === key);
        if (!field) return { finalValue: value, unitMultiplier };

        const logic = coreCriterionHandlerRegistry[field.type] || textHandler;
        return logic.process(value as SearchValue, value2 as SearchValue, operator, unitMultiplier);
    }
};
