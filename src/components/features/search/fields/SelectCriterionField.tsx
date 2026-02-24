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
