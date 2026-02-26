import { createContext, useContext, Accessor } from 'solid-js';

/**
 * Context properties for RadioGroup state sharing.
 */
interface RadioGroupContextValue {
    /** The group identifier name for form association. */
    name: string;
    /** Reactive accessor for the current selected value. */
    value: Accessor<string>;
    /** Changes the selected value within the group. */
    onChange: (value: string) => void;
    /** Whether the group or individual item is disabled. */
    disabled: boolean;
}

/**
 * Internal context for syncing state between RadioGroup root and items.
 */
export const RadioGroupContext = createContext<RadioGroupContextValue>();

/**
 * Hook to consume RadioGroup context within child components.
 * @throws {Error} If used outside of a RadioGroup.
 */
export const useRadioGroup = () => {
    const context = useContext(RadioGroupContext);
    if (!context) {
        throw new Error('RadioGroup components must be used within a RadioGroup');
    }
    return context;
};
