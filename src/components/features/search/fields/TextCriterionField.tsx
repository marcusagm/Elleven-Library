import { Component, JSX } from 'solid-js';
import { Input } from '../../../ui/Input';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders a basic text input for text-based search criteria (e.g., filenames).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the text field renderer.
 * @returns {JSX.Element} The rendered text input component.
 */
export const TextCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
    return (
        <Input
            size={properties.size || 'md'}
            value={(properties.value as string) || ''}
            onInput={event => properties.setValue(event.currentTarget.value)}
            placeholder="Value..."
            error={!!properties.errors.value}
            errorMessage={properties.errors.value}
        />
    );
};
