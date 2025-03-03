#!/usr/bin/env python3

import json
import os
import csv
from typing import NewType, TypedDict, Union

Instance = NewType("Instance", str)

class JobSummary(TypedDict):
    array_id: int
    task_id: int
    mem_requested: int
    status: str
    flags: str
    state: str
    time: float

class Analysis(TypedDict):
    assumptions: int
    rules: int
    rule_body_average: float
    rule_body_minimum: float
    rule_body_maximum: float
    rule_body_stddev: float
    loops: int
    loops_status: str

class Data(TypedDict):
    aba2sat: Union[JobSummary, None]
    aspforaba: Union[JobSummary, None]
    atom_count: int
    assumption_ratio: float
    max_rules_per_head: int
    max_rule_size: int
    loop_percent: float
    solved_correctly: bool
    file: Instance
    analysis: Union[Analysis, None]

assert os.path.isdir("instances"), "The current directory is missing 'instances', is this a test-run folder?"
assert os.path.isfile("instances.list"), "The current directory is missing 'instances.list', is this a test-run folder?"
assert os.path.isdir("output"), "The current directory is missing 'output', is this a test-run folder?"
assert os.path.isdir("slurms"), "The current directory is missing 'slurms', is this a test-run folder?"

def convert_to_csv(data: dict[Instance, Data], output_file='output.csv'):
    # Define the exact fields we need based on the known structure
    fieldnames = [
        'file',
        'aba2sat_mem_requested', 'aba2sat_array_id', 'aba2sat_task_id',
        'aba2sat_status', 'aba2sat_flags', 'aba2sat_state', 'aba2sat_time',
        'aspforaba_mem_requested', 'aspforaba_array_id', 'aspforaba_task_id',
        'aspforaba_status', 'aspforaba_flags', 'aspforaba_state', 'aspforaba_time',
        'atom_count', 'assumption_ratio', 'max_rules_per_head',
        'max_rule_size', 'loop_percent', 'solved_correctly', 'speedup', 'analysis_assumptions',
        'analysis_loops', 'analysis_loops_status', 'analysis_rule_body_average',
        'analysis_rule_body_maximum', 'analysis_rule_body_minimum', 'analysis_rule_body_stddev',
        'analysis_rules',
    ]

    rows = []

    # Process each entry in the data
    for filename in data.keys():
        details = data[filename]
        assert details['aba2sat'] is not None, f"Missing aba2sat run for {filename}"
        assert details['aspforaba'] is not None, f"Missing aspforaba run for {filename}"
        row = {
            'file': filename,
            'aba2sat_mem_requested': details['aba2sat']['mem_requested'],
            'aba2sat_array_id': details['aba2sat']['array_id'],
            'aba2sat_task_id': details['aba2sat']['task_id'],
            'aba2sat_status': details['aba2sat']['status'],
            'aba2sat_flags': details['aba2sat']['flags'],
            'aba2sat_state': details['aba2sat']['state'],
            'aba2sat_time': details['aba2sat']['time'],
            'aspforaba_mem_requested': details['aspforaba']['mem_requested'],
            'aspforaba_array_id': details['aspforaba']['array_id'],
            'aspforaba_task_id': details['aspforaba']['task_id'],
            'aspforaba_status': details['aspforaba']['status'],
            'aspforaba_flags': details['aspforaba']['flags'],
            'aspforaba_state': details['aspforaba']['state'],
            'aspforaba_time': details['aspforaba']['time'],
            'atom_count': details['atom_count'],
            'assumption_ratio': details['assumption_ratio'],
            'max_rules_per_head': details['max_rules_per_head'],
            'max_rule_size': details['max_rule_size'],
            'loop_percent': details['loop_percent'],
            'solved_correctly': details['solved_correctly'],
            'speedup': details['aspforaba']['time'] / details['aba2sat']['time'],
            'analysis_assumptions': details['analysis']['assumptions'] if details['analysis'] != None else None,
            'analysis_loops': details['analysis']['loops'] if details['analysis'] != None else None,
            'analysis_loops_status': details['analysis']['loops_status'] if details['analysis'] != None else None,
            'analysis_rule_body_average': details['analysis']['rule_body_average'] if details['analysis'] != None else None,
            'analysis_rule_body_maximum': details['analysis']['rule_body_maximum'] if details['analysis'] != None else None,
            'analysis_rule_body_minimum': details['analysis']['rule_body_minimum'] if details['analysis'] != None else None,
            'analysis_rule_body_stddev': details['analysis']['rule_body_stddev'] if details['analysis'] != None else None,
            'analysis_rules': details['analysis']['rules'] if details['analysis'] != None else None
        }
        rows.append(row)

    # Write to CSV
    with open(output_file, 'w', newline='') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    print(f"Data saved to {output_file}")

