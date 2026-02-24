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
