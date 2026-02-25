import { Component, JSX, splitProps, createMemo } from 'solid-js';
import { cn } from '../../lib/utils';
import './button-group.css';

/**
 * Defines the orientation options for the ButtonGroup component.
 */
export type ButtonGroupOrientation = 'horizontal' | 'vertical';

/**
 * Defines the size options for the ButtonGroup component.
 */
export type ButtonGroupSize = 'sm' | 'md' | 'lg';

/**
 * Properties for the ButtonGroup component.
 */
export interface ButtonGroupProps extends JSX.HTMLAttributes<HTMLDivElement> {
    /**
     * The orientation of the buttons within the group.
     * @default 'horizontal'
     */
    orientation?: ButtonGroupOrientation;
    /**
     * The size variant of the buttons within the group.
     * @default 'md'
     */
    size?: ButtonGroupSize;
    /**
     * Whether the buttons should appear attached to each other.
     * @default false
     */
    attached?: boolean;
    /**
     * The content to be rendered inside the button group, typically Button components.
     */
    children: JSX.Element;
}

/**
 * ButtonGroup component for grouping related buttons.
 * Supports horizontal/vertical layouts and attached styling.
 *
 * @example
 * <ButtonGroup attached>
 *   <Button variant="ghost">Left</Button>
 *   <Button variant="ghost">Center</Button>
 *   <Button variant="ghost">Right</Button>
 * </ButtonGroup>
 */
export const ButtonGroup: Component<ButtonGroupProps> = props => {
    const [local, others] = splitProps(props, [
        'class',
        'orientation',
        'size',
        'attached',
        'children'
    ]);

    /**
     * Computes the CSS class names for the ButtonGroup component based on its props.
     */
    const classes = createMemo(() =>
        cn(
            'ui-button-group',
            `ui-button-group-${local.orientation || 'horizontal'}`,
            local.attached && 'ui-button-group-attached',
            local.size && `ui-button-group-${local.size}`,
            local.class
        )
    );

    return (
        <div class={classes()} role="group" {...others}>
            {local.children}
        </div>
    );
};
