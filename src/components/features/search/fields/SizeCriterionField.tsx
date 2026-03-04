import { Component, JSX, Show } from 'solid-js';
import { NumberInput } from '../../../ui/NumberInput';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProperties } from './types';
import { SIZE_UNITS } from '../../../../core/store/filter/constants';

/**
 * Renders a specialized input group for file size criteria.
 * Includes one or two number inputs (depending on the operator) and a unit selector (B, KB, MB, GB).
 *
 * @param {CriterionFieldRendererProperties} properties - The configuration and state for the size field renderer.
 * @returns {JSX.Element} The rendered size input group.
 */
export const SizeCriterionField: Component<CriterionFieldRendererProperties> = (
    properties: CriterionFieldRendererProperties
): JSX.Element => {
    /**
     * Checks if the current comparison logic expects a range of two sizes.
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
                placeholder={isRangeMode() ? 'From Size...' : 'Size Value...'}
                error={!!properties.errors.value}
                errorMessage={properties.errors.value}
            />
            <Show when={isRangeMode() && properties.setValue2}>
                <span class="range-separator">to</span>
                <NumberInput
                    size={properties.size || 'md'}
                    value={(properties.value2 as number) ?? undefined}
                    onChange={value => properties.setValue2?.(value ?? null)}
                    placeholder="To Size..."
                    error={!!properties.errors.value2}
                    errorMessage={properties.errors.value2}
                />
            </Show>
            <Show when={properties.setUnitMultiplier}>
                <Select
                    size={properties.size || 'md'}
                    options={SIZE_UNITS}
                    value={properties.unitMultiplier || '1048576'}
                    onValueChange={value => properties.setUnitMultiplier?.(value)}
                    error={!!properties.errors.unit}
                    errorMessage={properties.errors.unit}
                    class="unit-select"
                />
            </Show>
        </div>
    );
};