def run():
    data: dict[Instance, Data] = {}

    # Read all the instances
    with open("instances.list") as content:
        files = [Instance(os.path.basename(file.strip())) for file in content.readlines()]
        for file in files:
            # Read results and compare them
            aba2sat_result_file = os.path.join("output", f"{file}-aba2sat-result")
            aspforaba_result_file = os.path.join("output", f"{file}-aspforaba-result")
            analysis_file = os.path.join("output", f"{file}-analysis.json")
            solved_correctly = False
            with open(aba2sat_result_file, 'r') as aba2sat, open(aspforaba_result_file, 'r') as aspforaba:
                our = aba2sat.read().strip()
                their = aspforaba.read().strip()
                solved_correctly = our == their
            analysis: Union[Analysis, None] = None
            if os.path.exists(analysis_file):
                with open(analysis_file, 'r') as f:
                    try:
                        analysis = json.load(f)
                    except json.JSONDecodeError as e:
                        print(f"Error decoding JSON: {analysis_file}")
                    except Exception as e:
                        print(f"An unexpected error occurred: {e}")
            (
                _,
                atom_count,
                assumption_ratio,
                max_rules_per_head,
                max_rule_size,
                loop_percent,
                _,
            ) = file.split("_")
            info: Data = {
                "aba2sat": None,
                "aspforaba": None,
                "atom_count": int(atom_count),
                "assumption_ratio": float(assumption_ratio),
                "max_rules_per_head": int(max_rules_per_head),
                "max_rule_size": int(max_rule_size),
                "loop_percent": float(loop_percent),
                "solved_correctly": solved_correctly,
                "file": file,
                "analysis": analysis
            }
            data[file] = info

    # Concat the Jobinfo JSON files
    json_files = [file for file in os.listdir(".") if file.startswith('jobinfo-') and file.endswith('.json')]
    assert len(json_files), "No jobinfo files found, is this a test-run folder?"
    for file in json_files:
        # Decode file name
        (_, program, batch) = file.rstrip(".json").split("-")
        # Open file
        with open(file, 'r') as content:
            # Parse json content
            content = json.load(content)
            # Iterate over all jobs
            for job in content["jobs"]:
                # Calculate instance idx
                nr = (int(batch) - 1) * 15000 + int(job["array"]["task_id"]["number"]) - 1
                instance = files[nr]
                time = float(job["time"]["total"]["seconds"]) + float(job["time"]["total"]["microseconds"]) / 1000000
                summary: JobSummary = {
                    # Extract the requested memory
                    "mem_requested": [item["count"] for item in job["tres"]["requested"] if item["type"] == "mem"][0],
                    "array_id": job["array"]["job_id"],
                    "task_id": job["array"]["task_id"]["number"],
                    "status": job["derived_exit_code"]["status"][0],
                    "flags": " ".join(job["flags"]),
                    "state": job["state"]["current"][0],
                    "time": time,
                }
                if (program == "aspforaba"):
                    data[instance]["aspforaba"] = summary
                elif (program == "aba2sat"):
                    data[instance]["aba2sat"] = summary

    convert_to_csv(data, "all.csv")

run()
