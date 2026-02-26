import { JSX } from 'solid-js';

/**
 * Defines the available sizes for the Loader component.
 */
export type LoaderSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the Loader component.
 */
export interface LoaderProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /**
     * Size of the loading indicator.
     * @default 'md'
     */
    size?: LoaderSize;
    /**
     * Whether to take up the full screen/container.
     * @default false
     */
    isFullscreen?: boolean;
    /**
     * Optional text to display below the loading indicator.
     */
    text?: string;
    /**
     * If provided, shows a progress bar below the text.
     * Value should be percentage (0-100) or current if max is set.
     */
    progress?: number;
    /**
     * Maximum value for progress calculation.
     */
    maximumValue?: number;
    /**
     * Additional CSS class.
     */
    class?: string;
}
