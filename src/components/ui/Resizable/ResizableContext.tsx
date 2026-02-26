import { createContext, useContext, Accessor } from 'solid-js';
import { createStore } from 'solid-js/store';
import { ResizableContextValue, ResizableDirection, PanelState, PanelConfiguration } from './types';

/**
 * Context for sharing resizable state between Group, Panels, and Handles.
 */
export const ResizableContext = createContext<ResizableContextValue>();

/**
 * Hook to access the resizable context.
 *
 * @throws {Error} If used outside of a ResizablePanelGroup.
 */
export const useResizable = () => {
    const context = useContext(ResizableContext);
    if (!context) {
        throw new Error('Resizable components must be used within a ResizablePanelGroup');
    }
    return context;
};

/**
 * Creates and manages the internal state for a resizable group.
 *
 * @param direction - The layout direction (horizontal/vertical).
 * @param onLayoutChange - Callback for size updates.
 * @returns {ResizableContextValue & { panels: PanelState[] }} The state and actions.
 */
export const createResizableState = (
    direction: Accessor<ResizableDirection>,
    onLayoutChangeAccessor?: Accessor<((sizes: number[]) => void) | undefined>
) => {
    const [store, setStore] = createStore({
        panels: [] as PanelState[],
        handles: [] as string[],
        isResizing: false
    });

    const registerPanel = (panelIdentifier: string, configuration: PanelConfiguration) => {
        setStore('panels', existingPanels => {
            const panelExists = existingPanels.find(panel => panel.id === panelIdentifier);
            if (panelExists) {
                // Update configuration if it changed (e.g., isCollapsed)
                return existingPanels.map(panel =>
                    panel.id === panelIdentifier ? { ...panel, ...configuration } : panel
                );
            }

            return [
                ...existingPanels,
                {
                    ...configuration,
                    id: panelIdentifier,
                    size: configuration.defaultSize,
                    index: existingPanels.length
                }
            ];
        });
    };

    const registerHandle = (handleIdentifier: string) => {
        setStore('handles', existingHandles => {
            if (existingHandles.includes(handleIdentifier)) {
                return existingHandles;
            }
            return [...existingHandles, handleIdentifier];
        });
    };

    const getPanelSize = (panelIdentifier: string): number => {
        const panelFound = store.panels.find(panel => panel.id === panelIdentifier);
        return panelFound?.size ?? 0;
    };

    const setPanelSize = (panelIdentifier: string, newSize: number) => {
        setStore('panels', panel => panel.id === panelIdentifier, 'size', newSize);
    };

    const calculateInitialSizes = (
        containerElement: HTMLElement,
        panelBeforeId: string,
        panelAfterId: string,
        isHorizontal: boolean,
        containerSizePx: number
    ) => {
        const beforeElement = containerElement.querySelector(`[data-panel-id="${panelBeforeId}"]`);
        const afterElement = containerElement.querySelector(`[data-panel-id="${panelAfterId}"]`);

        if (!beforeElement || !afterElement) return null;

        const beforeRect = beforeElement.getBoundingClientRect();
        const afterRect = afterElement.getBoundingClientRect();

        const sizeBefore =
            ((isHorizontal ? beforeRect.width : beforeRect.height) / containerSizePx) * 100;
        const sizeAfter =
            ((isHorizontal ? afterRect.width : afterRect.height) / containerSizePx) * 100;

        return { sizeBefore, sizeAfter };
    };

    /**
     * Finds the IDs and states of panels adjacent to a given handle element.
     *
     * @param handleElement - The handle element to check neighbors for.
     * @returns {{ before: PanelState; after: PanelState } | null} The adjacent panels.
     */
    const findAdjacentPanels = (
        handleElement: HTMLElement
    ): { before: PanelState; after: PanelState } | null => {
        const beforeSibling = handleElement.previousElementSibling as HTMLElement;
        const afterSibling = handleElement.nextElementSibling as HTMLElement;

        if (!beforeSibling || !afterSibling) return null;

        const panelBeforeIdentifier = beforeSibling.getAttribute('data-panel-id');
        const panelAfterIdentifier = afterSibling.getAttribute('data-panel-id');

        if (!panelBeforeIdentifier || !panelAfterIdentifier) return null;

        const panelBefore = store.panels.find(panel => panel.id === panelBeforeIdentifier);
        const panelAfter = store.panels.find(panel => panel.id === panelAfterIdentifier);

        if (!panelBefore || !panelAfter) return null;

        return { before: panelBefore, after: panelAfter };
    };

    /**
     * Applies min/max constraints to the new calculated sizes of two panels.
     *
     * @param requestedSizeBefore - Calculated size for the preceding panel.
     * @param panelBefore - Configuration of the preceding panel.
     * @param panelAfter - Configuration of the succeeding panel.
     * @param combinedSizeInitial - Total percentage available for both panels.
     * @returns {{ sizeBefore: number; sizeAfter: number }} Constrained sizes.
     */
    const applySizeConstraints = (
        requestedSizeBefore: number,
        panelBefore: PanelState,
        panelAfter: PanelState,
        combinedSizeInitial: number
    ): { sizeBefore: number; sizeAfter: number } => {
        let sizeBefore = requestedSizeBefore;
        let sizeAfter = combinedSizeInitial - requestedSizeBefore;

        if (sizeBefore < panelBefore.minSize) {
            sizeBefore = panelBefore.minSize;
            sizeAfter = combinedSizeInitial - sizeBefore;
        } else if (sizeBefore > panelBefore.maxSize) {
            sizeBefore = panelBefore.maxSize;
            sizeAfter = combinedSizeInitial - sizeBefore;
        }

        if (sizeAfter < panelAfter.minSize) {
            sizeAfter = panelAfter.minSize;
            sizeBefore = combinedSizeInitial - sizeAfter;
        } else if (sizeAfter > panelAfter.maxSize) {
            sizeAfter = panelAfter.maxSize;
            sizeBefore = combinedSizeInitial - sizeAfter;
        }

        return { sizeBefore, sizeAfter };
    };

    const startResize = (handleElement: HTMLElement, event: PointerEvent) => {
        const containerElement = handleElement.parentElement;
        if (!containerElement) return;

        const adjacentPanels = findAdjacentPanels(handleElement);
        if (!adjacentPanels) return;

        const isHorizontal = direction() === 'horizontal';
        const containerRect = containerElement.getBoundingClientRect();
        const containerSizePx = isHorizontal ? containerRect.width : containerRect.height;

        const initialSizes = calculateInitialSizes(
            containerElement,
            adjacentPanels.before.id,
            adjacentPanels.after.id,
            isHorizontal,
            containerSizePx
        );
        if (!initialSizes) return;

        // Capture pointer to ensure events continue even if mouse leaves handle
        handleElement.setPointerCapture(event.pointerId);

        const startPosition = isHorizontal ? event.clientX : event.clientY;
        const combinedSizeInitial = initialSizes.sizeBefore + initialSizes.sizeAfter;

        const handlePointerMove = (moveEvent: PointerEvent) => {
            const currentPosition = isHorizontal ? moveEvent.clientX : moveEvent.clientY;
            const deltaPercent = ((currentPosition - startPosition) / containerSizePx) * 100;

            const constrainedSizes = applySizeConstraints(
                initialSizes.sizeBefore + deltaPercent,
                adjacentPanels.before,
                adjacentPanels.after,
                combinedSizeInitial
            );

            setStore(
                'panels',
                panel => panel.id === adjacentPanels.before.id,
                'size',
                constrainedSizes.sizeBefore
            );
            setStore(
                'panels',
                panel => panel.id === adjacentPanels.after.id,
                'size',
                constrainedSizes.sizeAfter
            );
        };

        const handlePointerUp = (upEvent: PointerEvent) => {
            handleElement.releasePointerCapture(upEvent.pointerId);
            window.removeEventListener('pointermove', handlePointerMove);
            window.removeEventListener('pointerup', handlePointerUp);

            setStore('isResizing', false);
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
            containerElement.classList.remove('is-resizing');

            const onLayoutChange = onLayoutChangeAccessor?.();
            if (onLayoutChange) {
                onLayoutChange(store.panels.map(panel => panel.size));
            }
        };

        setStore('isResizing', true);
        window.addEventListener('pointermove', handlePointerMove);
        window.addEventListener('pointerup', handlePointerUp);
        document.body.style.cursor = isHorizontal ? 'col-resize' : 'row-resize';
        document.body.style.userSelect = 'none';
        containerElement.classList.add('is-resizing');
    };

    return {
        direction,
        registerPanel,
        registerHandle,
        getPanelSize,
        setPanelSize,
        startResize,
        get panels() {
            return store.panels;
        },
        get isResizing() {
            return store.isResizing;
        }
    };
};
