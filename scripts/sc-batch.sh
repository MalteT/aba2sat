#!/usr/bin/env bash

# Batch script to run on sc.uni-leipzig.de cluster, i used
# sbatch -a "1-$(cat acyclic.list | wc -l)" ./scripts/sc-batch.sh

FILE_LIST=acyclic.list

file="$(pwd)/$(awk "NR == $SLURM_ARRAY_TASK_ID" "$FILE_LIST")"
arg=$(cat "$file.asm")

OUTPUT_DIR="$(pwd)/output"

export OUTPUT_DIR
./validate --file "$file" --arg "$arg" --time --problem dc-co
