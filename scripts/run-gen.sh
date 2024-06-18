#!/bin/bash
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=32
#SBATCH --time=10:00:00

./aba-generator-acyclic --directory "$(pwd)/instances"
