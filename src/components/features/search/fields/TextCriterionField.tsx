import { Component } from 'solid-js';
import { Input } from '../../../ui/Input';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders a basic text input for text-based search criteria (e.g., filenames).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the text field renderer.
 * @returns {JSX.Element} The rendered text input component.
 */
export const TextCriterionField: Component<CriterionFieldRendererProperties> = properties => {
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

/**
 * Handler implementation for text-based search criteria.
 */
export const textHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the text input. */
    component: TextCriterionField,

    /**
     * Validates that some text has been entered.
     *
     * @param value - Primary text value input.
     * @returns A map of validation error messages.
     */
    validate: value => {
        const validationErrors: Record<string, string> = {};
        if (value === null || value === '') {
            validationErrors.value = 'Value is required';
        }
        return validationErrors;
    },

    /**
     * Processes the text value for the search query.
     *
     * @param value - Selected text string.
     * @returns The unchanged value.
     */
    process: value => ({ finalValue: value })
};
