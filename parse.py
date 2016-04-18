#!/usr/bin/env python3
import re
import operator
from collections import defaultdict
import sys

COMMON_ERRS_ONLY = True

reg_err = re.compile("sputnik/([\w\.]+/)*([\w\.]+)/.*: (\w+)Error: (.*)")
reg_ok = re.compile("sputnik/([\w\.]+/)*([\w\.]+)/.*: OK")

f = open('test_results.txt')
counts = defaultdict(lambda: defaultdict(lambda: 0))
total = 0
all_groups = set()

for line in f:
    result = reg_err.search(line)
    if result is not None:
        counts[result.group(3)][result.group(4)] += 1
        all_groups.add(result.group(2))
        continue

    result = reg_ok.search(line)
    if result is not None:
        all_groups.add(result.group(2))
        counts["OK"][result.group(2)] += 1
        continue

    sys.stderr.write(line)

#for k in sorted(counts.items(), key=operator.itemgetter(1)):
for k in counts.keys():
    print("---- {} ---- {} different errors".format(k, len(counts[k])))
    print("Count\tError message".format(k, len(counts[k])))
    #for errtext in counts[k].keys():
    for errtext in sorted(counts[k].items(), key=operator.itemgetter(1)):
        if COMMON_ERRS_ONLY and int(errtext[1]) > 10:
            print("{}:\t{}".format(errtext[1], errtext[0]))
        elif COMMON_ERRS_ONLY is False:
            print("{}:\t{}".format(errtext[1], errtext[0]))
    print()

print("Recap:")
for k in counts.keys():
    total += sum(v for k, v in counts[k].items())
    print("{}:\t{} groups,\t{} total".format(k[:6], len(counts[k]),
        sum(v for k, v in counts[k].items())))

print("Number of categories with at least some passes: %s/%s" % (len(counts["OK"]), len(all_groups)))
ok = sum(v for k, v in counts["OK"].items())
print("Passed: {}/{} = {}%".format(ok, total, ok/total * 100))
