import { Component, splitProps, createMemo } from 'solid-js';
import { cn } from '../../../lib/utils';
import { ButtonGroupProperties } from './types';
import './button-group.css';

/**
 * ButtonGroup component for grouping related buttons together.
 * Supports horizontal/vertical layouts and attached styling for a unified look.
 *
 * @param {ButtonGroupProperties} properties - The properties for the ButtonGroup component.
 * @returns {JSX.Element} The rendered button group container.
 *
 * @example
 * <ButtonGroup attached>
 *   <Button variant="outline">Left</Button>
 *   <Button variant="outline">Center</Button>
 *   <Button variant="outline">Right</Button>
 * </ButtonGroup>
 *
 * @example
 * <ButtonGroup orientation="vertical" size="sm">
 *   <Button>Up</Button>
 *   <Button>Down</Button>
 * </ButtonGroup>
 */
export const ButtonGroup: Component<ButtonGroupProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'class',
        'orientation',
        'size',
        'attached',
        'children'
    ]);

    /**
     * Computes the CSS class names for the ButtonGroup component based on its active properties.
     */
    const computedClasses = createMemo(() =>
        cn(
            'ui-button-group',
            `ui-button-group-${localProperties.orientation || 'horizontal'}`,
            localProperties.attached && 'ui-button-group-attached',
            localProperties.size && `ui-button-group-${localProperties.size}`,
            localProperties.class
        )
    );

    return (
        <div class={computedClasses()} role="group" {...remainingProperties}>
            {localProperties.children}
        </div>
    );
};
