import { Component, splitProps } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { AlertDescriptionProperties } from './types';

/**
 * Component for the description or body content of an alert.
 *
 * @param {AlertDescriptionProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered description component.
 *
 * @example
 * <Alert.Description>Your changes have been saved successfully.</Alert.Description>
 */
export const AlertDescription: Component<AlertDescriptionProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, ['class', 'children']);

    return (
        <div
            class={concatenateClasses('ui-alert-description', localProperties.class)}
            {...remainingProperties}
        >
            {localProperties.children}
        </div>
    );
};
