#!/usr/bin/env python3
import re
import operator
from collections import defaultdict

reg_err = re.compile("(.*): (\w+)Error: (.*)")
reg_ok = re.compile("(.*): OK")

f = open('test_results.txt')
counts = defaultdict(lambda: defaultdict(lambda: 0))

for line in f:
    #print(line)
    result = reg_err.search(line)
    if result is not None:
        counts[result.group(2)][result.group(3)] += 1
        continue

    result = reg_ok.search(line)
    if result is not None:
        counts["OK"][""] += 1

#for k in sorted(counts.items(), key=operator.itemgetter(1)):
for k in counts.keys():
    print("---- {} ---- {}".format(k, len(counts[k])))
    #for errtext in counts[k].keys():
    for errtext in sorted(counts[k].items(), key=operator.itemgetter(1)):
        print("{}:\t{}".format(errtext[1], errtext[0]))
    print()

print("Recap:")
for k in counts.keys():
    print("{}:\t{} groups,\t{} total".format(k[:6], len(counts[k]),
        sum(v for k, v in counts[k].items())))

ok = sum(v for k, v in counts["OK"].items())
print("Passed: {}/{} = {}%".format(ok, 5295, ok/5295. * 100))
