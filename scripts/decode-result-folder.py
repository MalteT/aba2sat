#!/usr/bin/env python3

import json
import os
import csv

assert os.path.isdir("instances"), "The current directory is missing 'instances', is this a test-run folder?"
assert os.path.isfile("instances.list"), "The current directory is missing 'instances.list', is this a test-run folder?"
assert os.path.isdir("output"), "The current directory is missing 'output', is this a test-run folder?"
assert os.path.isdir("slurms"), "The current directory is missing 'slurms', is this a test-run folder?"

def run():
    # Concat the Jobinfo JSON files
    json_files = [file for file in os.listdir(".") if file.startswith('jobinfo-') and file.endswith('.json')]
    assert len(json_files), "No jobinfo files found, is this a test-run folder?"
    jobs = []
    for file in json_files:
        with open(file, 'r') as content:
            jobs.extend(json.load(content)["jobs"])

    # Read all the instances
    with open("instances.list") as content:
        files = [file.strip() for file in content.readlines()]

    # Construct output
    out = []
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
            loop_percent,
            _idx_with_file_end,
        ) = file.split("_")

        # Read results and compare them
        aba2sat_result_file = os.path.join("output", f"{file}-aba2sat-result")
        aspforaba_result_file = os.path.join("output", f"{file}-aspforaba-result")
        solved_correctly = False
        with open(aba2sat_result_file, 'r') as aba2sat, open(aspforaba_result_file, 'r') as aspforaba:
            our = aba2sat.read().strip()
            their = aspforaba.read().strip()
            solved_correctly = our == their

        speedup = None
        time_ours = None
        time_theirs = None
        stddev = None

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
        with open("all.csv", 'w') as output_file:
            output_file.write
            writer = csv.DictWriter(output_file, fieldnames=out[0].keys())
            writer.writeheader()
            writer.writerows(out)
    else:
        print('Empty set')


run()
