#!/bin/bash

#SBATCH --ntasks=1
#SBATCH --time=10:00:00

./scripts/aba-generator-acyclic.py
