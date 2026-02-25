import { Component, Show } from 'solid-js';
import { DateInput } from '../../../ui/DateInput';
import { CriterionFieldRendererProps } from './types';
import { formatToISO, formatToDisplay } from '../../../../utils/format';

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

export const dateHandler: import('./types').SearchFieldHandler = {
    component: DateCriterionField,
    validate: (val, val2, op) => {
        const errors: Record<string, string> = {};
        if (val === null || val === '') errors.value = 'Date is required';

        if (op === 'between') {
            if (val2 === null || val2 === '') {
                errors.value2 = 'End date is required';
            } else if (val !== null && val !== '') {
                const d1 = new Date(val as string | Date);
                const d2 = new Date(val2 as string | Date);
                if (d1 > d2) errors.value2 = 'End date must be after start date';
            }
        }
        return errors;
    },
    process: (val, val2, op) => {
        if (op === 'between') {
            const v1 = formatToISO(val as Date | string);
            const v2 = formatToISO(val2 as Date | string);
            return { finalValue: [v1, v2] };
        }
        return { finalValue: formatToISO(val as Date | string) };
    },
    formatDisplay: (v1, v2, op) => {
        if (op === 'between') {
            return `${formatToDisplay(v1 as string | Date)} to ${formatToDisplay(v2 as string | Date)}`;
        }
        return formatToDisplay(v1 as string | Date);
    }
};
