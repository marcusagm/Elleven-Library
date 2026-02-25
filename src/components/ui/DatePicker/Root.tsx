import { Component, For, splitProps } from 'solid-js';
import { ChevronLeft, ChevronRight } from 'lucide-solid';
import { cn } from '../../../lib/utils';
import { DatePickerProperties } from './types';
import { useDatePicker } from './useDatePicker';
import './date-picker.css';

/**
 * A calendar-based date selection component allowing users to pick a date from a visual grid.
 * Supports specialized day, month, and year selection views for quick navigation.
 *
 * @param properties - The reactive properties defining the behavior and appearance of the DatePicker.
 * @returns The rendered DatePicker component interface.
 *
 * @example
 * <DatePicker value={new Date()} onChange={(date) => console.log(date)} />
 */
export const DatePicker: Component<DatePickerProperties> = properties => {
    // Separate specialized picker properties from other standard HTML attributes.
    const [pickerProperties, otherHtmlAttributes] = splitProps(properties, [
        'value',
        'onChange',
        'minDate',
        'maxDate',
        'class'
    ]);

    const {
        viewMode,
        viewTitle,
        calendarDays,
        selectableYears,
        MONTH_NAME_LIST,
        WEEKDAY_LABEL_LIST,
        isDaySelected,
        isToday,
        viewDate,
        navigatePrevious,
        navigateNext,
        handleTitleClick,
        selectDay,
        selectMonth,
        selectYear
    } = useDatePicker(pickerProperties);

    return (
        <div
            class={cn('ui-date-picker', pickerProperties.class)}
            onClick={event => event.stopPropagation()}
            {...otherHtmlAttributes}
        >
            {/* Calendar Header */}
            <div class="ui-date-picker-header">
                <button
                    type="button"
                    class="ui-date-picker-nav"
                    onClick={navigatePrevious}
                    aria-label="Previous period"
                >
                    <ChevronLeft size={16} />
                </button>

                <button
                    type="button"
                    class="ui-date-picker-title-button"
                    onClick={handleTitleClick}
                    aria-label="Zoom out view"
                >
                    {viewTitle()}
                </button>

                <button
                    type="button"
                    class="ui-date-picker-nav"
                    onClick={navigateNext}
                    aria-label="Next period"
                >
                    <ChevronRight size={16} />
                </button>
            </div>

            {/* Day Selection View */}
            {viewMode() === 'day' && (
                <div class="ui-date-picker-grid ui-date-picker-day-grid">
                    <For each={WEEKDAY_LABEL_LIST}>
                        {label => <div class="ui-date-picker-weekday">{label}</div>}
                    </For>
                    <For each={calendarDays()}>
                        {dayOfMonth => (
                            <button
                                type="button"
                                class={cn(
                                    'ui-date-picker-cell',
                                    !dayOfMonth && 'ui-date-picker-empty',
                                    dayOfMonth &&
                                        isDaySelected(dayOfMonth) &&
                                        'ui-date-picker-selected',
                                    dayOfMonth &&
                                        isToday(dayOfMonth) &&
                                        !isDaySelected(dayOfMonth) &&
                                        'ui-date-picker-today'
                                )}
                                disabled={!dayOfMonth}
                                onClick={() => dayOfMonth && selectDay(dayOfMonth)}
                            >
                                {dayOfMonth}
                            </button>
                        )}
                    </For>
                </div>
            )}

            {/* Month Selection View */}
            {viewMode() === 'month' && (
                <div class="ui-date-picker-grid ui-date-picker-month-grid">
                    <For each={MONTH_NAME_LIST}>
                        {(monthName, index) => (
                            <button
                                type="button"
                                class={cn(
                                    'ui-date-picker-cell ui-date-picker-month-cell',
                                    viewDate().getMonth() === index() && 'ui-date-picker-selected'
                                )}
                                onClick={() => selectMonth(index())}
                            >
                                {monthName.substring(0, 3)}
                            </button>
                        )}
                    </For>
                </div>
            )}

            {/* Year Selection View */}
            {viewMode() === 'year' && (
                <div class="ui-date-picker-grid ui-date-picker-year-grid">
                    <For each={selectableYears()}>
                        {yearValue => (
                            <button
                                type="button"
                                class={cn(
                                    'ui-date-picker-cell ui-date-picker-year-cell',
                                    viewDate().getFullYear() === yearValue &&
                                        'ui-date-picker-selected'
                                )}
                                onClick={() => selectYear(yearValue)}
                            >
                                {yearValue}
                            </button>
                        )}
                    </For>
                </div>
            )}
        </div>
    );
};
