import { Component } from 'solid-js';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProps } from './types';

export const RatingCriterionField: Component<CriterionFieldRendererProps> = props => {
    return (
        <Select
            size={props.size || 'md'}
            options={[0, 1, 2, 3, 4, 5].map(v => ({
                value: String(v),
                label: `${v} Stars`
            }))}
            value={String(props.value ?? '0')}
            onValueChange={val => props.setValue(Number(val))}
            error={!!props.errors.value}
            errorMessage={props.errors.value}
        />
    );
};

export const ratingHandler: import('./types').SearchFieldHandler = {
    component: RatingCriterionField,
    validate: val => {
        const errors: Record<string, string> = {};
        if (val === null || val === '') errors.value = 'Value is required';
        return errors;
    },
    process: val => ({ finalValue: val }),
    formatDisplay: val => `${val} Stars`
};
