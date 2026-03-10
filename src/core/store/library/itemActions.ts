import { invokeCommand as invoke } from '../../../lib/api';
import { tagService } from '../../../lib/tags';
import { libraryStateInternal } from './libraryState';

const { setLibraryState } = libraryStateInternal;

export const itemActions = {
    updateItemRating: async (id: string, rating: number) => {
        try {
            setLibraryState('items', i => i.id === id, 'rating', rating);
            await tagService.updateAssetRating(id, rating);
        } catch (err) {
            console.error(`Failed to update rating for ${id}:`, err);
        }
    },

    updateItemNotes: async (id: string, notes: string) => {
        try {
            setLibraryState('items', i => i.id === id, 'notes', notes);
            await tagService.updateAssetNotes(id, notes);
        } catch (err) {
            console.error(`Failed to update notes for ${id}:`, err);
        }
    },

    updateThumbnail: (id: string, path: string) => {
        setLibraryState('items', item => item.id === id, 'thumbnail_path', path);
    },

    setThumbnailPriority: async (ids: string[]) => {
        try {
            if (ids.length > 0) {
                await invoke('set_thumbnail_priority', { ids });
            }
        } catch (err) {
            console.error('Failed to set thumbnail priority:', err);
        }
    }
};
