import { Component, Show } from 'solid-js';
import { NumberInput } from '../../../ui/NumberInput';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProps } from './types';
import { SIZE_UNITS } from '../searchConstants';

export const SizeCriterionField: Component<CriterionFieldRendererProps> = props => {
    const isRange = () => props.operator === 'between';

    return (
        <div class="number-input-group">
            <NumberInput
                size={props.size || 'md'}
                value={(props.value as number) ?? undefined}
                onChange={val => props.setValue(val ?? null)}
                placeholder={isRange() ? 'From Size...' : 'Size Value...'}
                error={!!props.errors.value}
                errorMessage={props.errors.value}
            />
            <Show when={isRange() && props.setValue2}>
                <span class="range-separator">to</span>
                <NumberInput
                    size={props.size || 'md'}
                    value={(props.value2 as number) ?? undefined}
                    onChange={val => props.setValue2?.(val ?? null)}
                    placeholder="To Size..."
                    error={!!props.errors.value2}
                    errorMessage={props.errors.value2}
                />
            </Show>
            <Show when={props.setUnit}>
                <Select
                    size={props.size || 'md'}
                    options={SIZE_UNITS}
                    value={props.unit || '1048576'}
                    onValueChange={val => props.setUnit?.(val)}
                    error={!!props.errors.unit}
                    errorMessage={props.errors.unit}
                    class="unit-select"
                />
            </Show>
        </div>
    );
};
