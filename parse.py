import re
from collections import defaultdict

reg_err = re.compile("(.*): (\w+)Error")
reg_ok = re.compile("(.*): OK")

f = open('test_results.txt')
counts = defaultdict(lambda: 0)
for line in f:
    #print(line)
    result = reg_err.search(line)
    if result is not None:
        counts[result.group(2)] += 1
    result = reg_ok.search(line)
    if result is not None:
        counts["OK"] += 1

print(counts)
