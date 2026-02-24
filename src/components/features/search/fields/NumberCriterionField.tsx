import { Component, Show } from 'solid-js';
import { NumberInput } from '../../../ui/NumberInput';
import { CriterionFieldRendererProps } from './types';

export const NumberCriterionField: Component<CriterionFieldRendererProps> = props => {
    const isRange = () => props.operator === 'between';

    return (
        <div class="number-input-group">
            <NumberInput
                size={props.size || 'md'}
                value={(props.value as number) ?? undefined}
                onChange={val => props.setValue(val ?? null)}
                placeholder={isRange() ? 'From...' : 'Value...'}
                error={!!props.errors.value}
                errorMessage={props.errors.value}
            />
            <Show when={isRange() && props.setValue2}>
                <span class="range-separator">to</span>
                <NumberInput
                    size={props.size || 'md'}
                    value={(props.value2 as number) ?? undefined}
                    onChange={val => props.setValue2?.(val ?? null)}
                    placeholder="To..."
                    error={!!props.errors.value2}
                    errorMessage={props.errors.value2}
                />
            </Show>
        </div>
    );
};
