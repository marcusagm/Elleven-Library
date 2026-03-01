import { filterState, filterActions } from '../store/filter';
import { libraryActions } from '../store/library';

/**
 * Hook providing access to filter state and actions for the application content.
 *
 * @returns {Object} Accessors and methods for filtering and sorting items.
 */
export const useFilters = () => {
    const withRefresh =
        <Args extends unknown[], R>(action: (...args: Args) => R) =>
        (...args: Args) => {
            const result = action(...args);
            libraryActions.refreshImages(true);
            return result;
        };

    return {
        get selectedTags() {
            return filterState.selectedTags;
        },
        get selectedFolderId() {
            return filterState.selectedFolderId;
        },
        get folderRecursiveView() {
            return filterState.folderRecursiveView;
        },
        get filterUntagged() {
            return filterState.filterUntagged;
        },
        get searchQuery() {
            return filterState.searchQuery;
        },
        get sortBy() {
            return filterState.sortBy;
        },
        get sortOrder() {
            return filterState.sortOrder;
        },
        get layout() {
            return filterState.layout;
        },
        get thumbSize() {
            return filterState.thumbSize;
        },
        get advancedSearch() {
            return filterState.advancedSearch;
        },

        get canGoBack() {
            return filterActions.canGoBack();
        },
        get canGoForward() {
            return filterActions.canGoForward();
        },

        toggleTag: withRefresh(filterActions.toggleTag),
        setUntagged: withRefresh(filterActions.setUntagged),
        toggleUntagged: withRefresh(filterActions.toggleUntagged),
        setFolder: withRefresh(filterActions.setFolder),
        setFolderRecursiveView: withRefresh(filterActions.setFolderRecursiveView),
        setSearch: withRefresh(filterActions.setSearch),
        setSortBy: withRefresh(filterActions.setSortBy),
        setSortOrder: withRefresh(filterActions.setSortOrder),
        setLayout: filterActions.setLayout,
        setThumbSize: filterActions.setThumbSize,
        setAdvancedSearch: withRefresh(filterActions.setAdvancedSearch),
        clearAll: withRefresh(filterActions.clearAll),

        goBack: withRefresh(filterActions.goBack),
        goForward: withRefresh(filterActions.goForward),

        hasActiveFilters: filterActions.hasActiveFilters
    };
};
