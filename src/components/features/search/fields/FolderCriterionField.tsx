import { Component, createMemo } from 'solid-js';
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
export const FolderCriterionField: Component<CriterionFieldRendererProperties> = properties => {
    /** Accesses global metadata containing available storage locations. */
    const metadata = useMetadata();

    /** Computes a flat list of folders with visual indentation to represent hierarchy. */
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

/**
 * Handler implementation for folder-based search criteria.
 * Resolves numeric folder IDs to human-readable names for display.
 */
export const folderHandler: import('./types').SearchFieldHandler = {
    /** The visual component representing the folder selector. */
    component: FolderCriterionField,

    /**
     * Validates that a folder selection has been made.
     *
     * @param value - The primary folder ID selection.
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
     * Processes the raw folder ID for query usage.
     *
     * @param value - Selected folder ID.
     * @returns The unchanged value for direct usage.
     */
    process: value => ({ finalValue: value }),

    /**
     * Resolves the folder ID to its display name using provided metadata.
     *
     * @param folderId - The ID of the folder to format.
     * @param _value2 - Unused secondary value.
     * @param _operator - Unused operator.
     * @param _unit - Unused unit.
     * @param metadata - Store metadata containing the locations list.
     * @returns The descriptive name of the folder or the raw ID if not found.
     */
    formatDisplay: (folderId, _value2, _operator, _unit, metadata) => {
        const matchedFolder = metadata?.locations.find(
            location => String(location.id) === String(folderId)
        );
        return matchedFolder?.name || String(folderId);
    }
};
