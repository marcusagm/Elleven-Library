import { createSignal, createMemo, createEffect } from 'solid-js';
import { DatePickerProperties, DatePickerViewMode } from './types';

/**
 * Custom hook to manage the internal state and business logic of the DatePicker component.
 * Handles view transitions between day/month/year, calendar grid calculations, and date selection logic.
 *
 * @param properties - The reactive configuration properties for the DatePicker.
 * @returns An object containing reactive state, computed values, and orchestration methods for the picker.
 */
export const useDatePicker = (properties: DatePickerProperties) => {
    const today = new Date();

    // View date manages what month/year is currently displayed in the calendar grid.
    const [viewDate, setViewDate] = createSignal(today);
    const [viewMode, setViewMode] = createSignal<DatePickerViewMode>('day');

    // Synchronize the internal view date whenever the external value property changes.
    createEffect(() => {
        const selectedDateValue = properties.value;
        if (selectedDateValue) {
            setViewDate(new Date(selectedDateValue));
        }
    });

    const MONTH_NAME_LIST = [
        'January',
        'February',
        'March',
        'April',
        'May',
        'June',
        'July',
        'August',
        'September',
        'October',
        'November',
        'December'
    ];

    const WEEKDAY_LABEL_LIST = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];

    /**
     * Calculates the days to display for the current month view.
     */
    const calendarDays = createMemo(() => {
        const year = viewDate().getFullYear();
        const month = viewDate().getMonth();

        // Days in current month
        const daysInMonth = new Date(year, month + 1, 0).getDate();
        // Index of the first day of the month (0 = Sunday)
        const firstWeekdayIndex = new Date(year, month, 1).getDay();

        // Padding for the grid
        const paddingDays = Array(firstWeekdayIndex).fill(null);
        const actualDays = Array.from({ length: daysInMonth }, (_, index) => index + 1);

        return [...paddingDays, ...actualDays];
    });

    /**
     * Calculates the decade start year for the year selection view.
     */
    const decadeStartYear = createMemo(() => {
        return Math.floor(viewDate().getFullYear() / 12) * 12;
    });

    /**
     * Array of years to display in the year selection grid.
     */
    const selectableYears = createMemo(() => {
        const start = decadeStartYear();
        return Array.from({ length: 12 }, (_, index) => start + index);
    });

    /**
     * Determines if a specific day represents the currently selected value.
     *
     * @param dayOfMonth - The numeric day of the month to evaluate.
     * @returns True if the day is currently selected.
     */
    const isDaySelected = (dayOfMonth: number) => {
        if (!properties.value) return false;
        return (
            properties.value.getDate() === dayOfMonth &&
            properties.value.getMonth() === viewDate().getMonth() &&
            properties.value.getFullYear() === viewDate().getFullYear()
        );
    };

    /**
     * Checks if a specific day is "today".
     */
    const isToday = (dayOfMonth: number) => {
        return (
            dayOfMonth === today.getDate() &&
            viewDate().getMonth() === today.getMonth() &&
            viewDate().getFullYear() === today.getFullYear()
        );
    };

    /**
     * Navigates to the previous period based on the current view mode.
     */
    const navigatePrevious = () => {
        const date = new Date(viewDate());
        if (viewMode() === 'day') {
            date.setMonth(date.getMonth() - 1);
        } else if (viewMode() === 'month') {
            date.setFullYear(date.getFullYear() - 1);
        } else {
            date.setFullYear(date.getFullYear() - 12);
        }
        setViewDate(date);
    };

    /**
     * Navigates to the next period based on the current view mode.
     */
    const navigateNext = () => {
        const date = new Date(viewDate());
        if (viewMode() === 'day') {
            date.setMonth(date.getMonth() + 1);
        } else if (viewMode() === 'month') {
            date.setFullYear(date.getFullYear() + 1);
        } else {
            date.setFullYear(date.getFullYear() + 12);
        }
        setViewDate(date);
    };

    /**
     * Handles clicking on the calendar title to zoom out the view.
     */
    const handleTitleClick = () => {
        if (viewMode() === 'day') {
            setViewMode('month');
        } else if (viewMode() === 'month') {
            setViewMode('year');
        }
    };

    /**
     * Computes the formatted title string (e.g., "August 2026" or "2020 - 2031") for the current view.
     */
    const viewTitle = createMemo(() => {
        if (viewMode() === 'day') {
            return `${MONTH_NAME_LIST[viewDate().getMonth()]} ${viewDate().getFullYear()}`;
        }
        if (viewMode() === 'month') {
            return `${viewDate().getFullYear()}`;
        }
        const start = decadeStartYear();
        return `${start} - ${start + 11}`;
    });

    /**
     * Finalizes the selection of a specific day and triggers the change event.
     *
     * @param dayOfMonth - The numeric day of the month selected by the user.
     */
    const selectDay = (dayOfMonth: number) => {
        const selectedFullDate = new Date(
            viewDate().getFullYear(),
            viewDate().getMonth(),
            dayOfMonth
        );
        properties.onChange?.(selectedFullDate);
    };

    /**
     * Handles selecting a specific month and zooms into day view.
     */
    const selectMonth = (monthIndex: number) => {
        const date = new Date(viewDate());
        date.setMonth(monthIndex);
        setViewDate(date);
        setViewMode('day');
    };

    /**
     * Handles selecting a specific year and zooms into month view.
     */
    const selectYear = (year: number) => {
        const date = new Date(viewDate());
        date.setFullYear(year);
        setViewDate(date);
        setViewMode('month');
    };

    return {
        viewDate,
        viewMode,
        viewTitle,
        calendarDays,
        selectableYears,
        MONTH_NAME_LIST,
        WEEKDAY_LABEL_LIST,
        isDaySelected,
        isToday,
        navigatePrevious,
        navigateNext,
        handleTitleClick,
        selectDay,
        selectMonth,
        selectYear
    };
};
