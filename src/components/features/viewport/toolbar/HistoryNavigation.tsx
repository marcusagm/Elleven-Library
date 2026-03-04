import { Component, JSX } from 'solid-js';
import { Button, ButtonGroup, Tooltip } from '../../../ui';
import { ArrowLeft, ArrowRight } from 'lucide-solid';
import { useFilters } from '../../../../core/hooks';

/**
 * Component responsible for history navigation (Back/Forward).
 * Uses the filters store to manage history state.
 *
 * @returns {JSX.Element} The history navigation buttons.
 */
export const HistoryNavigation: Component = (): JSX.Element => {
    /**
     * Filters store
     */
    const filters = useFilters();

    return (
        <div class="toolbar-group">
            <ButtonGroup attached>
                <Tooltip content="Back" placement="bottom">
                    <Button
                        variant="secondary"
                        size="icon"
                        onClick={() => filters.goBack()}
                        disabled={!filters.canGoBack}
                    >
                        <ArrowLeft size={18} />
                    </Button>
                </Tooltip>
                <Tooltip content="Forward" placement="bottom">
                    <Button
                        variant="secondary"
                        size="icon"
                        onClick={() => filters.goForward()}
                        disabled={!filters.canGoForward}
                    >
                        <ArrowRight size={18} />
                    </Button>
                </Tooltip>
            </ButtonGroup>
        </div>
    );
};
