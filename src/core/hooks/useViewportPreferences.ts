import {
    viewportPreferencesState,
    viewportPreferencesActions,
    MetadataField,
    MetadataPosition
} from '../store/viewportPreferencesStore';

/**
 * Hook providing access to user view preferences for asset layout logic
 * and visible metadata fields.
 *
 * @returns {Object} Accessors and methods for adjusting display config.
 */
export const useViewportPreferences = () => {
    return {
        /**
         * Where metadata is displayed relative to the thumbnail.
         *
         * @returns {MetadataPosition}
         */
        get metadataPosition(): MetadataPosition {
            return viewportPreferencesState.metadataPosition;
        },

        /**
         * Active list of fields that the user wants to see on each card.
         *
         * @returns {MetadataField[]}
         */
        get visibleFields(): MetadataField[] {
            return viewportPreferencesState.visibleFields;
        },

        /**
         * Modifies the metadata position strategy.
         *
         * @param {MetadataPosition} position - The layout string.
         * @returns {void}
         */
        setMetadataPosition: viewportPreferencesActions.setMetadataPosition,

        /**
         * Toggles rendering of given individual metadata elements onto cards.
         *
         * @param {MetadataField} field - Value to insert or remove.
         * @returns {void}
         */
        toggleVisibleField: viewportPreferencesActions.toggleVisibleField,

        /**
         * Returns configurations back to default rendering settings.
         *
         * @returns {void}
         */
        resetVisibleFields: viewportPreferencesActions.resetVisibleFields
    };
};
