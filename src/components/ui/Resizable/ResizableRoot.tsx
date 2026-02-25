import { Component, splitProps } from 'solid-js';
import { cn } from '../../../lib/utils';
import { ResizablePanelGroupProperties } from './types';
import { createResizableState, ResizableContext } from './ResizableContext';
import './resizable.css';

/**
 * The root component for a resizable layout.
 * Manages the coordination between multiple panels and handles.
 *
 * @param {ResizablePanelGroupProperties} props - Component properties.
 * @returns {JSX.Element} The rendered group.
 *
 * @example
 * <ResizablePanelGroup direction="horizontal">
 *   <ResizablePanel id="left" defaultSize={20}>Left</ResizablePanel>
 *   <ResizableHandle />
 *   <ResizablePanel id="right" defaultSize={80}>Right</ResizablePanel>
 * </ResizablePanelGroup>
 */
export const ResizablePanelGroup: Component<ResizablePanelGroupProperties> = props => {
    const [componentProperties, delegatedAttributes] = splitProps(props, [
        'class',
        'direction',
        'onLayout',
        'children'
    ]);

    const directionAccessor = () => componentProperties.direction || 'horizontal';

    const resizableState = createResizableState(
        directionAccessor,
        () => componentProperties.onLayout
    );

    return (
        <ResizableContext.Provider value={resizableState}>
            <div
                class={cn(
                    'ui-resizable-group',
                    `ui-resizable-group-${directionAccessor()}`,
                    componentProperties.class
                )}
                {...delegatedAttributes}
            >
                {componentProperties.children}
            </div>
        </ResizableContext.Provider>
    );
};
