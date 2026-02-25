import re
from collections import defaultdict
import os

with open('importances.txt', 'r') as f: raw_text = f.read()

# Parse lines like: p1_all_3x3s_with_x511: 132.000000
pattern = re.compile(r'p[12]_all_3x3s_with_(x|y)(\d+):\s*([\d.]+)')

x_patterns = defaultdict(float)   # number → max importance
y_patterns = defaultdict(float)

for line in raw_text.strip().splitlines():
    line = line.strip()
    if not line:
        continue
    match = pattern.search(line)
    if match:
        axis, num_str, imp_str = match.groups()
        num = int(num_str)
        imp = float(imp_str)
        if axis == 'x':
            x_patterns[num] = max(x_patterns[num], imp)
        else:
            y_patterns[num] = max(y_patterns[num], imp)

# Sort descending by importance, then by first appearance order if ties (stable)
sorted_x = sorted(x_patterns.items(), key=lambda item: (-item[1], item[0]))
sorted_y = sorted(y_patterns.items(), key=lambda item: (-item[1], item[0]))

# Take top 100 unique (already unique because of dict)
top_x = [num for num, _ in sorted_x[:100]]
top_y = [num for num, _ in sorted_y[:100]]

# Print in Rust array format
print("pub const top_100_3x3s_with_x: [usize; 100] = [")
for i in range(0, len(top_x), 10):
    chunk = top_x[i:i+10]
    print("    " + ", ".join(map(str, chunk)) + ",")
print("];")
print()

print("pub const top_100_3x3s_with_y: [usize; 100] = [")
for i in range(0, len(top_y), 10):
    chunk = top_y[i:i+10]
    print("    " + ", ".join(map(str, chunk)) + ",")
print("];")