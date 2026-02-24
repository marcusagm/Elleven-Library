import { SearchCriterion } from '../../../core/store/filterStore';
import { criterionHandlerRegistry } from './fields';
import { supportedFormats } from '../../../core/store/systemStore';
import { SearchFieldHandler, StoreMetadata } from './fields/types';
import { SEARCH_FIELDS } from './searchConstants';

const isEmpty = (v: unknown) => v === null || v === undefined || v === '';

const formatWithHandler = (
    handler: SearchFieldHandler | undefined,
    value: unknown,
    op: string,
    unit: string | undefined,
    meta: StoreMetadata,
    isArray: boolean
) => {
    if (!handler?.formatDisplay) return null;
    if (isArray) {
        const arr = value as unknown[];
        const v1 = handler.formatDisplay(arr[0], op, unit, meta);
        const v2 = handler.formatDisplay(arr[1], op, unit, meta);
        return `${v1} to ${v2}`;
    }
    return handler.formatDisplay(value, op, unit, meta);
};

export const computeDisplayValue = (
    item: Partial<SearchCriterion>,
    metadata: { locations: { id: number; name: string }[]; tags: { id: number; name: string }[] }
): string => {
    if (item.displayValue) return item.displayValue;
    if (isEmpty(item.value)) return '';

    const key = item.key ?? '';
    const fieldObj = SEARCH_FIELDS.find(f => f.value === key);
    const handlerName = key === 'size' ? 'size' : (fieldObj?.type ?? 'text');
    const handler = criterionHandlerRegistry[handlerName];
    const isArrayType = Array.isArray(item.value);

    const meta = { ...metadata, supportedFormats: supportedFormats() };
    const formatted = formatWithHandler(
        handler,
        item.value,
        item.operator ?? '',
        item.unitMultiplier,
        meta,
        isArrayType
    );

    if (formatted) return formatted;

    const arr = item.value as unknown[];
    return isArrayType ? `${arr[0]} to ${arr[1]}` : String(item.value);
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
