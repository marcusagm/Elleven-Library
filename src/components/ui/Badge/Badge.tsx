import { Component, splitProps } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { BadgeProperties } from './types';
import './badge.css';

/**
 * Badge component for small statuses, categories, or indicators.
 *
 * @param {BadgeProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered badge component.
 *
 * @example
 * <Badge variant="success">Active</Badge>
 *
 * @example
 * <Badge variant="outline" size="sm">Tag</Badge>
 */
export const Badge: Component<BadgeProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'variant',
        'size',
        'class',
        'children'
    ]);

    const activeVariant = () => localProperties.variant || 'default';
    const activeSize = () => localProperties.size || 'md';

    return (
        <span
            class={concatenateClasses(
                'ui-badge',
                `ui-badge-${activeVariant()}`,
                `ui-badge-${activeSize()}`,
                localProperties.class
            )}
            {...remainingProperties}
        >
            {localProperties.children}
        </span>
    );
};
