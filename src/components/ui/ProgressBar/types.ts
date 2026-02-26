/**
 * Available sizes for the ProgressBar component.
 */
export type ProgressBarSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the ProgressBar component.
 */
export interface ProgressBarProperties {
    /**
     * The current progress value.
     * Usually between 0 and 100, or up to the `maximumValue`.
     */
    value: number;
    /**
     * The maximum value representing 100% progress.
     * @default 100
     */
    maximumValue?: number;
    /**
     * The visual size variant of the progress bar.
     * @default 'sm'
     */
    size?: ProgressBarSize;
    /**
     * Whether to display the progress label (title and percentage).
     * @default false
     */
    isLabelVisible?: boolean;
    /**
     * A descriptive title to display above the progress bar.
     */
    labelTitle?: string;
    /**
     * Whether the progress is indeterminate (ongoing animation with no fixed value).
     * @default false
     */
    isIndeterminate?: boolean;
    /**
     * Additional CSS class for the container.
     */
    class?: string;
}
