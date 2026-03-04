import { Component, JSX } from 'solid-js';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders a dropdown selection field for rating-based search criteria (0-5 stars).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the rating field renderer.
 * @returns {JSX.Element} The rendered rating select component.
 */
export const RatingCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
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
