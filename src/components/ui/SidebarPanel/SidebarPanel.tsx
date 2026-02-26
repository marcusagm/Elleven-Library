import { Component, Show, splitProps, JSX } from 'solid-js';
import { cn as concatenateClasses } from '../../../lib/utils';
import { SidebarPanelProperties } from './types';
import './sidebar-panel.css';

/**
 * SidebarPanel component for creating organized sections within a sidebar.
 * Features a header with actions, body content, and an optional footer.
 *
 * @param {SidebarPanelProperties} properties - Properties for the sidebar panel.
 * @returns {JSX.Element} A stylized panel with header, content, and footer sections.
 *
 * @example
 * <SidebarPanel title="Layers" headerActions={<Button icon="plus" />}>
 *   <LayerList />
 * </SidebarPanel>
 */
export const SidebarPanel: Component<SidebarPanelProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, [
        'title',
        'children',
        'headerActions',
        'footerContent',
        'class',
        'contentClass'
    ]);

    return (
        <section
            class={concatenateClasses('ui-sidebar-panel', localProperties.class)}
            aria-label={localProperties.title}
            {...(remainingProperties as JSX.HTMLAttributes<HTMLElement>)}
        >
            <header class="ui-sidebar-panel-header">
                <h3 class="ui-sidebar-panel-title">{localProperties.title}</h3>
                <Show when={localProperties.headerActions}>
                    <div class="ui-sidebar-panel-actions" role="group">
                        {localProperties.headerActions}
                    </div>
                </Show>
            </header>

            <div
                class={concatenateClasses('ui-sidebar-panel-content', localProperties.contentClass)}
            >
                {localProperties.children}
            </div>

            <Show when={localProperties.footerContent}>
                <footer class="ui-sidebar-panel-footer">{localProperties.footerContent}</footer>
            </Show>
        </section>
    );
};
