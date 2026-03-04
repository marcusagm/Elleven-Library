import { createStore } from 'solid-js/store';

/**
 * Metadata fields that can be displayed on an asset card.
 */
export type MetadataField =
    | 'filename'
    | 'extension'
    | 'dimensions'
    | 'size'
    | 'rating'
    | 'modified_at'
    | 'created_at'
    | 'added_at'
    | 'tags';

/**
 * The display position of the metadata on an asset card.
 */
export type MetadataPosition = 'overlay' | 'stacked';

/**
 * State interface for viewport display preferences.
 */
export interface ViewportPreferencesState {
    /**
     * Determines where the metadata is rendered relative to the thumbnail.
     * Overlaid on hover, or strictly stacked below.
     *
     * @type {MetadataPosition}
     */
    metadataPosition: MetadataPosition;
    /**
     * The list of metadata fields that are currently visible to the user.
     *
     * @type {MetadataField[]}
     */
    visibleFields: MetadataField[];
}

const [viewportPreferencesState, setViewportPreferencesState] =
    createStore<ViewportPreferencesState>({
        metadataPosition: 'overlay',
        visibleFields: ['filename']
    });

/**
 * Actions to manipulate the viewport preferences.
 */
export const viewportPreferencesActions = {
    /**
     * Updates the metadata position layout strategy.
     *
     * @param {MetadataPosition} position - The new position strategy (e.g., 'overlay', 'stacked').
     * @returns {void}
     */
    setMetadataPosition: (position: MetadataPosition): void => {
        setViewportPreferencesState('metadataPosition', position);
    },

    /**
     * Toggles the visibility of a given metadata field.
     *
     * @param {MetadataField} field - The field to toggle.
     * @returns {void}
     */
    toggleVisibleField: (field: MetadataField): void => {
        setViewportPreferencesState('visibleFields', currentFields => {
            if (currentFields.includes(field)) {
                return currentFields.filter(existingField => existingField !== field);
            }
            return [...currentFields, field];
        });
    },

    /**
     * Resets the visible fields to their default state.
     *
     * @returns {void}
     */
    resetVisibleFields: (): void => {
        setViewportPreferencesState('visibleFields', ['filename']);
    }
};

export { viewportPreferencesState };
