import { loading, progress, rootPath, systemActions } from '../store/systemStore';

/**
 * Hook providing access to the overall system state like loading status and root path.
 *
 * @returns {Object} System state accessors and basic initialization methods.
 */
export const useSystem = () => {
    return {
        /** Overall application loading state flag */
        loading,
        /** Global progress tracking mapping */
        progress,
        /** Scanned library root path directory */
        rootPath,

        /** Core actions startup call */
        initialize: systemActions.initialize,
        /** Triggers a root configuration path change */
        setRootLocation: systemActions.setRootLocation,
        /** Send progress events */
        updateProgress: systemActions.updateProgress,
        /** Discards cached progress notifications */
        clearProgress: systemActions.clearProgress
    };
};
