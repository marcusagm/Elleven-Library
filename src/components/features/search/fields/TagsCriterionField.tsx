import { Component, createMemo } from 'solid-js';
import { Select } from '../../../ui/Select';
import { useMetadata } from '../../../../core/hooks';
import { getHierarchicalTags } from '../searchHelpers';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders a specialized dropdown selector for tag-based search criteria.
 * Uses a hierarchical representation of tags for easier navigation.
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the tag field renderer.
 * @returns {JSX.Element} The rendered tag select component.
 */
export const TagsCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    /** Accesses global metadata containing available tags. */
    const metadata = useMetadata();

    /** Computes a flat list of tags with visual indentation to represent hierarchy. */
    const hierarchicalTags = createMemo(() => getHierarchicalTags(metadata.tags));

    return (
        <Select
            size={properties.size || 'md'}
            options={hierarchicalTags()}
            value={String(properties.value || '')}
            onValueChange={value => properties.setValue(value)}
            searchable
            error={!!properties.errors.value}
            errorMessage={properties.errors.value}
            class="tag-select-wrapper"
        />
    );
};

/**
 * Handler implementation for tag-based search criteria.
 * Resolves numeric tag IDs to human-readable names for display.
 */
export const tagsHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the tag selector. */
    component: TagsCriterionField,

    /**
     * Validates that a tag has been selected.
     *
     * @param value - The primary tag ID selection.
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
     * Processes the raw tag ID for query usage.
     *
     * @param value - Selected tag ID.
     * @returns The unchanged value.
     */
    process: value => ({ finalValue: value }),

    /**
     * Resolves the tag ID to its display name using provided metadata.
     *
     * @param tagId - The ID of the tag to resolve.
     * @param _value2 - Unused secondary value.
     * @param _operator - Unused operator.
     * @param _unitMultiplier - Unused unit.
     * @param metadata - Store metadata containing the global tags list.
     * @returns The descriptive name of the tag or the raw ID if not found.
     */
    formatDisplay: (tagId, _value2, _operator, _unitMultiplier, metadata) => {
        const matchedTag = metadata?.tags.find(tag => String(tag.id) === String(tagId));
        return matchedTag?.name || String(tagId);
    }
};
