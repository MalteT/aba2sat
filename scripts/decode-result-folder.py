#!/usr/bin/env python3

import glob
import json
import os
import argparse
import csv

parser = argparse.ArgumentParser()
parser.add_argument("-d", "--directory")
parser.add_argument("-o", "--output")
args = parser.parse_args()

# Path to the folder
folder_path = args.directory if args.directory is not None else "output"
output = args.output if args.output is not None else "all.csv"


def run():
    count = 0
    out = []
    # Using glob to match all .json files
    for file_path in glob.glob(os.path.join(folder_path, "*.json")):
        try:
            # Open and read the contents of the file
            with open(file_path, "r", encoding="utf-8") as json_file:
                (
                    _ident,
                    atom_count,
                    assumption_ratio,
                    max_rules_per_head,
                    max_rule_size,
                    _idx,
                ) = file_path.split("_")
                data = json.load(json_file)["results"]
                aba2sat, aspforaba = (
                    (data[0], data[1])
                    if (
                        data[0]["command"] == "aba2sat"
                        and data[1]["command"] == "aspforaba"
                    )
                    else (data[1], data[0])
                )
                speedup = float(aspforaba['mean']) / float(aba2sat['mean'])
                out.append({
                    "atom_count": atom_count,
                    "assumption_ratio": assumption_ratio,
                    "max_rules_per_head": max_rules_per_head,
                    "max_rule_size": max_rule_size,
                    "time_ours": aba2sat["mean"],
                    "time_theirs": aspforaba['mean'],
                    "stddev": aba2sat['stddev'],
                    "speedup": speedup,
                })
                if count > 700:
                    break
            count += 1
        except Exception:
            print(f'Failed to read {file_path}. Skipping..')
    if len(out) > 0:
        with open(output, 'w') as output_file:
            output_file.write
            writer = csv.DictWriter(output_file, fieldnames=out[0].keys())
            writer.writeheader()
            writer.writerows(out)
    else:
        print('Empty set')

run()
