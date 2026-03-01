import { metadataState } from './metadataState';
import { tagActions, initTagRefs } from './tagActions';
import { searchActions } from './searchActions';
import { locationActions, initLocationRefs } from './locationActions';

// Initialize circular dependencies avoiding proxies.
initTagRefs(locationActions);
initLocationRefs(tagActions, searchActions);

// Export the aggregated metadata actions under a single object
export const metadataActions = {
    ...tagActions,
    ...searchActions,
    ...locationActions
};

// Re-export state
export { metadataState };
export * from './metadataState';
