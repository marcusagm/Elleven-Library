import { Component, createMemo, JSX } from 'solid-js';
import { Select } from '../../../ui/Select';
import { useMetadata } from '../../../../core/hooks';
import { getHierarchicalFolders } from '../searchHelpers';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders a specialized dropdown selector for folder-based search criteria.
 * Uses a hierarchical representation of storage locations for easier navigation.
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the folder field renderer.
 * @returns {JSX.Element} The rendered folder select component.
 */
export const FolderCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
    /**
     * Accesses global metadata containing available storage locations.
     *
     * @returns {Metadata} The global metadata object.
     */
    const metadata = useMetadata();

    /**
     * Computes a flat list of folders with visual indentation to represent hierarchy.
     *
     * @returns {Array<{ value: string; label: string }>} The list of selectable options.
     */
    const hierarchicalFolders = createMemo(() => getHierarchicalFolders(metadata.locations));

    return (
        <Select
            size={properties.size || 'md'}
            options={hierarchicalFolders()}
            value={String(properties.value || '')}
            onValueChange={value => properties.setValue(value)}
            searchable
            error={!!properties.errors.value}
            errorMessage={properties.errors.value}
            class="tag-select-wrapper"
        />
    );
};
