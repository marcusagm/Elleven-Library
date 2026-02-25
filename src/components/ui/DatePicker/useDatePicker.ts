import { createSignal, createMemo, createEffect } from 'solid-js';
import { DatePickerProperties, DatePickerViewMode } from './types';

/**
 * Hook to manage the internal logic of the DatePicker component.
 * Handles view transitions, calendar calculations, and selection.
 *
 * @param props - Component properties.
 * @returns State and methods for the DatePicker.
 */
export const useDatePicker = (props: DatePickerProperties) => {
    const today = new Date();

    // View date manages what month/year is currently displayed in the calendar grid.
    const [viewDate, setViewDate] = createSignal(today);
    const [viewMode, setViewMode] = createSignal<DatePickerViewMode>('day');

    // Sync view date if props.value changes externally
    createEffect(() => {
        const selectedDate = props.value;
        if (selectedDate) {
            setViewDate(new Date(selectedDate));
        }
    });

    const MONTH_NAMES = [
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

    const WEEKDAYS_LABELS = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];

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
     * Checks if a specific day is currently selected.
     * @param dayOfMonth - Day of the month to check.
     */
    const isDaySelected = (dayOfMonth: number) => {
        if (!props.value) return false;
        return (
            props.value.getDate() === dayOfMonth &&
            props.value.getMonth() === viewDate().getMonth() &&
            props.value.getFullYear() === viewDate().getFullYear()
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
     * Formats the title display based on the current view.
     */
    const viewTitle = createMemo(() => {
        if (viewMode() === 'day') {
            return `${MONTH_NAMES[viewDate().getMonth()]} ${viewDate().getFullYear()}`;
        }
        if (viewMode() === 'month') {
            return `${viewDate().getFullYear()}`;
        }
        const start = decadeStartYear();
        return `${start} - ${start + 11}`;
    });

    /**
     * Handles selecting a specific day.
     */
    const selectDay = (dayOfMonth: number) => {
        const newDate = new Date(viewDate().getFullYear(), viewDate().getMonth(), dayOfMonth);
        props.onChange?.(newDate);
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
        MONTH_NAMES,
        WEEKDAYS_LABELS,
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
