import { Component, splitProps } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { AlertTitleProperties } from './types';

/**
 * Component for the title or heading of an alert.
 *
 * @param {AlertTitleProperties} properties - Component properties.
 * @returns {JSX.Element} The rendered title component.
 *
 * @example
 * <Alert.Title>Success!</Alert.Title>
 */
export const AlertTitle: Component<AlertTitleProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, ['class', 'children']);

    return (
        <h5
            class={concatenateClasses('ui-alert-title', localProperties.class)}
            {...remainingProperties}
        >
            {localProperties.children}
        </h5>
    );
};
