import { Component, JSX, Show } from 'solid-js';
import { DateInput } from '../../../ui/DateInput';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders a specialized input field for date criteria in the advanced search.
 * Supports single date selection or a date range selection when the "between" operator is active.
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the date field renderer.
 * @returns {JSX.Element} The rendered date input group.
 */
export const DateCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
    /**
     * Checks if the current comparison logic expects a range of two dates.
     *
     * @returns {boolean} True if the comparison operator is 'between', false otherwise.
     */
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
