import { Component, createSignal, Show } from "solid-js";
import { Portal } from "solid-js/web";
import "./count-badge.css";

interface CountBadgeProps {
  count: number;
  variant?: "primary" | "secondary" | "outline";
}

/**
 * A specialized badge for displaying numbers with auto-abbreviation (e.g., 1.2k)
 * and a custom Tooltip for the exact value.
 */
export const CountBadge: Component<CountBadgeProps> = (props) => {
  const [showTooltip, setShowTooltip] = createSignal(false);
  const [coords, setCoords] = createSignal({ x: 0, y: 0 });
  let badgeRef: HTMLDivElement | undefined;

  const formatNumber = (num: number): string => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1).replace(/\.0$/, "") + "M";
    }
    if (num >= 1000) {
      return (num / 1000).toFixed(1).replace(/\.0$/, "") + "k";
    }
    return num.toString();
  };

  const handleMouseEnter = () => {
    if (badgeRef) {
      const rect = badgeRef.getBoundingClientRect();
      setCoords({
        x: rect.left + rect.width / 2,
        y: rect.top - 8,
      });
      setShowTooltip(true);
    }
  };

  return (
    <div
      ref={badgeRef}
      class={`count-badge ${props.variant || "secondary"}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={() => setShowTooltip(false)}
    >
      {formatNumber(props.count)}

      <Show when={showTooltip()}>
        <Portal>
          <div
            class="count-badge-tooltip"
            style={{
              left: `${coords().x}px`,
              top: `${coords().y}px`,
            }}
          >
            {props.count.toLocaleString()}
            <div class="tooltip-arrow" />
          </div>
        </Portal>
      </Show>
    </div>
  );
};
