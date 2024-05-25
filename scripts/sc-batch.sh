#!/usr/bin/env bash

# Batch script to run on sc.uni-leipzig.de cluster, i used
# sbatch -a "1-$(cat acyclic.list | wc -l)" ./scripts/sc-batch.sh

# Somehow all paths used in spawned processes need to be absolute,
# there's probably a good explanation, but I don't have it

FILE_LIST=acyclic.list

# Pick line `$SLURM_ARRAY_TASK_ID` from the FILE_LIST
# This will probably cause issues if more processes are allocated
# than lines in the FILE_LIST, but who knows
file="$(pwd)/$(awk "NR == $SLURM_ARRAY_TASK_ID" "$FILE_LIST")"
# Read the extra argument
arg=$(cat "$file.asm")

# Make sure we get all the data in one central place
OUTPUT_DIR="$(pwd)/output"
export OUTPUT_DIR

# This assumes that `validate` accepts the --no-rm flag,
# which is not a flag the script accepts, but recognized by
# the default bundler `nix bundle .#validate` uses. Required here
# to prevent the fastest process from cleaning the extracted
# package. Slower processes or those allocated later *will* fail
# without the flag
./validate --no-rm // --file "$file" --arg "$arg" --time --problem dc-co
