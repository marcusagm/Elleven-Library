import { JSX } from 'solid-js';

/**
 * Properties for the Switch component.
 */
export interface SwitchProperties extends Omit<
    JSX.InputHTMLAttributes<HTMLInputElement>,
    'onChange' | 'type'
> {
    /** Whether the switch is checked (controlled). */
    checked?: boolean;
    /** Whether the switch is checked by default (uncontrolled). */
    defaultChecked?: boolean;
    /** Callback invoked when the checked state changes. */
    onCheckedChange?: (checked: boolean) => void;
    /** Accessible label for the switch. */
    label?: string;
    /** Additional description text for the switch. */
    description?: string;
    /** Size variant of the switch. @default 'md' */
    size?: 'sm' | 'md' | 'lg';
}
