import { Component, For, Show, createSignal } from 'solid-js';
import { Button, Badge } from '../../ui';
import { CheckCircle2 } from 'lucide-solid';
import { Thumbnail } from '../viewport/assets/Thumbnail';
import { DuplicateGroup } from './types';
import './duplicate-comparison-panel.css';
export interface DuplicateComparisonPanelProperties {
    /** The duplicate group to display and resolve */
    group: DuplicateGroup;
    /** Callback triggered when the group has been resolved */
    onResolve: (groupId: string, action: string, keptAssetIds?: string[]) => Promise<void>;
}

export const DuplicateComparisonPanel: Component<DuplicateComparisonPanelProperties> = props => {
    const [selectedCandidates, setSelectedCandidates] = createSignal<Set<string>>(new Set());
    const [processing, setProcessing] = createSignal(false);

    const toggleCandidate = (id: string) => {
        const newSet = new Set(selectedCandidates());
        if (newSet.has(id)) {
            newSet.delete(id);
        } else {
            newSet.add(id);
        }
        setSelectedCandidates(newSet);
    };

    const handleIgnoreGroup = async () => {
        setProcessing(true);
        try {
            await props.onResolve(props.group.id, 'ignore_group');
        } catch (error) {
            console.error('Failed to ignore group:', error);
        } finally {
            setProcessing(false);
        }
    };

    const handleKeepSelected = async () => {
        if (selectedCandidates().size === 0) return;

        setProcessing(true);
        try {
            const keptIds = Array.from(selectedCandidates());
            await props.onResolve(props.group.id, 'custom_selection', keptIds);
        } catch (error) {
            console.error('Failed to keep selected candidates:', error);
        } finally {
            setProcessing(false);
        }
    };

    const handleKeepOnlyThis = async (candidateId: string) => {
        setProcessing(true);
        try {
            await props.onResolve(props.group.id, 'custom_selection', [candidateId]);
        } catch (error) {
            console.error('Failed to keep candidate:', error);
        } finally {
            setProcessing(false);
        }
    };

    return (
        <div class="comparison-panel">
            <div class="comparison-header">
                <div class="comparison-title-container">
                    <h2>Group Details</h2>
                    <div class="comparison-meta">
                        <Badge>{props.group.type}</Badge>
                        <span class="comparison-confidence">
                            Confidence: {(props.group.confidence * 100).toFixed(0)}%
                        </span>
                    </div>
                </div>
                <div class="comparison-actions">
                    <Button variant="secondary" onClick={handleIgnoreGroup} disabled={processing()}>
                        Ignore Group
                    </Button>
                    <Button
                        disabled={selectedCandidates().size === 0 || processing()}
                        onClick={handleKeepSelected}
                    >
                        Keep Selected
                    </Button>
                </div>
            </div>

            <div class="comparison-grid">
                <For each={props.group.candidates}>
                    {candidate => {
                        const isSelected = () => selectedCandidates().has(candidate.id);
                        return (
                            <div
                                tabIndex={0}
                                class={`candidate-card ${isSelected() ? 'is-selected' : ''}`}
                                onClick={() => toggleCandidate(candidate.id)}
                                onKeyDown={e => {
                                    if (e.key === 'Enter' || e.key === ' ') {
                                        e.preventDefault();
                                        toggleCandidate(candidate.id);
                                    }
                                }}
                            >
                                <div class="candidate-card-body">
                                    <div class="candidate-header">
                                        <h3 class="candidate-name">{candidate.name}</h3>
                                        <div class="candidate-badges">
                                            <Show when={isSelected()}>
                                                <div class="candidate-selected-icon">
                                                    <CheckCircle2
                                                        size={20}
                                                        fill="currentColor"
                                                        color="var(--bg-secondary)"
                                                    />
                                                </div>
                                            </Show>
                                            <Show when={candidate.isFavorite}>
                                                <Badge variant="secondary">Favorite</Badge>
                                            </Show>
                                            <Badge variant="secondary">
                                                {(candidate.score * 100).toFixed(0)}%
                                            </Badge>
                                        </div>
                                    </div>

                                    <div class="candidate-preview">
                                        <Thumbnail
                                            id={candidate.id}
                                            src={candidate.path}
                                            thumbnail={candidate.thumbnailUrl || null}
                                            alt={candidate.name}
                                            mediaType={candidate.mediaType}
                                            state={candidate.state}
                                        />
                                    </div>

                                    <div class="candidate-details">
                                        <div class="candidate-detail-item full-width">
                                            <span class="candidate-detail-label">Path</span>
                                            <span class="candidate-detail-value">
                                                {candidate.path}
                                            </span>
                                        </div>
                                        <div class="candidate-detail-item">
                                            <span class="candidate-detail-label">Format</span>
                                            <span class="candidate-detail-value">
                                                {candidate.format}
                                            </span>
                                        </div>
                                        <div class="candidate-detail-item">
                                            <span class="candidate-detail-label">Size</span>
                                            <span class="candidate-detail-value">
                                                {candidate.size}
                                            </span>
                                        </div>
                                        <div class="candidate-detail-item">
                                            <span class="candidate-detail-label">Dimensions</span>
                                            <span class="candidate-detail-value">
                                                {candidate.dimensions}
                                            </span>
                                        </div>
                                        <div class="candidate-detail-item">
                                            <span class="candidate-detail-label">Created</span>
                                            <span class="candidate-detail-value">
                                                {new Date(candidate.createdAt).toLocaleString()}
                                            </span>
                                        </div>
                                        <div class="candidate-detail-item">
                                            <span class="candidate-detail-label">Modified</span>
                                            <span class="candidate-detail-value">
                                                {new Date(candidate.updatedAt).toLocaleString()}
                                            </span>
                                        </div>
                                        <Show when={candidate.tags && candidate.tags.length > 0}>
                                            <div class="candidate-detail-item full-width">
                                                <span class="candidate-detail-label">Tags</span>
                                                <span class="candidate-detail-value">
                                                    {candidate.tags.join(', ')}
                                                </span>
                                            </div>
                                        </Show>
                                        <Show when={candidate.notes}>
                                            <div class="candidate-detail-item full-width">
                                                <span class="candidate-detail-label">Notes</span>
                                                <span class="candidate-detail-value">
                                                    {candidate.notes}
                                                </span>
                                            </div>
                                        </Show>
                                    </div>
                                </div>

                                <div class="candidate-actions">
                                    <Button
                                        class="candidate-button"
                                        disabled={processing()}
                                        onClick={() => handleKeepOnlyThis(candidate.id)}
                                    >
                                        Keep Only This
                                    </Button>
                                </div>
                            </div>
                        );
                    }}
                </For>
            </div>
        </div>
    );
};
