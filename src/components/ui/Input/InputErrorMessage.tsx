import { Component, Show } from 'solid-js';

/**
 * Properties for the InputErrorMessage component.
 */
interface InputErrorMessageProps {
    /**
     * Whether to show the error message.
     */
    show: boolean;

    /**
     * The error message text.
     */
    message?: string;

    /**
     * The ID of the input element this message describes (for aria-describedby).
     */
    inputId?: string;
}

/**
 * Sub-component to render error messages for the Input component.
 *
 * @param props - Component properties.
 * @returns An error message element if shown and message is provided.
 */
export const InputErrorMessage: Component<InputErrorMessageProps> = props => {
    return (
        <Show when={props.show && props.message}>
            <span id={`${props.inputId}-error`} class="ui-input-error-message" role="alert">
                {props.message}
            </span>
        </Show>
    );
};
