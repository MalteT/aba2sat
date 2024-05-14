#!/usr/bin/env python3

import random
import argparse

def create_framework(n_sentences, n_assumptions, n_rules_per_head, size_of_bodies, cycle_prob):
    """
    Create a random framework.

    sentences contains the non-assumption sentences.
    n_rules_per_head should be a list exhausting the possible number of rules each head can have
    size_of_bodies should be a list exhausting the possible number of sentences in any rule body
    These should hold in order to get non-counterintuitive results:
    - n_assumptions < n_sentences
    - max(size_of_bodies) <= n_sentences+n_assumptions
    """

    assumptions = [str(i) for i in range(1,n_assumptions+1)]
    sentences = [str(i) for i in range(n_assumptions+1,n_sentences+1)]

    contraries = {asmpt: random.choice(sentences+assumptions) for asmpt in assumptions}

    # order sentences to avoid cycles
    random.shuffle(sentences)
    rules = []
    for i, head in enumerate(sentences):
        n_rules_in_this_head = random.choice(n_rules_per_head)
        for _ in range(n_rules_in_this_head):
            size_of_body = random.choice(size_of_bodies)

            # only allow stuff to occur in body if it is lower in the (topological) order
            n_available = len(assumptions) + i

            selection_set = assumptions+sentences[:i]
            # add potentially cycle creating sentences to the selection set with a given probability
            extra_selection = random.sample(sentences[i:], min(len(sentences[i:]), int(cycle_prob*len(sentences))))
            selection_set.extend(extra_selection)

            #body = random.sample(assumptions+sentences[:i], min(size_of_body, n_available))
            body = random.sample(assumptions+selection_set, min(size_of_body, n_available))
            rules.append((head, body))

    return assumptions, sentences, contraries, rules

def print_ICCMA_format(assumptions, contraries, rules, n_sentences, out_filename):
    """
    Print the given framework in the ICCMA 2023 format.
    """
    offset = len(assumptions)

    with open(out_filename, 'w') as out:
        out.write(f"p aba {n_sentences}\n")
        for i, asm in enumerate(assumptions):
            out.write(f"a {asm}\n")
            #print(f"a {asm}")
        for ctr in contraries:
            out.write(f"c {ctr} {contraries.get(ctr)}\n")
            #print(f"c {ctr} {contraries.get(ctr)}")
        for rule in rules:
            out.write(f"r {rule[0]} {' '.join(rule[1])}\n")
            #print(f"r {rule[0]} {' '.join(rule[1])}")

def ICCMA23_benchmarks(sentences=[1000,2000,3000,4000,5000], max_rules_per_head_list=[5,10], max_rule_size_list=[5,10], assumption_ratios=[0.1,0.3], count=10, directory="iccma23_aba_benchmarks", identifier="aba"):
    random.seed(811543731122527)
    for sentence in sentences:
        for assumption_ratio in assumption_ratios:
            for max_rules_per_head in max_rules_per_head_list:
                for max_rule_size in max_rule_size_list:
                    for i in range(count):
                        number_assumptions = int(round(assumption_ratio*sentence))
                        number_rules_per_head = range(1,max_rules_per_head+1)
                        n_spb = range(1,max_rule_size+1)
                        filename = f"{directory}/{identifier}_{sentence}_{assumption_ratio}_{max_rules_per_head}_{max_rule_size}_{i}.aba"
                        print(filename)
                        framework = create_framework(sentence, number_assumptions, number_rules_per_head, n_spb, 0)
                        query = random.randint(1,number_assumptions)
                        with open(f"{filename}.asm", 'w') as out:
                            print(f"{filename}.asm")
                            out.write(f"{query}")
                        print_ICCMA_format(framework[0], framework[2], framework[3], sentence, filename)

parser = argparse.ArgumentParser()
parser.add_argument('-d', '--directory')
parser.add_argument('-i', '--identifier')
args = parser.parse_args()

ICCMA23_benchmarks(
    sentences = [50,100,200,300,400,500,1000,2000],
    max_rules_per_head_list = [1,2,4,8,16],
    max_rule_size_list = [1,2,4,8,16],
    assumption_ratios = [0.1,0.3,0.5,0.7,0.9],
    count = 5,
    directory=args.directory if args.directory is not None else "acyclic",
    identifier=args.identifier if args.identifier is not None else "aba",
)
