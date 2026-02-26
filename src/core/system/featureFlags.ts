/**
 * Simple Feature Flags utility to manage rollout of new features.
 * This allows us to merge code without necessarily activating it in production.
 */

type FeatureFlag = 'new-actions-search' | 'experimental-dnd' | 'zod-validation-strict';

const flags: Record<FeatureFlag, boolean> = {
    'new-actions-search': false,
    'experimental-dnd': false,
    'zod-validation-strict': true
};

/**
 * Checks if a specific feature flag is enabled.
 *
 * @param {FeatureFlag} flag - The name of the feature flag.
 * @returns {boolean} True if the flag is enabled.
 */
export const isFeatureEnabled = (flag: FeatureFlag): boolean => {
    // In the future, this could be connected to environment variables or a remote config.
    return flags[flag] || false;
};
