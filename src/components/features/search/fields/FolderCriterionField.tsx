import { Component, createMemo } from 'solid-js';
import { Select } from '../../../ui/Select';
import { useMetadata } from '../../../../core/hooks';
import { getHierarchicalFolders } from '../searchHelpers';
import { CriterionFieldRendererProps } from './types';

export const FolderCriterionField: Component<CriterionFieldRendererProps> = props => {
    const metadata = useMetadata();
    const hierarchicalFolders = createMemo(() => getHierarchicalFolders(metadata.locations));

    return (
        <Select
            size={props.size || 'md'}
            options={hierarchicalFolders()}
            value={String(props.value || '')}
            onValueChange={val => props.setValue(val)}
            searchable
            error={!!props.errors.value}
            errorMessage={props.errors.value}
            class="tag-select-wrapper"
        />
    );
};

export const folderHandler: import('./types').SearchFieldHandler = {
    component: FolderCriterionField,
    validate: val => {
        const errors: Record<string, string> = {};
        if (val === null || val === '') errors.value = 'Value is required';
        return errors;
    },
    process: val => ({ finalValue: val }),
    formatDisplay: (v1, _v2, _op, _unit, metadata) => {
        const found = metadata?.locations.find(l => String(l.id) === String(v1));
        return found?.name || String(v1);
    }
};
