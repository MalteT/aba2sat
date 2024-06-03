#!/usr/bin/env bash

# Batch script to run on sc.uni-leipzig.de cluster, i used
# sbatch -a "1-$(cat acyclic.list | wc -l)" ./scripts/sc-batch.sh

# Somehow all paths used in spawned processes need to be absolute,
# there's probably a good explanation, but I don't have it

FILE_LIST=acyclic.list

# Pick line `$SLURM_ARRAY_TASK_ID` from the FILE_LIST
# This will probably cause issues if more processes are allocated
# than lines in the FILE_LIST, but who knows
file="$(awk "NR == $SLURM_ARRAY_TASK_ID" "$FILE_LIST")"
basefile="$(basename "$file")"
# Read the extra argument
arg=$(cat "$(pwd)/$file.asm")

# Make sure we get all the data in one central place
OUTPUT_DIR="output"

singularity run \
  --env OUTPUT_DIR=/out \
  --bind "$(pwd)/acyclic:/in:ro" \
  --bind "$(pwd)/$OUTPUT_DIR:/out" \
  validate.sif \
  validate --file "/in/$basefile" --arg "$arg" --time --problem dc-co
