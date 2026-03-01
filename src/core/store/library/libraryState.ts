import { createStore } from 'solid-js/store';
import { type ImageItem } from '../../../types';

interface LibraryState {
    items: ImageItem[];
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
