import { createSignal, createResource, createMemo, Resource, Setter, Accessor } from 'solid-js';
import { duplicatesApi } from '../../../../lib/duplicates';
import { DuplicateGroup } from '../types';

export interface UseDuplicateGroupsReturn {
    /** All fetched groups (open + ignored when enabled). */
    groups: Resource<DuplicateGroup[]>;
    /** The currently visible groups after filtering. */
    visibleGroups: Accessor<DuplicateGroup[]>;
    selectedGroupId: () => string | null;
    setSelectedGroupId: Setter<string | null>;
    selectGroup: (groupId: string) => Promise<void>;
    resolveGroup: (groupId: string, action: string, keptAssetIds?: string[]) => Promise<void>;
    startScan: () => Promise<void>;
    showIgnored: Accessor<boolean>;
    setShowIgnored: Setter<boolean>;
}

/**
 * Hook to manage the state and actions of duplicate groups.
 *
 * @returns {UseDuplicateGroupsReturn} The state and actions for duplicate groups.
 */
export function useDuplicateGroups(): UseDuplicateGroupsReturn {
    const [selectedGroupId, setSelectedGroupId] = createSignal<string | null>(null);
    const [showIgnored, setShowIgnored] = createSignal(false);

    // Fetch all duplicate groups (open + ignored) so that the toggle is instant.
    const [groups, { mutate }] = createResource(async () => {
        try {
            const openGroups = await duplicatesApi.getDuplicateGroups('open');
            const ignoredGroups = await duplicatesApi.getDuplicateGroups('ignored');
            return [...openGroups, ...ignoredGroups];
        } catch (error) {
            console.error('Failed to load duplicate groups:', error);
            return [];
        }
    });

    /**
     * Derived list of groups visible in the sidebar after applying the ignored filter.
     */
    const visibleGroups = createMemo(() => {
        const allGroups = groups() || [];
        if (showIgnored()) return allGroups;
        return allGroups.filter(group => group.status !== 'ignored');
    });

    /**
     * Selects a group and fetches its candidates if they are not already loaded.
     *
     * @param {string} groupId - The ID of the group to select.
     */
    const selectGroup = async (groupId: string) => {
        setSelectedGroupId(groupId);
        const currentGroups = groups();
        if (!currentGroups) return;

        const groupIndex = currentGroups.findIndex(group => group.id === groupId);
        if (groupIndex !== -1 && !currentGroups[groupIndex].candidatesLoaded) {
            try {
                const candidates = await duplicatesApi.getDuplicateCandidates(groupId);
                const updatedGroups = [...currentGroups];
                updatedGroups[groupIndex] = {
                    ...updatedGroups[groupIndex],
                    candidates,
                    candidatesLoaded: true,
                    candidateCount: candidates.length
                };
                mutate(updatedGroups);
            } catch (error) {
                console.error(`Failed to load candidates for group ${groupId}:`, error);
                // Mark as loaded even on error so we don't retry endlessly
                const updatedGroups = [...currentGroups];
                updatedGroups[groupIndex] = {
                    ...updatedGroups[groupIndex],
                    candidatesLoaded: true
                };
                mutate(updatedGroups);
            }
        }
    };

    /**
     * Resolves a duplicate group using the provided action and keeps track of the state.
     *
     * @param {string} groupId - The ID of the group to resolve.
     * @param {string} action - The action chosen by the user.
     * @param {string[]} [keptAssetIds] - The list of asset IDs to keep.
     */
    const resolveGroup = async (groupId: string, action: string, keptAssetIds?: string[]) => {
        try {
            await duplicatesApi.resolveDuplicateGroup(groupId, action, keptAssetIds);

            const currentGroups = groups();
            if (currentGroups) {
                if (action === 'ignore_group') {
                    // Move the group to 'ignored' status locally
                    mutate(
                        currentGroups.map(group =>
                            group.id === groupId ? { ...group, status: 'ignored' as const } : group
                        )
                    );
                } else {
                    // Remove resolved groups from local state
                    mutate(currentGroups.filter(group => group.id !== groupId));
                }
            }

            if (selectedGroupId() === groupId) {
                setSelectedGroupId(null);
            }
        } catch (error) {
            console.error('Failed to resolve duplicate group:', error);
            throw error;
        }
    };

    /**
     * Triggers a manual deep scan and refreshes the groups list.
     */
    const startScan = async () => {
        try {
            await duplicatesApi.startDuplicateScan();
            // Refetch all groups
            const openGroups = await duplicatesApi.getDuplicateGroups('open');
            const ignoredGroups = await duplicatesApi.getDuplicateGroups('ignored');
            mutate([...openGroups, ...ignoredGroups]);
        } catch (error) {
            console.error('Failed to start scan:', error);
            throw error;
        }
    };

    return {
        groups,
        visibleGroups,
        selectedGroupId,
        setSelectedGroupId,
        selectGroup,
        resolveGroup,
        startScan,
        showIgnored,
        setShowIgnored
    };
}
