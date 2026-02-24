import { Component, Show } from 'solid-js';
import { DateInput } from '../../../ui/DateInput';
import { CriterionFieldRendererProps } from './types';

export const DateCriterionField: Component<CriterionFieldRendererProps> = props => {
    const isRange = () => props.operator === 'between';

    return (
        <div class="date-input-group">
            <DateInput
                size={props.size || 'md'}
                value={(props.value as Date) || null}
                onChange={val => props.setValue(val)}
                placeholder={isRange() ? 'From Date' : 'Date'}
                error={!!props.errors.value}
                errorMessage={props.errors.value}
            />
            <Show when={isRange() && props.setValue2}>
                <span class="range-separator">to</span>
                <DateInput
                    size={props.size || 'md'}
                    value={(props.value2 as Date) || null}
                    onChange={val => props.setValue2?.(val)}
                    placeholder="To Date"
                    error={!!props.errors.value2}
                    errorMessage={props.errors.value2}
                />
            </Show>
        </div>
    );
};
