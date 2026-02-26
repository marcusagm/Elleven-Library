import { Component, splitProps } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { SeparatorProperties } from './types';
import './separator.css';

/**
 * Separator component for visually or semantically dividing content.
 *
 * @param {SeparatorProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered separator component.
 *
 * @example
 * <div>
 *   <p>First section</p>
 *   <Separator />
 *   <p>Second section</p>
 * </div>
 *
 * @example
 * <div style={{ display: "flex", gap: "8px", "align-items": "center" }}>
 *   <span>Item 1</span>
 *   <Separator orientation="vertical" />
 *   <span>Item 2</span>
 * </div>
 */
export const Separator: Component<SeparatorProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'orientation',
        'isDecorative',
        'class'
    ]);

    const activeOrientation = () => localProperties.orientation || 'horizontal';
    const isActuallyDecorative = () => localProperties.isDecorative ?? true;

    return (
        <div
            class={concatenateClasses(
                'ui-separator',
                `ui-separator-${activeOrientation()}`,
                localProperties.class
            )}
            role={isActuallyDecorative() ? 'none' : 'separator'}
            aria-orientation={!isActuallyDecorative() ? activeOrientation() : undefined}
            data-orientation={activeOrientation()}
            {...remainingProperties}
        />
    );
};
