import { JSX } from 'solid-js';

/**
 * Defines the visual style variants for the CountBadge component.
 */
export type CountBadgeVariant = 'primary' | 'secondary' | 'outline';

/**
 * Properties for the CountBadge component.
 */
export interface CountBadgeProperties extends JSX.HTMLAttributes<HTMLSpanElement> {
    /**
     * The numeric value to display.
     */
    count: number;
    /**
     * Visual variant for the badge.
     * @default 'secondary'
     */
    variant?: CountBadgeVariant;
    /**
     * Maximum count before showing "max+".
     * @default 9999
     */
    max?: number;
    /**
     * Whether to show the badge when count is zero.
     * @default false
     */
    showZero?: boolean;
    /**
     * Additional CSS class.
     */
    class?: string;
}
