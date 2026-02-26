import { Button } from '../../ui';
import { Component, Show, createSignal, Switch, Match } from 'solid-js';
import { useSystem } from '../../../core/hooks';
import { Loader } from '../../ui/Loader';
import { CircleCheck, PaintRoller, Settings } from 'lucide-solid';

export const StatusSystem: Component = () => {
    const system = useSystem();
    // Thumbnail queue represents real backend state from systemStore
    const thumbnailQueue = () => system.thumbnailProgress();
    const [isPopoverOpen, setIsPopoverOpen] = createSignal(false);

    return (
        <div class="statusbar-section statusbar-system">
            {/* Background Processes Indicator */}
            <div class="system-indicator">
                <Switch
                    fallback={
                        <Button
                            variant="ghost"
                            size="icon-sm"
                            class="status-btn success-text"
                            title="All Systems Operational"
                            onClick={() => setIsPopoverOpen(!isPopoverOpen())}
                        >
                            <CircleCheck size={12} />
                        </Button>
                    }
                >
                    <Match when={system.progress() || thumbnailQueue()}>
                        <Button
                            variant="ghost"
                            size="icon-sm"
                            class="status-btn"
                            onClick={() => setIsPopoverOpen(!isPopoverOpen())}
                        >
                            <Loader size="sm" />
                        </Button>
                    </Match>
                </Switch>
            </div>

            {/* Popover Logic (Simplistic Inline for now, ideal to be a real Popover) */}
            <Show when={isPopoverOpen()}>
                <div class="system-popover ui-popover-content">
                    <div class="popover-header">System Activity</div>
                    <div class="popover-content">
                        <Show when={!system.progress() && !thumbnailQueue()}>
                            <div class="empty-state">No background tasks running.</div>
                        </Show>

                        <Show when={system.progress()}>
                            <div class="task-row">
                                <Loader size="sm" />
                                <div>
                                    <div class="task-name">Indexing Library</div>
                                    <div class="task-status">
                                        {system.progress()?.processed} / {system.progress()?.total}
                                    </div>
                                </div>
                            </div>
                        </Show>

                        <Show when={thumbnailQueue()}>
                            <div class="task-row">
                                <Loader size="sm" />
                                <div>
                                    <div class="task-name">Generating Thumbnails</div>
                                    <div class="task-status">
                                        {thumbnailQueue()?.processed} / {thumbnailQueue()?.total}
                                    </div>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </Show>

            <div class="statusbar-divider" />

            <Button
                variant="ghost"
                size="icon-sm"
                title="Settings (Cmd+,)"
                onClick={() => system.openSettings(true)}
            >
                <Settings size={12} />
            </Button>

            <Button
                variant="ghost"
                size="icon-sm"
                title="Design System"
                onClick={() => system.openDesignSystem(true)}
            >
                <PaintRoller size={12} />
            </Button>
        </div>
    );
};
