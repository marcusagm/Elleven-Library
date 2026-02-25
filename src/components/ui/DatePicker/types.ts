/**
 * Properties defining the configuration and interactive state of the DatePicker component.
 * Used to control the calendar selection interface.
 */
export interface DatePickerProperties {
    /** The currently selected JS Date object. If undefined, no date is highlighted as selected. */
    value?: Date;

    /**
     * Callback function executed when the user selects a date from the calendar.
     * @param date - The newly selected Date object.
     */
    onChange?: (date: Date) => void;

    /** The minimum possible date that can be navigated to or selected (optional validation limit). */
    minDate?: Date;

    /** The maximum possible date that can be navigated to or selected (optional validation limit). */
    maxDate?: Date;

    /** Optional CSS class string to be applied to the root container of the date picker. */
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
