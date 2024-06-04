#!/usr/bin/env bash
#SBATCH --time=4:00:00
#SBATCH --mem=4G

# Batch script to run on sc.uni-leipzig.de cluster, i used
# > sbatch -a "1-$(cat acyclic.list | wc -l)" ./scripts/sc-batch.sh

# or

# > sbatch -a "1-14999" --mail-type BEGIN,END,FAIL --output 'slurms/slurm-%A_%a.out' --job-name "ABA2SAT-$(date)" ./scripts/sc-batch.sh
# > sbatch -a "15000-22680" --mail-type BEGIN,END,FAIL --output 'slurms/slurm-%A_%a.out' --job-name "ABA2SAT-$(date)" ./scripts/sc-batch.sh
# for when there's more than 15k files..

# Somehow all paths used in spawned processes need to be absolute,
# there's probably a good explanation, but I don't have it

FILE_LIST=acyclic.list
# Calculate the task index based on the SLURM array task ID and an optional offset.
# The OFFSET variable is set to 0 if it is not already defined.
NR="$(("${OFFSET:=0}" + "$SLURM_ARRAY_TASK_ID"))"

# Pick line `$SLURM_ARRAY_TASK_ID` from the FILE_LIST
# This will probably cause issues if more processes are allocated
# than lines in the FILE_LIST, but who knows
file="$(awk "NR == $NR" "$FILE_LIST")"
basefile="$(basename "$file")"
# Read the extra argument
arg=$(cat "$(pwd)/$file.asm")

# Make sure we get all the data in one central place
OUTPUT_DIR="output"

# Run singularity, this is basically docker
# We mount /in and /out inside the container and use that to read and write

# validate.sif is a converted format, easily done with
# > singularity build validate.sif docker-archive://<your-docker-image.tar.gz>
# The docker image was build using
# > nix bundle .#validate --bundler github:NixOS/bundlers#toDockerImage
# on my local machine to create an agnostic package that could run on the cluster
singularity run \
  --env OUTPUT_DIR=/out \
  --bind "$(pwd)/acyclic:/in:ro" \
  --bind "$(pwd)/$OUTPUT_DIR:/out" \
  validate.sif \
  validate --file "/in/$basefile" --arg "$arg" --time --problem dc-co
