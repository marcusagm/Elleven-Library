import { SearchCriterion } from '../../../core/store/filter';
import { criterionHandlerRegistry } from './fields';
import { supportedFormats } from '../../../core/store/systemStore';
import { SearchFieldHandler, StoreMetadata } from './fields/types';
import { SEARCH_FIELDS } from '../../../core/store/filter/constants';

/**
 * Checks if a given search value is considered empty.
 * @param value - The value to examine.
 */
const checkIsSearchValueEmpty = (value: unknown) =>
    value === null || value === undefined || value === '';

/**
 * Internal delegate to format a value using a specific type handler.
 *
 * @param handler - The designated field handler (if unknown).
 * @param value - The raw search value.
 * @param operator - The comparison operator.
 * @param unitMultiplier - Optional numeric multiplier.
 * @param metadata - Contextual store information.
 * @param isArrayValue - Whether the value is a range array.
 * @returns A formatted string or null if the handler doesn't support display formatting.
 */
const formatUsingTypeHandler = (
    handler: SearchFieldHandler | undefined,
    value: unknown,
    operator: string,
    unitMultiplier: string | undefined,
    metadata: StoreMetadata,
    isArrayValue: boolean
) => {
    if (!handler?.formatDisplay) return null;
    if (isArrayValue) {
        const itemArray = value as unknown[];
        return handler.formatDisplay(
            itemArray[0],
            itemArray[1],
            operator,
            unitMultiplier,
            metadata
        );
    }
    return handler.formatDisplay(value, undefined, operator, unitMultiplier, metadata);
};

/**
 * Derives a human-readable string representation of a search criterion for the bar/tag UI.
 *
 * @param criterionItem - The search criterion to format.
 * @param metadata - The metadata containing locations and tags for resolving IDs.
 * @returns A descriptive string representing the search logic.
 */
export const computeDisplayValue = (
    criterionItem: Partial<SearchCriterion>,
    metadata: {
        locations: { id: string; name: string }[];
        tags: { id: string; name: string }[];
    }
): string => {
    if (criterionItem.displayValue) return criterionItem.displayValue;
    if (checkIsSearchValueEmpty(criterionItem.value)) return '';

    const fieldKey = criterionItem.key ?? '';
    const searchFieldDefinition = SEARCH_FIELDS.find(field => field.value === fieldKey);
    const handlerName = fieldKey === 'size' ? 'size' : (searchFieldDefinition?.type ?? 'text');
    const fieldHandler = criterionHandlerRegistry[handlerName];
    const isArrayType = Array.isArray(criterionItem.value);

    const consolidatedMetadata = { ...metadata, supportedFormats: supportedFormats() };
    const formattedString = formatUsingTypeHandler(
        fieldHandler,
        criterionItem.value,
        criterionItem.operator ?? '',
        criterionItem.unitMultiplier,
        consolidatedMetadata,
        isArrayType
    );

    if (formattedString) return formattedString;

    const valuesArray = criterionItem.value as unknown[];
    return isArrayType ? `${valuesArray[0]} to ${valuesArray[1]}` : String(criterionItem.value);
};

/**
 * Transforms a flat list of tags into a hierarchical array of selection options.
 *
 * @param tags - The raw list of tags from the database.
 * @param parentId - The ID of the parent tag for the current recursion level.
 * @param depth - Current numeric nesting level for visual indentation.
 * @returns A flat list of options with prefixed labels for hierarchy.
 */
export const getHierarchicalTags = (
    tags: { id: string; name: string; parent_id?: string | null }[],
    parentId: string | null = null,
    depth = 0
): { value: string; label: string }[] => {
    return tags
        .filter(tag => tag.parent_id === parentId || (parentId === null && !tag.parent_id))
        .flatMap(tag => [
            { value: String(tag.id), label: `${'\u00A0'.repeat(depth * 3)}${tag.name}` },
            ...getHierarchicalTags(tags, tag.id, depth + 1)
        ]);
};

/**
 * Transforms a flat list of folders into a hierarchical array of selection options.
 *
 * @param folders - The raw list of storage locations/folders.
 * @param parentId - The ID of the parent folder for recursion.
 * @param depth - Visual nesting level.
 * @returns A flat list of options with prefixed labels.
 */
export const getHierarchicalFolders = (
    folders: { id: string; name: string; parent_id?: string | null }[],
    parentId: string | null = null,
    depth = 0
): { value: string; label: string }[] => {
    return folders
        .filter(folder => folder.parent_id === parentId || (parentId === null && !folder.parent_id))
        .flatMap(folder => [
            { value: String(folder.id), label: `${'\u00A0'.repeat(depth * 4)}${folder.name}` },
            ...getHierarchicalFolders(folders, folder.id, depth + 1)
        ]);
};
