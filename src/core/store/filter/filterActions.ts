import { batch } from 'solid-js';
import { APP_CONFIG } from '../../../config/constants';
import { SEARCH_FIELDS } from './constants';
import {
    SearchGroupSchema,
    type SearchCriterion,
    type SearchGroup,
    type LogicalOperator
} from './schemas';
import { ActionResult, ErrorCode } from '../../types/actions';
import { criterionLogicRegistry, textLogic, SearchValue } from './logic/handlers';
import { metadataState } from '../metadata';
import { supportedFormats } from '../systemStore';
import { createId } from '../../../lib/primitives/createId';
import {
    filterState,
    filterStateInternal,
    persist,
    type FilterSnapshot,
    type SortField,
    type SortOrder,
    type ViewLayout
} from './filterState';

const { setFilterState } = filterStateInternal;

let searchDebounceTimer: ReturnType<typeof setTimeout> | undefined;

export const filterActions = {
    pushHistory: () => {
        const snapshot: FilterSnapshot = {
            selectedTags: filterState.selectedTags,
            selectedFolderId: filterState.selectedFolderId,
            folderRecursiveView: filterState.folderRecursiveView,
            filterUntagged: filterState.filterUntagged,
            searchQuery: filterState.searchQuery,
            searchFuzzy: filterState.searchFuzzy,
            advancedSearch: filterState.advancedSearch,
            sortBy: filterState.sortBy,
            sortOrder: filterState.sortOrder
        };

        // Check if the current state is different from the last history item
        const current = filterState.history[filterState.historyIndex];
        const isSame = JSON.stringify(snapshot) === JSON.stringify(current);

        if (isSame) return;

        const newHistory = filterState.history.slice(0, filterState.historyIndex + 1);
        newHistory.push(snapshot);

        // Limit history
        const limit = filterState.historyLimit || 50;
        const finalHistory =
            newHistory.length > limit ? newHistory.slice(newHistory.length - limit) : newHistory;

        setFilterState({
            history: finalHistory,
            historyIndex: finalHistory.length - 1
        });
    },

    setHistoryLimit: (limit: number) => {
        setFilterState('historyLimit', limit);
        persist({ historyLimit: limit });
    },

    goBack: () => {
        if (filterState.historyIndex > 0) {
            const prevIndex = filterState.historyIndex - 1;
            const snapshot = filterState.history[prevIndex];
            batch(() => {
                setFilterState({
                    ...snapshot,
                    historyIndex: prevIndex
                });
            });
        }
    },

    goForward: () => {
        if (filterState.historyIndex < filterState.history.length - 1) {
            const nextIndex = filterState.historyIndex + 1;
            const snapshot = filterState.history[nextIndex];
            batch(() => {
                setFilterState({
                    ...snapshot,
                    historyIndex: nextIndex
                });
            });
        }
    },

    canGoBack: () => filterState.historyIndex > 0,
    canGoForward: () => filterState.historyIndex < filterState.history.length - 1,

    toggleTag: (tagId: number) => {
        const current = filterState.selectedTags;
        if (current.includes(tagId)) {
            setFilterState('selectedTags', tags => tags.filter(id => id !== tagId));
        } else {
            setFilterState({
                selectedTags: [...current, tagId],
                filterUntagged: false
            });
        }
        filterActions.pushHistory();
    },

    setUntagged: (isActive: boolean) => {
        setFilterState('filterUntagged', isActive);
        if (isActive) {
            setFilterState('selectedTags', []);
        }
        filterActions.pushHistory();
    },

    toggleUntagged: () => {
        filterActions.setUntagged(!filterState.filterUntagged);
    },

    setFolder: (folderId: number | null) => {
        setFilterState('selectedFolderId', folderId);
        filterActions.pushHistory();
    },

    setFolderRecursiveView: (isRecursive: boolean) => {
        setFilterState('folderRecursiveView', isRecursive);
        filterActions.pushHistory();
    },

    setSearch: (query: string) => {
        setFilterState('searchQuery', query);

        // Debounce history push for search
        if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
        searchDebounceTimer = setTimeout(() => {
            filterActions.pushHistory();
        }, APP_CONFIG.SEARCH_DEBOUNCE_MS);
    },

    setSearchFuzzy: (isFuzzy: boolean) => {
        setFilterState('searchFuzzy', isFuzzy);
        filterActions.pushHistory();
    },

    validateCriterion: (
        key: string,
        operator: string,
        value: unknown,
        value2?: unknown,
        unitMultiplier?: string
    ): Record<string, string> => {
        const field = SEARCH_FIELDS.find(f => f.value === key);
        if (!field) return { value: 'Invalid field' };

        const logic = criterionLogicRegistry[field.type] || textLogic;
        return logic.validate(
            value as SearchValue,
            value2 as SearchValue,
            operator,
            unitMultiplier
        );
    },

    formatCriterionDisplay: (criterion: Omit<SearchCriterion, 'id'>): string => {
        const field = SEARCH_FIELDS.find(f => f.value === criterion.key);
        if (!field) return String(criterion.value);

        const logic = criterionLogicRegistry[field.type] || textLogic;
        if (logic.formatDisplay) {
            const rawValue = criterion.value;
            const value1 = Array.isArray(rawValue) ? rawValue[0] : rawValue;
            const value2 = Array.isArray(rawValue) ? rawValue[1] : undefined;

            return logic.formatDisplay(
                value1,
                value2,
                criterion.operator,
                criterion.unitMultiplier,
                {
                    locations: metadataState.locations,
                    tags: metadataState.tags,
                    supportedFormats: supportedFormats()
                }
            );
        }

        return String(criterion.value);
    },

    processCriterion: (
        key: string,
        operator: string,
        value: unknown,
        value2?: unknown,
        unitMultiplier?: string
    ) => {
        const field = SEARCH_FIELDS.find(f => f.value === key);
        if (!field) return { finalValue: value, unitMultiplier };

        const logic = criterionLogicRegistry[field.type] || textLogic;
        return logic.process(value as SearchValue, value2 as SearchValue, operator, unitMultiplier);
    },

    setAdvancedSearch: (search: SearchGroup | null): ActionResult => {
        if (search) {
            const result = SearchGroupSchema.safeParse(search);
            if (!result.success) {
                return {
                    success: false,
                    error: {
                        code: ErrorCode.VALIDATION_ERROR,
                        message: 'Invalid search group structure',
                        details: result.error.format()
                    }
                };
            }
        }

        setFilterState('advancedSearch', search);
        filterActions.pushHistory();
        return { success: true, data: undefined };
    },

    addCriterion: (
        criterion: Omit<SearchCriterion, 'id' | 'displayValue'>
    ): ActionResult<string> => {
        const errors = filterActions.validateCriterion(
            criterion.key,
            criterion.operator,
            criterion.value,
            undefined, // value2 should be handled by logic.process before calling this
            criterion.unitMultiplier
        );

        if (Object.keys(errors).length > 0) {
            return {
                success: false,
                error: {
                    code: ErrorCode.VALIDATION_ERROR,
                    message: 'Invalid criterion values',
                    details: errors
                }
            };
        }

        const id = createId('criterion');
        const displayValue = filterActions.formatCriterionDisplay(criterion);
        const newCriterion: SearchCriterion = { ...criterion, id, displayValue };

        let currentGroup = filterState.advancedSearch;
        if (!currentGroup) {
            currentGroup = { id: createId('group'), logicalOperator: 'and', items: [] };
        }

        const newGroup: SearchGroup = {
            ...currentGroup,
            items: [...currentGroup.items, newCriterion]
        };

        const result = filterActions.setAdvancedSearch(newGroup);
        if (!result.success) return result;

        return { success: true, data: id };
    },

    removeCriterion: (id: string) => {
        const currentGroup = filterState.advancedSearch;
        if (!currentGroup) return;

        const newItems = currentGroup.items.filter((item: SearchCriterion | SearchGroup) => {
            if ('id' in item) return item.id !== id;
            return true;
        });

        filterActions.setAdvancedSearch({ ...currentGroup, items: newItems });
    },

    updateCriterion: (id: string, updates: Partial<SearchCriterion>): ActionResult => {
        const currentGroup = filterState.advancedSearch;
        if (!currentGroup)
            return {
                success: false,
                error: { code: ErrorCode.VALIDATION_ERROR, message: 'No search group active' }
            };

        const newItems = currentGroup.items.map((item: SearchCriterion | SearchGroup) => {
            if ('key' in item && item.id === id) {
                const merged = { ...item, ...updates } as SearchCriterion;
                const displayValue = filterActions.formatCriterionDisplay(merged);
                return { ...merged, displayValue };
            }
            return item;
        });

        return filterActions.setAdvancedSearch({ ...currentGroup, items: newItems });
    },

    setMatchMode: (mode: LogicalOperator) => {
        const currentGroup = filterState.advancedSearch;
        if (currentGroup) {
            filterActions.setAdvancedSearch({ ...currentGroup, logicalOperator: mode });
        } else {
            filterActions.setAdvancedSearch({
                id: createId('group'),
                logicalOperator: mode,
                items: []
            });
        }
    },

    setSortBy: (field: SortField) => {
        setFilterState('sortBy', field);
        persist({ sortBy: field });
        filterActions.pushHistory();
    },

    setSortOrder: (order: SortOrder) => {
        setFilterState('sortOrder', order);
        persist({ sortOrder: order });
        filterActions.pushHistory();
    },

    setLayout: (layout: ViewLayout) => {
        setFilterState('layout', layout);
        persist({ layout: layout });
        // Layout and zoom don't go to history as per user request
    },

    setThumbSize: (size: number) => {
        setFilterState('thumbSize', size);
        persist({ thumbSize: size });
    },

    clearAll: () => {
        setFilterState({
            selectedTags: [],
            selectedFolderId: null,
            filterUntagged: false,
            searchQuery: '',
            searchFuzzy: false,
            advancedSearch: null
        });
        filterActions.pushHistory();
    },

    hasActiveFilters: () => {
        return (
            filterState.selectedTags.length > 0 ||
            filterState.filterUntagged ||
            filterState.selectedFolderId !== null ||
            filterState.searchQuery !== '' ||
            filterState.advancedSearch !== null
        );
    }
};
