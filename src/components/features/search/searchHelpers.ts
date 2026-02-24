import { SearchCriterion } from '../../../core/store/filterStore';
import { formatToDisplay } from '../../../utils/format';
import { SIZE_UNITS } from './searchConstants';

export const computeDisplayValue = (
    item: Partial<SearchCriterion>,
    metadata: { locations: { id: number; name: string }[]; tags: { id: number; name: string }[] }
): string => {
    if (item.displayValue) return item.displayValue;
    if (item.value === null || item.value === undefined || item.value === '') return '';

    const key = item.key || '';
    const val = item.value;

    if (key === 'size') {
        const multiplier = Number(item.unitMultiplier || '1048576');
        const label = SIZE_UNITS.find(unit => unit.value === String(multiplier))?.label || 'MB';
        if (Array.isArray(val)) {
            return `${Number(val[0]) / multiplier} ${label} to ${Number(val[1]) / multiplier} ${label}`;
        }
        return `${Number(val) / multiplier} ${label}`;
    }

    if (['added_at', 'created_at', 'modified_at'].includes(key)) {
        if (Array.isArray(val)) {
            return `${formatToDisplay(String(val[0]))} to ${formatToDisplay(String(val[1]))}`;
        }
        return formatToDisplay(String(val));
    }

    if (key === 'folder') {
        return (
            metadata.locations.find(location => String(location.id) === String(val))?.name ||
            String(val)
        );
    }

    if (key === 'tags') {
        return metadata.tags.find(tag => String(tag.id) === String(val))?.name || String(val);
    }

    if (Array.isArray(val)) {
        return `${val[0]} to ${val[1]}`;
    }

    return String(val);
};

export const getHierarchicalTags = (
    tags: { id: number; name: string; parent_id?: number | null }[],
    parentId: number | null = null,
    depth = 0
): { value: string; label: string }[] => {
    return tags
        .filter(t => t.parent_id === parentId || (parentId === null && !t.parent_id))
        .flatMap(t => [
            { value: String(t.id), label: `${'\u00A0'.repeat(depth * 3)}${t.name}` },
            ...getHierarchicalTags(tags, t.id, depth + 1)
        ]);
};

export const getHierarchicalFolders = (
    folders: { id: number; name: string; parent_id?: number | null }[],
    parentId: number | null = null,
    depth = 0
): { value: string; label: string }[] => {
    return folders
        .filter(f => f.parent_id === parentId || (parentId === null && !f.parent_id))
        .flatMap(f => [
            { value: String(f.id), label: `${'\u00A0'.repeat(depth * 4)}${f.name}` },
            ...getHierarchicalFolders(folders, f.id, depth + 1)
        ]);
};
