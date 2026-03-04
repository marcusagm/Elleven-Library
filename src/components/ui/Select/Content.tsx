import { Component, splitProps, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { cn as concatenateClasses } from '../../../lib/utils';
import { useSelect } from './context';
import { SelectContentProperties } from './types';

/**
 * Renders the dropdown menu in a portal for the Select component.
 * Automatically positions itself relative to the Select.Trigger component.
 *
 * @param {SelectContentProperties} contentProperties - Properties for the portal content.
 * @returns {JSX.Element} A dropdown list rendered in a portal.
 *
 * @example
 * ```tsx
 * import { Select } from '@/components/ui';
 * <Select.Content class="custom-dropdown">
 *   {children}
 * </Select.Content>
 * ```
 */
export const Content: Component<SelectContentProperties> = contentProperties => {
    /**
     * Split the properties into local properties and remaining properties.
     */
    const [localProperties, remainingProperties] = splitProps(contentProperties, [
        'class',
        'children'
    ]);

    /**
     * Access the shared Select context.
     */
    const context = useSelect();

    return (
        <Show when={context.isOpen()}>
            <Portal>
                <div
                    ref={element => {
                        context.setContentElement(element);
                        element.addEventListener('mousedown', e => e.stopPropagation());
                        element.addEventListener('touchstart', e => e.stopPropagation());
                    }}
                    class={concatenateClasses('ui-select-content', localProperties.class)}
                    style={{
                        position: 'fixed',
                        top: `${context.contentPosition().top}px`,
                        left: `${context.contentPosition().left}px`,
                        width: `${context.contentPosition().width}px`,
                        'z-index': 9999
                    }}
                    role="listbox"
                    aria-expanded={context.isOpen()}
                    {...remainingProperties}
                >
                    {localProperties.children}
                </div>
            </Portal>
        </Show>
    );
};
