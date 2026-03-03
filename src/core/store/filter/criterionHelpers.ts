import { SEARCH_FIELDS } from './constants';
import { type SearchCriterion } from './schemas';
import { criterionLogicRegistry, textLogic, type SearchValue } from './logic/handlers';
import { metadataState } from '../metadata';
import { supportedFormats } from '../systemStore';

export const criterionHelpers = {
    validateCriterion: (
        key: string,
        operator: string,
        value: unknown,
        value2?: unknown,
        unitMultiplier?: string
    ): Record<string, string> => {
        const field = SEARCH_FIELDS.find(f => f.value === key);
        if (!field) return { value: 'Invalid field' };

        const logic = criterionLogicRegistry[field.type] || textLogic;
        return logic.validate(
            value as SearchValue,
            value2 as SearchValue,
            operator,
            unitMultiplier
        );
    },

    formatCriterionDisplay: (criterion: Omit<SearchCriterion, 'id'>): string => {
        const field = SEARCH_FIELDS.find(f => f.value === criterion.key);
        if (!field) return String(criterion.value);

        const logic = criterionLogicRegistry[field.type] || textLogic;
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

    processCriterion: (
        key: string,
        operator: string,
        value: unknown,
        value2?: unknown,
        unitMultiplier?: string
    ) => {
        const field = SEARCH_FIELDS.find(f => f.value === key);
        if (!field) return { finalValue: value, unitMultiplier };

        const logic = criterionLogicRegistry[field.type] || textLogic;
        return logic.process(value as SearchValue, value2 as SearchValue, operator, unitMultiplier);
    }
};
