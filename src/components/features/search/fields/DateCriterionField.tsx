import { Component, Show } from 'solid-js';
import { DateInput } from '../../../ui/DateInput';
import { CriterionFieldRendererProperties } from './types';
import { formatToISO, formatToDisplay } from '../../../../utils/format';

/**
 * Renders a specialized input field for date criteria in the advanced search.
 * Supports single date selection or a date range selection when the "between" operator is active.
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the date field renderer.
 * @returns {JSX.Element} The rendered date input group.
 */
export const DateCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    /** Checks if the current comparison logic expects a range of two dates. */
    const isRangeMode = () => properties.comparisonOperator === 'between';

    return (
        <div class="date-input-group">
            <DateInput
                size={properties.size || 'md'}
                value={(properties.value as Date) || null}
                onChange={value => properties.setValue(value)}
                placeholder={isRangeMode() ? 'From Date' : 'Date'}
                error={!!properties.errors.value}
                errorMessage={properties.errors.value}
            />
            <Show when={isRangeMode() && properties.setValue2}>
                <span class="range-separator">to</span>
                <DateInput
                    size={properties.size || 'md'}
                    value={(properties.value2 as Date) || null}
                    onChange={value => properties.setValue2?.(value)}
                    placeholder="To Date"
                    error={!!properties.errors.value2}
                    errorMessage={properties.errors.value2}
                />
            </Show>
        </div>
    );
};

/**
 * Handler implementation for date-based search criteria.
 * Manages validation, processing for storage, and human-readable display formatting.
 */
export const dateHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the date inputs. */
    component: DateCriterionField,

    /**
     * Validates that date inputs are provided and that range logic (if active) is consistent.
     *
     * @param value - The primary date value.
     * @param value2 - The secondary end date value for ranges.
     * @param operator - The comparison logic being used.
     * @returns A record of validation error messages.
     */
    validate: (value, value2, operator) => {
        const validationErrors: Record<string, string> = {};
        if (value === null || value === '') {
            validationErrors.value = 'Date is required';
        }

        if (operator === 'between') {
            if (value2 === null || value2 === '') {
                validationErrors.value2 = 'End date is required';
            } else if (value !== null && value !== '') {
                const startDateObject = new Date(value as string | Date);
                const endDateObject = new Date(value2 as string | Date);
                if (startDateObject > endDateObject) {
                    validationErrors.value2 = 'End date must be after start date';
                }
            }
        }
        return validationErrors;
    },

    /**
     * Converts the reactive Date objects into ISO standard strings for data persistence.
     *
     * @param value - Primary date selection.
     * @param value2 - Secondary date selection (optional).
     * @param operator - Current comparison operator.
     * @returns The final processed value representation.
     */
    process: (value, value2, operator) => {
        if (operator === 'between') {
            const isoStringStart = formatToISO(value as Date | string);
            const isoStringEnd = formatToISO(value2 as Date | string);
            return { finalValue: [isoStringStart, isoStringEnd] };
        }
        return { finalValue: formatToISO(value as Date | string) };
    },

    /**
     * Creates a human-friendly display string for the date criterion.
     *
     * @param value1 - Formatted primary date string.
     * @param value2 - Formatted secondary date string (if any).
     * @param operator - Comparison operator for context.
     * @returns The localized display string.
     */
    formatDisplay: (value1, value2, operator) => {
        if (operator === 'between') {
            return `${formatToDisplay(value1 as string | Date)} to ${formatToDisplay(value2 as string | Date)}`;
        }
        return formatToDisplay(value1 as string | Date);
    }
};
