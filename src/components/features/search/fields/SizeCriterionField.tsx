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

const isEmpty = (v: unknown) => v === null || v === '';

export const sizeHandler: import('./types').SearchFieldHandler = {
    component: SizeCriterionField,
    validate: (val, val2, op, unit) => {
        const errors: Record<string, string> = {};

        if (isEmpty(val)) {
            errors.value = 'Value is required';
        }

        if (op === 'between') {
            if (isEmpty(val2)) {
                errors.value2 = 'End value is required';
            } else if (!isEmpty(val) && Number(val) > Number(val2)) {
                errors.value2 = 'End value must be greater than start';
            }
        }

        if (!SIZE_UNITS.find(u => u.value === unit)) {
            errors.unit = 'Unit is required';
        }

        return errors;
    },
    process: (val, val2, op, unit) => {
        const multiplier = Number(unit);
        let finalValue: unknown;

        if (op === 'between') {
            const v1 = Math.round(Number(val) * multiplier);
            const v2 = Math.round(Number(val2) * multiplier);
            finalValue = [v1, v2];
        } else {
            finalValue = Math.round(Number(val) * multiplier);
        }

        return { finalValue, unitMultiplier: unit };
    },
    formatDisplay: (v1, v2, op, unit) => {
        const u = SIZE_UNITS.find(opt => opt.value === unit)?.label || 'bytes';
        if (op === 'between') {
            return `${v1} ${u} to ${v2} ${u}`;
        }
        return `${v1} ${u}`;
    }
};
