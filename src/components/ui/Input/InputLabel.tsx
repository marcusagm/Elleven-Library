import { Component, Show } from 'solid-js';

/**
 * Properties for the InputLabel component.
 */
interface InputLabelProps {
    /**
     * The text to display as the label.
     */
    text?: string;
}

/**
 * Sub-component to render the label for the Input component.
 *
 * @param props - Component properties.
 * @returns A label element if text is provided, otherwise null.
 */
export const InputLabel: Component<InputLabelProps> = props => {
    return (
        <Show when={props.text}>
            <label class="ui-input-label">{props.text}</label>
        </Show>
    );
};
