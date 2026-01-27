import { loading, progress, rootPath, systemActions } from "../store/systemStore";

export const useSystem = () => {
  return {
    // State
    loading,
    progress,
    rootPath,
    
    // Actions
    initialize: systemActions.initialize,
    setRootLocation: systemActions.setRootLocation,
    updateProgress: systemActions.updateProgress,
    clearProgress: systemActions.clearProgress
  };
};
