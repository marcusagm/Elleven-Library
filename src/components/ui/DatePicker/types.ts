/**
 * Properties for the DatePicker component.
 * Used to select a specific date from a calendar interface.
 */
export interface DatePickerProperties {
    /** The currently selected date. */
    value?: Date;

    /** Callback triggered when a new date is selected. */
    onChange?: (date: Date) => void;

    /** The earliest date that can be selected. */
    minDate?: Date;

    /** The latest date that can be selected. */
    maxDate?: Date;

    /** Additional CSS class for the root element. */
    class?: string;
}

/**
 * Represents the current resolution of the calendar view.
 */
export type DatePickerViewMode = 'day' | 'month' | 'year';

/**
 * Internal state for the date picker management.
 */
export interface DatePickerState {
    /** The date currently being viewed in the calendar (may differ from selected value). */
    viewDate: Date;

    /** The current zoom level of the calendar. */
    viewMode: DatePickerViewMode;
}
