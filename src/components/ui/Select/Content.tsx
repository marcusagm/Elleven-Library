import { Component, splitProps, Show } from 'solid-js';
import { Portal } from 'solid-js/web';
import { cn as concatenateClasses } from '../../../lib/utils';
import { useSelect } from './context';
import { SelectContentProperties } from './types';

/**
 * Select.Content is a portal component that renders the dropdown dropdown menu.
 * Automatically positions itself relative to the Select.Trigger component.
 *
 * @param {SelectContentProperties} properties - Properties for the portal content.
 * @returns {JSX.Element} A dropdown list rendered in a portal.
 */
export const Content: Component<SelectContentProperties> = properties => {
    const [localProperties, remainingProperties] = splitProps(properties, ['class', 'children']);

    const context = useSelect();

    return (
        <Show when={context.isOpen()}>
            <Portal>
                <div
                    ref={element => context.setContentElement(element)}
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
