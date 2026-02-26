import { JSX } from 'solid-js';

/**
 * Defines the orientation of the separator element.
 */
export type SeparatorOrientation = 'horizontal' | 'vertical';

/**
 * Properties for the Separator component.
 */
export interface SeparatorProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /**
     * Orientation of the separator.
     * @default 'horizontal'
     */
    orientation?: SeparatorOrientation;
    /**
     * Whether this is a purely decorative element.
     * If false, the separator will be exposed to screen readers as a formal separator.
     * @default true
     */
    isDecorative?: boolean;
}
