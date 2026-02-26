import { JSX } from 'solid-js';

/**
 * Properties for the SectionGroup component.
 */
export interface SectionGroupProperties extends JSX.HTMLAttributes<HTMLElement> {
    /**
     * The title displayed at the top of the group section.
     */
    title: string;
    /**
     * An optional descriptive text for more context under the title.
     */
    description?: string;
}
