#!/usr/bin/env python3

import json
import os
import argparse
import csv


parser = argparse.ArgumentParser()
parser.add_argument("-d", "--directory")
parser.add_argument("-o", "--output")
# The jobinfo file
# Generated with `sacct -j JOB_ID --format=all --json > jobinfo.json`
parser.add_argument("-j", "--jobinfo")
# File list that was used when scheduling the task
parser.add_argument("-l", "--file-list", dest="file_list")
args = parser.parse_args()

# Path to the folder
folder_path = args.directory if args.directory is not None else "output"
output = args.output if args.output is not None else "all.csv"
jobinfo_path = args.jobinfo if args.jobinfo is not None else "jobinfo.json"
file_list_path = args.file_list if args.file_list is not None else "acyclic.list"


def run():
    # Read the list of files to match with their id
    with open(file_list_path, 'r') as file_list:
        files = [file.strip() for file in file_list.readlines()]
    # Open the generated jobinfo
    with open(jobinfo_path, 'r') as jobinfo_file:
        out = []
        jobs = json.load(jobinfo_file)["jobs"]
        for job in jobs:
            array_id = job["array"]["job_id"]
            task_id = job["array"]["task_id"]["number"]
            status = job["derived_exit_code"]["status"][0]
            flags = ",".join(job["flags"])
            state = job["state"]["current"][0]
            file = os.path.basename(files[task_id - 1])

            # Extract the requested memory
            mem_requested = [item["count"] for item in job["tres"]["requested"] if item["type"] == "mem"][0]

            (
                _ident,
                atom_count,
                assumption_ratio,
                max_rules_per_head,
                max_rule_size,
                #loop_percent,
                _idx_with_file_end,
            ) = file.split("_")
            loop_percent = 0

            aba2sat_result_file = f"{file}-aba2sat-result"
            aspforaba_result_file = f"{file}-aspforaba-result"
            hyperfine_file = f"{file}-hyperfine.json"

            solved_correctly = False
            speedup = None
            time_ours = None
            time_theirs = None
            stddev = None
            # Only override the values if the job succesfully ended, this guarantees correct values
            if state == "COMPLETED":
                try:
                    aba2sat_result_path = os.path.join(folder_path, aba2sat_result_file)
                    aspforaba_result_path = os.path.join(folder_path, aspforaba_result_file)
                    with open(aba2sat_result_path, 'r') as aba2sat_result, open(aspforaba_result_path, 'r') as aspforaba_result:
                        aba2sat = aba2sat_result.read()
                        aspforaba = aspforaba_result.read()
                        if aba2sat == aspforaba:
                            solved_correctly = True
                except Exception:
                    print("No result files")
                hyperfine_path = os.path.join(folder_path, hyperfine_file)
                try:
                    with open(hyperfine_path, 'r') as hyperfine:
                        data = json.load(hyperfine)["results"]
                        aba2sat, aspforaba = (
                            (data[0], data[1])
                            if (
                                data[0]["command"] == "aba2sat"
                                and data[1]["command"] == "aspforaba"
                            )
                            else (data[1], data[0])
                        )
                        speedup = float(aspforaba['mean']) / float(aba2sat['mean'])
                        time_ours = aba2sat["mean"]
                        time_theirs = aspforaba['mean']
                        stddev = aba2sat['stddev']
                except Exception:
                    print("No hyperfine file")

            out.append({
                "array_id": array_id,
                "task_id": task_id,
                "mem_requested": mem_requested,
                "status": status,
                "flags": flags,
                "state": state,
                "file": file,

                "atom_count": atom_count,
                "assumption_ratio": assumption_ratio,
                "max_rules_per_head": max_rules_per_head,
                "max_rule_size": max_rule_size,
                "loop_percent": loop_percent,

                "solved_correctly": solved_correctly,

                "time_ours": time_ours,
                "time_theirs": time_theirs,
                "stddev": stddev,
                "speedup": speedup,
            })
    if len(out) > 0:
        with open(output, 'w') as output_file:
            output_file.write
            writer = csv.DictWriter(output_file, fieldnames=out[0].keys())
            writer.writeheader()
            writer.writerows(out)
    else:
        print('Empty set')


run()
