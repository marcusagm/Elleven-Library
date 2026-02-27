import { StoreMetadata } from '../../../../components/features/search/fields/types';
import { formatToISO, formatToDisplay } from '../../../../utils/format';
import { SIZE_UNITS } from '../constants';

/**
 * Basic search value types.
 */
export type SearchValue = string | number | null | Date;

/**
 * Interface defining the business logic for validating, processing, and formatting
 * a specific search field type. This is decoupled from UI components.
 */
export interface SearchFieldLogic {
    validate: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unitMultiplier?: string
    ) => Record<string, string>;

    process: (
        value: SearchValue,
        value2: SearchValue,
        operator: string,
        unitMultiplier?: string
    ) => {
        finalValue: unknown;
        unitMultiplier?: string;
    };

    formatDisplay?: (
        value: unknown,
        value2: unknown,
        operator: string,
        unitMultiplier?: string,
        metadata?: StoreMetadata
    ) => string;
}

/** Helper to check for empty values */
const checkIsEmpty = (value: unknown) => value === null || value === undefined || value === '';

/** Date Logic */
export const dateLogic: SearchFieldLogic = {
    validate: (value, value2, operator) => {
        const validationErrors: Record<string, string> = {};
        if (checkIsEmpty(value)) {
            validationErrors.value = 'Date is required';
        }

        if (operator === 'between') {
            if (checkIsEmpty(value2)) {
                validationErrors.value2 = 'End date is required';
            } else if (!checkIsEmpty(value)) {
                const startDateObject = new Date(value as string | Date);
                const endDateObject = new Date(value2 as string | Date);
                if (startDateObject > endDateObject) {
                    validationErrors.value2 = 'End date must be after start date';
                }
            }
        }
        return validationErrors;
    },
    process: (value, value2, operator) => {
        if (operator === 'between') {
            return {
                finalValue: [
                    formatToISO(value as Date | string),
                    formatToISO(value2 as Date | string)
                ]
            };
        }
        return { finalValue: formatToISO(value as Date | string) };
    },
    formatDisplay: (value1, value2, operator) => {
        if (operator === 'between') {
            return `${formatToDisplay(value1 as string | Date)} to ${formatToDisplay(value2 as string | Date)}`;
        }
        return formatToDisplay(value1 as string | Date);
    }
};

/** Folder Logic */
export const folderLogic: SearchFieldLogic = {
    validate: value => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        return errors;
    },
    process: value => ({ finalValue: value }),
    formatDisplay: (folderId, _v2, _op, _unit, metadata) => {
        const matchedFolder = metadata?.locations.find(l => String(l.id) === String(folderId));
        return matchedFolder?.name || String(folderId);
    }
};

/** Generic Number Logic */
export const numberLogic: SearchFieldLogic = {
    validate: (value, value2, operator) => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        if (operator === 'between') {
            if (checkIsEmpty(value2)) errors.value2 = 'End value is required';
            else if (!checkIsEmpty(value) && Number(value) > Number(value2)) {
                errors.value2 = 'End value must be greater than start';
            }
        }
        return errors;
    },
    process: (value, value2, operator) => {
        if (operator === 'between') return { finalValue: [value, value2] };
        return { finalValue: value };
    }
};

/** Rating Logic */
export const ratingLogic: SearchFieldLogic = {
    validate: value => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        return errors;
    },
    process: value => ({ finalValue: value }),
    formatDisplay: ratingCount => `${ratingCount} Stars`
};

/** Select Logic (File Format) */
export const selectLogic: SearchFieldLogic = {
    validate: value => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        return errors;
    },
    process: value => ({ finalValue: value }),
    formatDisplay: (val, _v2, _op, _unit, metadata) => {
        const found = metadata?.supportedFormats?.find(f => f.extensions.includes(String(val)));
        return found ? `.${String(val).toUpperCase()} (${found.name})` : String(val);
    }
};

/** Size Logic */
export const sizeLogic: SearchFieldLogic = {
    validate: (value, value2, operator, unitMultiplier) => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        if (operator === 'between') {
            if (checkIsEmpty(value2)) errors.value2 = 'End value is required';
            else if (!checkIsEmpty(value) && Number(value) > Number(value2)) {
                errors.value2 = 'End value must be greater than start';
            }
        }
        if (!SIZE_UNITS.find(opt => opt.value === unitMultiplier)) errors.unit = 'Unit is required';
        return errors;
    },
    process: (value, value2, operator, unitMultiplier) => {
        const mult = Number(unitMultiplier);
        if (operator === 'between') {
            return {
                finalValue: [Math.round(Number(value) * mult), Math.round(Number(value2) * mult)],
                unitMultiplier
            };
        }
        return { finalValue: Math.round(Number(value) * mult), unitMultiplier };
    },
    formatDisplay: (v1, v2, op, unitMultiplier) => {
        const label = SIZE_UNITS.find(o => o.value === unitMultiplier)?.label || 'bytes';
        return op === 'between' ? `${v1} ${label} to ${v2} ${label}` : `${v1} ${label}`;
    }
};

/** Tags Logic */
export const tagsLogic: SearchFieldLogic = {
    validate: value => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        return errors;
    },
    process: value => ({ finalValue: value }),
    formatDisplay: (tagId, _v2, _op, _unit, metadata) => {
        const matched = metadata?.tags.find(t => String(t.id) === String(tagId));
        return matched?.name || String(tagId);
    }
};

/** Text Logic */
export const textLogic: SearchFieldLogic = {
    validate: value => {
        const errors: Record<string, string> = {};
        if (checkIsEmpty(value)) errors.value = 'Value is required';
        return errors;
    },
    process: value => ({ finalValue: value })
};

/**
 * Registry mapping field identifiers to their business logic.
 */
export const criterionLogicRegistry: Record<string, SearchFieldLogic> = {
    date: dateLogic,
    folder: folderLogic,
    number: numberLogic,
    rating: ratingLogic,
    select: selectLogic,
    size: sizeLogic,
    tags: tagsLogic,
    text: textLogic
};
