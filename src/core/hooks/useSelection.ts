import { selectionState, selectionActions } from "../store/selectionStore";

export const useSelection = () => {
  return {
    // State
    get selectedIds() { return selectionState.selectedIds; },
    
    // Actions
    toggle: selectionActions.toggle,
    select: selectionActions.select,
    clear: selectionActions.clear,
    isSelected: selectionActions.isSelected
  };
};
