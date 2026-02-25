import { Component, splitProps, createEffect } from 'solid-js';
import { cn } from '../../../lib/utils';
import { ResizableHandleProperties } from './types';
import { useResizable } from './ResizableContext';

let uniqueHandleIdentifierCounter = 0;

/**
 * Visual separator that allows the user to resize adjacent panels.
 *
 * @param {ResizableHandleProperties} props - Component properties.
 * @returns {JSX.Element} The rendered handle.
 */
export const ResizableHandle: Component<ResizableHandleProperties> = props => {
    const [componentProperties, delegatedAttributes] = splitProps(props, [
        'id',
        'class',
        'isCollapsed',
        'withHandle'
    ]);

    const resizableContext = useResizable();

    // Generate a unique ID if one wasn't provided
    const handleIdentifier = () =>
        componentProperties.id || `handle-${++uniqueHandleIdentifierCounter}`;

    // Register handle within the group context
    createEffect(() => {
        resizableContext.registerHandle(handleIdentifier());
    });

    /**
     * Proxies the pointer down event to the context's resize engine.
     */
    const handlePointerDown = (event: PointerEvent) => {
        event.preventDefault();
        resizableContext.startResize(event.currentTarget as HTMLElement, event);
    };

    return (
        <div
            class={cn(
                'ui-resizable-handle',
                `ui-resizable-handle-${resizableContext.direction()}`,
                componentProperties.isCollapsed && 'is-collapsed',
                componentProperties.class
            )}
            onPointerDown={handlePointerDown}
            style={{
                display: componentProperties.isCollapsed ? 'none' : 'flex',
                'touch-action': 'none'
            }}
            {...delegatedAttributes}
        >
            {componentProperties.withHandle && <div class="ui-resizable-handle-bar" />}
        </div>
    );
};
