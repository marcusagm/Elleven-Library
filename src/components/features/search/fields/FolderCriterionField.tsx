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
