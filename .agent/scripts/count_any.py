import os
import re
import sys

def count_any_in_directory(directory):
    any_pattern = re.compile(r'(?<!\w)any(?!\w)')
    total_any_count = 0
    file_stats = []

    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith(('.ts', '.tsx')):
                file_path = os.path.join(root, file)
                try:
                    with open(file_path, 'r', encoding='utf-8') as f:
                        content = f.read()

                        # Simple count of 'any' as a type (avoiding comments for simplicity in this baseline)
                        # A better check would be needed for production, but for a baseline this works.
                        count = len(any_pattern.findall(content))

                        if count > 0:
                            total_any_count += count
                            file_stats.append((file_path, count))
                except Exception as e:
                    print(f"Error reading {file_path}: {e}")

    return total_any_count, file_stats

if __name__ == "__main__":
    target_dir = sys.argv[1] if len(sys.argv) > 1 else 'src/core'
    absolute_path = os.path.abspath(target_dir)

    print(f"--- Running 'any' Count Baseline for: {target_dir} ---")
    total, stats = count_any_in_directory(absolute_path)

    print(f"\nTotal 'any' usage: {total}")
    print("\nBreakdown by file:")
    for file_path, count in sorted(stats, key=lambda x: x[1], reverse=True):
        relative_path = os.path.relpath(file_path, os.getcwd())
        print(f"- {relative_path}: {count}")

    print("\n--- End of Report ---")
