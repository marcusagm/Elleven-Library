import { Component } from 'solid-js';
import { Select } from '../../../ui/Select';
import { CriterionFieldRendererProps } from './types';
import { supportedFormats } from '../../../../core/store/systemStore';

export const SelectCriterionField: Component<CriterionFieldRendererProps> = props => {
    const extensions = () =>
        supportedFormats().flatMap(f =>
            f.extensions.map(ext => ({
                value: ext,
                label: `.${ext.toUpperCase()} (${f.name})`
            }))
        );

    return (
        <Select
            size={props.size || 'md'}
            options={extensions()}
            value={String(props.value || '')}
            onValueChange={val => props.setValue(val)}
            searchable
            error={!!props.errors.value}
            errorMessage={props.errors.value}
        />
    );
};

export const selectHandler: import('./types').SearchFieldHandler = {
    component: SelectCriterionField,
    validate: val => {
        const errors: Record<string, string> = {};
        if (val === null || val === '') errors.value = 'Value is required';
        return errors;
    },
    process: val => ({ finalValue: val }),
    formatDisplay: (v1, _v2, _op, _unit, metadata) => {
        const foundFormat = metadata?.supportedFormats?.find(sf =>
            sf.extensions.includes(String(v1))
        );
        return foundFormat ? `.${String(v1).toUpperCase()} (${foundFormat.name})` : String(v1);
    }
};
