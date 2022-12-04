#!/usr/bin/env python3
import subprocess
import yaml
import sys
import os
try:
    from yaml import CLoader as Loader, CDumper as Dumper
except ImportError:
    from yaml import Loader, Dumper

def prune_single_act(act_id):
    print("Pruning", act_id)
    result = subprocess.run(["cargo", "run", "--release", act_id], check=True, stdout=subprocess.PIPE).stdout

    fixup_filename = "data/fixups/{}/{}.yml".format(
        act_id.split('.', 1)[0],
        act_id
    )
    good_fixup = yaml.load(open(fixup_filename, 'rt'), Loader=Loader)

    to_delete = 0
    while to_delete<len(good_fixup):
        fixup_to_try = list(good_fixup)
        del fixup_to_try[to_delete]
        with open(fixup_filename, 'wt') as f:
            yaml.dump(fixup_to_try, f, Dumper=Dumper, allow_unicode=True, width=100000, sort_keys=False)

        new_result = subprocess.run(["cargo", "run", "--release", act_id], stdout=subprocess.PIPE)
        if new_result.returncode == 0 and new_result.stdout == result:
            print("Pruned {} from {}".format(good_fixup[to_delete], fixup_filename))
            good_fixup = fixup_to_try
        else:
            to_delete +=1

    if good_fixup:
        with open(fixup_filename, 'wt') as f:
            yaml.dump(good_fixup, f, Dumper=Dumper, allow_unicode=True, width=100000, sort_keys=False)
    else:
        os.remove(fixup_filename)

for root, dirs, files in os.walk('data/fixups'):
    for file in files:
        if file.endswith('.yml'):
            prune_single_act(file[:-4])
