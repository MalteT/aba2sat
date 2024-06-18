#!/bin/bash

#SBATCH --ntasks=1
#SBATCH --time=10:00:00

./scripts/aba_generator_acyclic.py
