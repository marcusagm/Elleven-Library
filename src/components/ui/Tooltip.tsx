import { Component, JSX, createSignal } from "solid-js";
import { Portal } from "solid-js/web";
import "./tooltip.css";

interface TooltipProps {
  content: string;
  children: JSX.Element;
  position?: "top" | "bottom" | "left" | "right";
}

export const Tooltip: Component<TooltipProps> = (props) => {
  const [isVisible, setIsVisible] = createSignal(false);
  const [coords, setCoords] = createSignal({ x: 0, y: 0 });
  let triggerRef: HTMLDivElement | undefined;

  const showTooltip = () => {
    if (triggerRef) {
      const rect = triggerRef.getBoundingClientRect();
      // Simple positioning logic (can be improved with floating-ui)
      // Default to right for now as requested for sidebar mostly
      const pos = props.position || "right";
      
      let x = 0, y = 0;
      if (pos === "right") {
         x = rect.right + 8;
         y = rect.top + (rect.height / 2) - 10; // Approx center
      } else if (pos === "bottom") {
         x = rect.left;
         y = rect.bottom + 8;
      }
      
      setCoords({ x, y });
      setIsVisible(true);
    }
  };

  const hideTooltip = () => setIsVisible(false);

  return (
    <>
      <div 
        ref={triggerRef} 
        onMouseEnter={showTooltip} 
        onMouseLeave={hideTooltip}
        style={{ display: "inline-block" }}
      >
        {props.children}
      </div>
      {isVisible() && (
        <Portal>
          <div 
            class="ui-tooltip"
            style={{ 
                top: `${coords().y}px`, 
                left: `${coords().x}px` 
            }}
          >
            {props.content}
          </div>
        </Portal>
      )}
    </>
  );
};
