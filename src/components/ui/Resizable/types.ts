import { JSX, Accessor } from 'solid-js';

/**
 * Valid directions for the resizable group.
 */
export type ResizableDirection = 'horizontal' | 'vertical';

/**
 * Configuration for an individual panel.
 */
export interface PanelConfiguration {
    /** The initial size of the panel in percentage (0-100) */
    defaultSize: number;
    /** The minimum size the panel can be shrunk to */
    minSize: number;
    /** The maximum size the panel can grow to */
    maxSize: number;
    /** Whether the panel is currently collapsed */
    isCollapsed?: boolean;
}

/**
 * Internal state tracking for a panel.
 */
export interface PanelState extends PanelConfiguration {
    /** Unique identifier for the panel */
    id: string;
    /** Current size in percentage */
    size: number;
    /** Order in the list */
    index: number;
}

/**
 * Properties for the ResizablePanelGroup component.
 */
export interface ResizablePanelGroupProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** The layout direction */
    direction?: ResizableDirection;
    /** Callback triggered when the layout changes */
    onLayout?: (sizes: number[]) => void;
    /** Component children (Panels and Handles) */
    children: JSX.Element;
}

/**
 * Properties for the ResizablePanel component.
 */
export interface ResizablePanelProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Unique identifier for the panel */
    id: string;
    /** Initial size in percentage */
    defaultSize?: number;
    /** Minimum size in percentage */
    minSize?: number;
    /** Maximum size in percentage */
    maxSize?: number;
    /** Whether the panel is currently collapsed */
    isCollapsed?: boolean;
    /** Optional flex-grow override (rarely used with absolute sizes) */
    flexGrow?: number;
    /** Panel content */
    children: JSX.Element;
}

/**
 * Properties for the ResizableHandle component.
 */
export interface ResizableHandleProperties extends JSX.HTMLAttributes<HTMLDivElement> {
    /** Optional unique identifier */
    id?: string;
    /** Whether to show a visual handle bar indicator */
    withHandle?: boolean;
    /** Whether the handle is hidden (e.g. adjacent panel is collapsed) */
    isCollapsed?: boolean;
}

/**
 * Context value for the resizable system.
 */
export interface ResizableContextValue {
    /** Current direction of the group */
    direction: Accessor<ResizableDirection>;
    /** Registers a panel within the context */
    registerPanel: (panelIdentifier: string, configuration: PanelConfiguration) => void;
    /** Retrieves the current size of a specific panel */
    getPanelSize: (panelIdentifier: string) => number;
    /** Initiates the resize process for a specific handle */
    startResize: (handleElement: HTMLElement, event: PointerEvent) => void;
    /** Registers a handle within the context */
    registerHandle: (handleIdentifier: string) => void;
    /** Manually sets the size of a panel */
    setPanelSize: (panelIdentifier: string, newSize: number) => void;
}
