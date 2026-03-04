import { Component, createMemo, JSX } from 'solid-js';
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
export const TagsCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
    /**
     * Accesses global metadata containing available tags.
     *
     * @type {Metadata}
     */
    const metadata = useMetadata();

    /**
     * Computes a flat list of tags with visual indentation to represent hierarchy.
     *
     * @type {Memo<Tag[]>}
     */
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
