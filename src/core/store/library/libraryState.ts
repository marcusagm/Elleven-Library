import { createStore } from 'solid-js/store';
import { type AssetItem } from '../../../types';

interface LibraryState {
    items: AssetItem[];
    isFetching: boolean;
    isRefreshing: boolean;
    totalItems: number; // useful for knowing if we reached end
}

export const [libraryState, setLibraryState] = createStore<LibraryState>({
    items: [],
    isFetching: false,
    isRefreshing: false,
    totalItems: 0
});

export const libraryStateInternal = {
    setLibraryState
};
