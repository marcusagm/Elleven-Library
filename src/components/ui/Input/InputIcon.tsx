import { Component, JSX, Show } from 'solid-js';
import { cn } from '../../../lib/utils';

/**
 * Properties for the InputIcon component.
 */
interface InputIconProps {
    /**
     * The icon element to render.
     */
    icon?: JSX.Element;

    /**
     * The position of the icon within the input container.
     */
    position: 'left' | 'right';
}

/**
 * Sub-component to render icons within the Input component.
 *
 * @param props - Component properties.
 * @returns An icon wrapper if an icon is provided, otherwise null.
 */
export const InputIcon: Component<InputIconProps> = props => {
    return (
        <Show when={props.icon}>
            <span
                class={cn(
                    'ui-input-icon',
                    props.position === 'left' ? 'ui-input-icon-left' : 'ui-input-icon-right'
                )}
            >
                {props.icon}
            </span>
        </Show>
    );
};
