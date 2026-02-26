import { JSX } from 'solid-js';

/**
 * Defines the visual style variants for the Badge component.
 */
export type BadgeVariant =
    | 'default'
    | 'outline'
    | 'secondary'
    | 'success'
    | 'warning'
    | 'error'
    | 'info';

/**
 * Defines the available sizes for the Badge component.
 */
export type BadgeSize = 'sm' | 'md';

/**
 * Properties for the Badge component.
 */
export interface BadgeProperties extends JSX.HTMLAttributes<HTMLSpanElement> {
    /**
     * Visual variant style of the badge.
     * @default 'default'
     */
    variant?: BadgeVariant;
    /**
     * Size indicator for the badge.
     * @default 'md'
     */
    size?: BadgeSize;
    /**
     * Additional content inside the badge, usually text.
     */
    children?: JSX.Element;
}
