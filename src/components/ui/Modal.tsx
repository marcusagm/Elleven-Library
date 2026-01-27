import { Component, JSX, Show, createEffect, onCleanup, createSignal, createMemo } from "solid-js";
import { Portal } from "solid-js/web";
import { X } from "lucide-solid";
import "./modal.css";
import { Button } from "./Button";
import { Input } from "./Input";

type ModalVariant = "sm" | "md" | "lg" | "full";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  variant?: ModalVariant;
  closeOnOverlayClick?: boolean;
  showCloseButton?: boolean;
  children: JSX.Element;
  footer?: JSX.Element;
}

export const Modal: Component<ModalProps> = (props) => {
  const [containerRef, setContainerRef] = createSignal<HTMLDivElement>();

  // Use default props
  const config = createMemo(() => ({
    variant: props.variant || "md",
    closeOnOverlayClick: props.closeOnOverlayClick ?? true,
    showCloseButton: props.showCloseButton ?? true,
  }));

  // Handle Escape key
  createEffect(() => {
    if (!props.isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        props.onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    document.body.classList.add("modal-open");

    onCleanup(() => {
      window.removeEventListener("keydown", handleKeyDown);
      document.body.classList.remove("modal-open");
    });
  });

  // Simple Focus Trap
  createEffect(() => {
    const el = containerRef();
    if (props.isOpen && el) {
      const focusableElements = el.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      if (focusableElements.length > 0) {
        (focusableElements[0] as HTMLElement).focus();
      }

      const handleTab = (e: KeyboardEvent) => {
        if (e.key !== "Tab") return;
        
        const first = focusableElements[0] as HTMLElement;
        const last = focusableElements[focusableElements.length - 1] as HTMLElement;

        if (e.shiftKey) {
          if (document.activeElement === first) {
            last.focus();
            e.preventDefault();
          }
        } else {
          if (document.activeElement === last) {
            first.focus();
            e.preventDefault();
          }
        }
      };

      el.addEventListener("keydown", handleTab);
      onCleanup(() => el.removeEventListener("keydown", handleTab));
    }
  });

  return (
    <Show when={props.isOpen}>
      <Portal>
        <div 
          class="modal-overlay" 
          onClick={() => config().closeOnOverlayClick && props.onClose()}
          aria-modal="true"
          role="dialog"
        >
          <div 
            ref={setContainerRef}
            class={`modal-container variant-${config().variant}`}
            onClick={(e) => e.stopPropagation()}
            tabindex="-1"
          >
            <Show when={props.title || config().showCloseButton}>
              <div class="modal-header">
                <Show when={props.title}>
                  <h3 class="modal-title">{props.title}</h3>
                </Show>
                <Show when={config().showCloseButton}>
                  <button 
                    class="modal-close-btn" 
                    onClick={props.onClose}
                    aria-label="Close"
                  >
                    <X size={18} />
                  </button>
                </Show>
              </div>
            </Show>

            <div class="modal-body">
              {props.children}
            </div>

            <Show when={props.footer}>
              <div class="modal-footer">
                {props.footer}
              </div>
            </Show>
          </div>
        </div>
      </Portal>
    </Show>
  );
};

// Internal sub-components for more flexibility if needed later
// (Currently we just use the props structure but exports allow for manual building)
export const ModalHeader: Component<{ children: JSX.Element }> = (p) => <div class="modal-header">{p.children}</div>;
export const ModalBody: Component<{ children: JSX.Element }> = (p) => <div class="modal-body">{p.children}</div>;
export const ModalFooter: Component<{ children: JSX.Element }> = (p) => <div class="modal-footer">{p.children}</div>;

// --- Specialized Modals ---

export const PromptModal: Component<{
    isOpen: boolean,
    onClose: () => void,
    onConfirm: (val: string) => void,
    title: string,
    initialValue?: string,
    placeholder?: string
}> = (props) => {
    let inputRef: HTMLInputElement | undefined;

    const handleSubmit = (e: Event) => {
        e.preventDefault();
        if (inputRef && inputRef.value) {
            props.onConfirm(inputRef.value);
            props.onClose();
        }
    }

    return (
        <Modal 
            isOpen={props.isOpen} 
            onClose={props.onClose} 
            title={props.title}
            variant="sm"
        >
            <form onSubmit={handleSubmit}>
                <Input 
                    ref={inputRef}
                    value={props.initialValue || ""} 
                    placeholder={props.placeholder}
                    autofocus
                />
                <div style={{ display: "flex", "justify-content": "flex-end", gap: "8px", "margin-top": "24px" }}>
                    <Button type="button" variant="ghost" onClick={props.onClose}>Cancel</Button>
                    <Button type="submit" variant="primary">Confirm</Button>
                </div>
            </form>
        </Modal>
    );
};

export const ConfirmModal: Component<{
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    kind?: "danger" | "warning" | "info";
    variant?: ModalVariant;
    children?: JSX.Element;
}> = (props) => {
    return (
        <Modal
            isOpen={props.isOpen}
            onClose={props.onClose}
            title={props.title}
            variant={props.variant || "sm"}
            footer={
              <>
                <Button variant="ghost" onClick={props.onClose}>
                    {props.cancelText || "Cancel"}
                </Button>
                <Button 
                    variant={props.kind === "danger" ? "destructive" : "primary"} 
                    onClick={() => {
                        props.onConfirm();
                        props.onClose();
                    }}
                >
                    {props.confirmText || "Confirm"}
                </Button>
              </>
            }
        >
            <Show when={props.children} fallback={<div>{props.message}</div>}>
                {props.children}
            </Show>
        </Modal>
    );
};
