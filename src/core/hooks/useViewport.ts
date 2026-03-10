import { viewportState, viewportActions } from '../store/viewportStore';

/**
 * useViewport
 *
 * Public API hook for the viewport and rendering engine.
 * Provides access to navigation, virtualization state, and viewport modes.
 */
export const useViewport = () => {
    return {
        // Navigation / Mode
        get mode() {
            return () => (viewportState.focusedItemId !== null ? 'item' : 'list');
        },
        get activeItemId() {
            return () => (viewportState.focusedItemId ? String(viewportState.focusedItemId) : null);
        },
        get focusedItemId() {
            return () => viewportState.focusedItemId;
        },

        openItem: (id: string | number) => {
            viewportActions.setFocusedItem(String(id));
        },
        closeItem: () => {
            viewportActions.setFocusedItem(null);
        },

        // Navigation actions
        nextItem: () => viewportActions.navigateToAsset('next'),
        prevItem: () => viewportActions.navigateToAsset('prev'),

        // NOTE: History navigation (goBack/goForward) is simplified
        // to previous/next in the list to avoid duplicate state tracking.
        goBack: () => viewportActions.setFocusedItem(null),
        goForward: () => {}, // Deprecated in favor of list navigation

        canGoBack: () => false,
        canGoForward: () => false,

        // Virtualization accessors
        get isCalculating() {
            return () => viewportState.isCalculating;
        },
        get visibleItems() {
            return () => viewportState.visibleItems;
        },
        get totalHeight() {
            return () => viewportState.totalHeight;
        },
        get scrollPosition() {
            return () => viewportState.scrollPosition;
        },

        // Immersive Viewer (Zoom/Pan/Fit)
        get zoom() {
            return () => viewportState.zoom;
        },
        get fitToScreen() {
            return () => viewportState.fitToScreen;
        },
        get pan() {
            return () => viewportState.pan;
        },

        setZoom: viewportActions.setZoom,
        setFitToScreen: viewportActions.setFitToScreen,
        setPan: viewportActions.setPan,
        resetView: viewportActions.resetView
    };
};
