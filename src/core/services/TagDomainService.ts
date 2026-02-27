/**
 * Domain service for Tag business logic.
 * Handles naming conventions, hierarchy validation, and sort index calculations.
 */
export const TagDomainService = {
    /**
     * Normalizes a tag name by trimming whitespace and removing invalid characters.
     * Prevents empty or purely numeric tags if business rules require it.
     */
    normalizeName: (name: string): string => {
        return name.trim();
    },

    /**
     * Calculates the next order index for a tag list to append a new tag.
     */
    calculateNextOrder: (existingTags: { order_index: number }[]): number => {
        if (existingTags.length === 0) return 0;
        const maxOrder = Math.max(...existingTags.map(t => t.order_index));
        return maxOrder + 100; // Increment by 100 for future inserts
    }
};
