import { Component, createMemo } from 'solid-js';
import { Select } from '../../../ui/Select';
import { useMetadata } from '../../../../core/hooks';
import { getHierarchicalTags } from '../searchHelpers';
import { CriterionFieldRendererProps } from './types';

export const TagsCriterionField: Component<CriterionFieldRendererProps> = props => {
    const metadata = useMetadata();
    const hierarchicalTags = createMemo(() => getHierarchicalTags(metadata.tags));

    return (
        <Select
            size={props.size || 'md'}
            options={hierarchicalTags()}
            value={String(props.value || '')}
            onValueChange={val => props.setValue(val)}
            searchable
            error={!!props.errors.value}
            errorMessage={props.errors.value}
            class="tag-select-wrapper"
        />
    );
};

export const tagsHandler: import('./types').SearchFieldHandler = {
    component: TagsCriterionField,
    validate: val => {
        const errors: Record<string, string> = {};
        if (val === null || val === '') errors.value = 'Value is required';
        return errors;
    },
    process: val => ({ finalValue: val }),
    formatDisplay: (v1, _v2, _op, _unit, metadata) => {
        const found = metadata?.tags.find(t => String(t.id) === String(v1));
        return found?.name || String(v1);
    }
};
