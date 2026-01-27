import { Component, createSignal, For } from "solid-js";
import { Star } from "lucide-solid";
import "./star-rating.css";

interface StarRatingProps {
  rating: number; // 0 to 5
  onChange?: (rating: number) => void;
  readOnly?: boolean;
}

export const StarRating: Component<StarRatingProps> = (props) => {
  const [hoverRating, setHoverRating] = createSignal<number | null>(null);

  const handleClick = (index: number) => {
    if (props.readOnly || !props.onChange) return;
    // Toggle: if clicking the current rating, reset to 0
    const newValue = props.rating === index ? 0 : index;
    props.onChange(newValue);
  };

  return (
    <div 
        class="star-rating" 
        role="radiogroup" 
        aria-label="Rating"
        onMouseLeave={() => setHoverRating(null)}
    >
      <For each={[1, 2, 3, 4, 5]}>
        {(index) => (
          <button
            type="button"
            class={`star-btn ${
                (hoverRating() !== null ? index <= hoverRating()! : index <= props.rating) 
                ? "active" : ""
            } ${
                (hoverRating() !== null && index <= hoverRating()!) ? "hover-active" : ""
            }`}
            onClick={() => handleClick(index)}
            onMouseEnter={() => !props.readOnly && setHoverRating(index)}
            aria-label={`${index} stars`}
            aria-checked={props.rating === index}
            role="radio"
            disabled={props.readOnly}
          >
            <Star 
                size={16} 
                fill={(hoverRating() !== null ? index <= hoverRating()! : index <= props.rating) ? "currentColor" : "none"} 
            />
          </button>
        )}
      </For>
    </div>
  );
};
