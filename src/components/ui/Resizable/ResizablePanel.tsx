import { Component, splitProps, createEffect } from 'solid-js';
import { cn } from '../../../lib/utils';
import { ResizablePanelProperties } from './types';
import { useResizable } from './ResizableContext';

/**
 * A container representing an individual resizable area.
 *
 * @param {ResizablePanelProperties} props - Component properties.
 * @returns {JSX.Element} The rendered panel.
 */
export const ResizablePanel: Component<ResizablePanelProperties> = props => {
    const [componentProperties, delegatedAttributes] = splitProps(props, [
        'class',
        'id',
        'defaultSize',
        'minSize',
        'maxSize',
        'isCollapsed',
        'flexGrow',
        'children'
    ]);

    const resizableContext = useResizable();

    // Register panel with the parent group on mount or property change
    createEffect(() => {
        resizableContext.registerPanel(componentProperties.id, {
            defaultSize: componentProperties.defaultSize ?? 50,
            minSize: componentProperties.minSize ?? 0,
            maxSize: componentProperties.maxSize ?? 100,
            isCollapsed: componentProperties.isCollapsed
        });
    });

    const panelSize = () => resizableContext.getPanelSize(componentProperties.id);

    return (
        <div
            class={cn(
                'ui-resizable-panel',
                componentProperties.isCollapsed && 'is-collapsed',
                componentProperties.class
            )}
            style={{
                [resizableContext.direction() === 'horizontal' ? 'width' : 'height']:
                    componentProperties.isCollapsed ? '0%' : `${panelSize()}%`,
                [resizableContext.direction() === 'horizontal' ? 'min-width' : 'min-height']:
                    componentProperties.isCollapsed ? '0' : undefined,
                'flex-shrink': 0,
                'flex-grow': componentProperties.flexGrow ?? 0,
                // If collapsed, we want to ensure it doesn't show content or borders
                opacity: componentProperties.isCollapsed ? 0 : 1,
                'pointer-events': componentProperties.isCollapsed ? 'none' : 'auto',
                overflow: componentProperties.isCollapsed ? 'hidden' : 'auto'
            }}
            data-panel-id={componentProperties.id}
            {...delegatedAttributes}
        >
            {componentProperties.children}
        </div>
    );
};
