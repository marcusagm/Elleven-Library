import { createStore } from "solid-js/store";

interface SelectionState {
  selectedIds: number[];
}

const [selectionState, setSelectionState] = createStore<SelectionState>({
  selectedIds: []
});

export const selectionActions = {
  toggle: (id: number, multi: boolean) => {
    if (multi) {
      const current = selectionState.selectedIds;
      if (current.includes(id)) {
        setSelectionState("selectedIds", current.filter(i => i !== id));
      } else {
        setSelectionState("selectedIds", [...current, id]);
      }
    } else {
      setSelectionState("selectedIds", [id]);
    }
  },

  select: (ids: number[]) => {
    setSelectionState("selectedIds", ids);
  },

  clear: () => {
    setSelectionState("selectedIds", []);
  },

  isSelected: (id: number) => {
    return selectionState.selectedIds.includes(id);
  }
};

export { selectionState };
