import { Component } from 'solid-js';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders a dropdown selection field for rating-based search criteria (0-5 stars).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the rating field renderer.
 * @returns {JSX.Element} The rendered rating select component.
 */
export const RatingCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    return (
        <Select
            size={properties.size || 'md'}
            options={[0, 1, 2, 3, 4, 5].map(ratingValue => ({
                value: String(ratingValue),
                label: `${ratingValue} Stars`
            }))}
            value={String(properties.value ?? '0')}
            onValueChange={value => properties.setValue(Number(value))}
            error={!!properties.errors.value}
            errorMessage={properties.errors.value}
        />
    );
};

/**
 * Handler implementation for rating-based search criteria.
 * Formats numeric ratings into "X Stars" strings for display.
 */
export const ratingHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the rating selector. */
    component: RatingCriterionField,

    /**
     * Validates that a rating has been provided.
     *
     * @param value - The primary rating value selection.
     * @returns A record of validation error messages.
     */
    validate: value => {
        const validationErrors: Record<string, string> = {};
        if (value === null || value === '') {
            validationErrors.value = 'Value is required';
        }
        return validationErrors;
    },

    /**
     * Processes the rating value for query usage.
     *
     * @param value - Selected numeric rating.
     * @returns The unchanged value.
     */
    process: value => ({ finalValue: value }),

    /**
     * Formats the numeric rating into a localized string.
     *
     * @param ratingCount - The numeric rating count.
     * @returns A friendly "X Stars" string.
     */
    formatDisplay: ratingCount => `${ratingCount} Stars`
};
