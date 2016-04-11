import re
from collections import defaultdict

reg = re.compile("(.*): (\w+)Error")
f = open('test_results.txt')
counts = defaultdict(lambda: 0)
for line in f:
    #print(line)
    result = reg.search(line)
    if result is not None:
        #print(result.group(1), result.group(2))
        counts[result.group(2)] += 1

print(counts)
