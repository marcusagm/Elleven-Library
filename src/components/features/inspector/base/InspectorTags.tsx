import { Component, createResource, createMemo, Show, For } from 'solid-js';
import { Tag as TagIcon, Plus } from 'lucide-solid';
import { AccordionItem, AccordionHeader, AccordionContent } from '../../../ui';
import { TagInput, type TagOption } from '../../../ui/TagInput';
import { useMetadata } from '../../../../core/hooks';
import { type Tag } from '../../../../lib/tags';
import './InspectorTags.css';

interface InspectorTagsProps {
    itemId?: string;
    itemIds?: string[];
}

export const InspectorTags: Component<InspectorTagsProps> = properties => {
    const metadata = useMetadata();

    const targetIds = createMemo(() => {
        if (properties.itemIds && properties.itemIds.length > 0) return properties.itemIds;
        if (properties.itemId !== undefined) return [properties.itemId];
        return [];
    });

    // Combine target IDs with the update version to trigger refetches
    // when tags are modified via Drag and Drop or batch operations.
    const resourceTrigger = createMemo(() => ({
        ids: targetIds(),
        version: metadata.tagUpdateVersion
    }));

    // Resource for tags of the selected item(s)
    const [itemTagsByAsset, { refetch }] = createResource(resourceTrigger, async ({ ids }) => {
        if (ids.length === 0) return new Map<string, Tag[]>();

        const results = await Promise.all(
            ids.map(async identifier => {
                const tags = await metadata.getAssetTags(identifier);
                return { identifier, tags };
            })
        );

        const map = new Map<string, Tag[]>();
        results.forEach(result => map.set(result.identifier, result.tags));
        return map;
    });

    const tagsAnalysis = createMemo(() => {
        const dataMap = itemTagsByAsset();
        const ids = targetIds();

        if (!dataMap || ids.length === 0) return { common: [], partial: [] };

        if (ids.length === 1) {
            return { common: dataMap.get(ids[0]) || [], partial: [] };
        }

        const allTagsList = ids.map(identifier => dataMap.get(identifier) || []);

        // Find common tags (intersection)
        const commonIds = allTagsList[0]
            .filter(tag => allTagsList.every(list => list.some(t => t.id === tag.id)))
            .map(tag => tag.id);

        const commonTags = allTagsList[0].filter(tag => commonIds.includes(tag.id));

        // Find partial tags (union minus common)
        const partialMap = new Map<number, Tag>();
        const commonSet = new Set(commonIds);

        allTagsList.forEach(list => {
            list.forEach(tag => {
                if (!commonSet.has(tag.id)) {
                    partialMap.set(tag.id, tag);
                }
            });
        });

        return {
            common: commonTags,
            partial: Array.from(partialMap.values())
        };
    });

    const tagOptions = createMemo(() =>
        (metadata.tags || []).map((tag: Tag) => ({
            id: tag.id,
            label: tag.name,
            color: tag.color || undefined
        }))
    );

    const selectedOptions = () =>
        tagsAnalysis().common.map((tag: Tag) => ({
            id: tag.id,
            label: tag.name,
            color: tag.color || undefined
        }));

    const handleChange = async (newOptions: TagOption[]) => {
        const ids = targetIds();
        if (ids.length === 0) return;

        const currentSelected = selectedOptions();
        const currentIdsSet = new Set(
            currentSelected.map((option: TagOption) => String(option.id))
        );
        const newIdsSet = new Set(newOptions.map((option: TagOption) => String(option.id)));

        // Calculate differences to perform atomic batch operations
        const tagsToAdd = newOptions
            .filter((option: TagOption) => !currentIdsSet.has(String(option.id)))
            .map((option: TagOption) => Number(option.id));

        if (tagsToAdd.length > 0) {
            await metadata.updateAssetsTags(ids, tagsToAdd, 'merge');
        }

        const tagsToRemove = currentSelected
            .filter((option: TagOption) => !newIdsSet.has(String(option.id)))
            .map((option: TagOption) => Number(option.id));

        if (tagsToRemove.length > 0) {
            await metadata.updateAssetsTags(ids, tagsToRemove, 'remove');
        }

        refetch();
    };

    const handleCreate = async (name: string) => {
        const ids = targetIds();
        if (ids.length === 0) return;

        const result = await metadata.createTag(name);
        if (result.success && result.data) {
            await metadata.updateAssetsTags(ids, [result.data], 'merge');
            refetch();
        }
    };

    const handlePartialAdd = async (tagId: number) => {
        const ids = targetIds();
        if (ids.length === 0) return;

        await metadata.updateAssetsTags(ids, [tagId], 'merge');
        refetch();
    };

    return (
        <AccordionItem value="tags">
            <AccordionHeader title="Tags" icon={<TagIcon size={14} />} />
            <AccordionContent>
                <div class="inspector-tags-wrapper">
                    <TagInput
                        value={selectedOptions()}
                        suggestions={tagOptions()}
                        onChange={handleChange}
                        onCreate={handleCreate}
                        placeholder="Add tags..."
                    />

                    <Show when={tagsAnalysis().partial.length > 0}>
                        <div class="partial-tags-section">
                            <h4 class="partial-tags-title">Parcialmente em alguns arquivos</h4>
                            <div class="partial-tags-list">
                                <For each={tagsAnalysis().partial}>
                                    {tag => (
                                        <button
                                            class="partial-tag-item"
                                            onClick={() => handlePartialAdd(tag.id)}
                                            style={{
                                                'border-left': tag.color
                                                    ? `3px solid ${tag.color}`
                                                    : undefined
                                            }}
                                            title="Clique para adicionar a todos os selecionados"
                                        >
                                            <span>{tag.name}</span>
                                            <Plus size={10} class="plus-icon" />
                                        </button>
                                    )}
                                </For>
                            </div>
                        </div>
                    </Show>
                </div>
            </AccordionContent>
        </AccordionItem>
    );
};
