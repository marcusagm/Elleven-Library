import { Component, JSX, Show } from 'solid-js';
import { NumberInput } from '../../../ui/NumberInput';
import { CriterionFieldRendererProperties } from './types';

/**
 * Renders an input group for generic numeric search criteria.
 * Supports single values and between-range values.
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the number field renderer.
 * @returns {JSX.Element} The rendered numeric input group.
 */
export const NumberCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
    /**
     * Checks if the current comparison logic expects a range of two numbers.
     *
     * @returns {boolean} True if the comparison operator is 'between', false otherwise.
     */
    const isRangeMode = (): boolean => properties.comparisonOperator === 'between';

    return (
        <div class="number-input-group">
            <NumberInput
                size={properties.size || 'md'}
                value={(properties.value as number) ?? undefined}
                onChange={value => properties.setValue(value ?? null)}
                placeholder={isRangeMode() ? 'From...' : 'Value...'}
                error={!!properties.errors.value}
                errorMessage={properties.errors.value}
            />
            <Show when={isRangeMode() && properties.setValue2}>
                <span class="range-separator">to</span>
                <NumberInput
                    size={properties.size || 'md'}
                    value={(properties.value2 as number) ?? undefined}
                    onChange={value => properties.setValue2?.(value ?? null)}
                    placeholder="To..."
                    error={!!properties.errors.value2}
                    errorMessage={properties.errors.value2}
                />
            </Show>
        </div>
    );
};
