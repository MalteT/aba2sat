#!/usr/bin/env python3

import argparse
import random
import os
import concurrent.futures


def create_framework(
    n_sentences, n_assumptions, n_rules_per_head, size_of_bodies, cycle_prob
):
    """
    Create a random framework.

    sentences contains the non-assumption sentences.
    n_rules_per_head should be a list exhausting the possible number of rules each head can have
    size_of_bodies should be a list exhausting the possible number of sentences in any rule body
    These should hold in order to get non-counterintuitive results:
    - n_assumptions < n_sentences
    - max(size_of_bodies) <= n_sentences+n_assumptions
    """

    assumptions = [str(i) for i in range(1, n_assumptions + 1)]
    sentences = [str(i) for i in range(n_assumptions + 1, n_sentences + 1)]

    contraries = {
        asmpt: random.choice(sentences + assumptions) for asmpt in assumptions
    }

    # order sentences to avoid cycles
    random.shuffle(sentences)
    rules = []
    for i, head in enumerate(sentences):
        n_rules_in_this_head = random.choice(n_rules_per_head)
        for _ in range(n_rules_in_this_head):
            size_of_body = random.choice(size_of_bodies)

            # only allow stuff to occur in body if it is lower in the (topological) order
            n_available = len(assumptions) + i

            selection_set = assumptions + sentences[:i]
            # add potentially cycle creating sentences to the selection set with a given probability
            extra_selection = random.sample(
                sentences[i:], min(len(sentences[i:]), int(cycle_prob * len(sentences)))
            )
            selection_set.extend(extra_selection)

            # body = random.sample(assumptions+sentences[:i], min(size_of_body, n_available))
            body = random.sample(
                assumptions + selection_set, min(size_of_body, n_available)
            )
            rules.append((head, body))

    return assumptions, sentences, contraries, rules


def print_ICCMA_format(assumptions, contraries, rules, n_sentences, out_filename):
    """
    Print the given framework in the ICCMA 2023 format.
    """
    offset = len(assumptions)

    with open(out_filename, "w") as out:
        out.write(f"p aba {n_sentences}\n")
        for i, asm in enumerate(assumptions):
            out.write(f"a {asm}\n")
            # print(f"a {asm}")
        for ctr in contraries:
            out.write(f"c {ctr} {contraries.get(ctr)}\n")
            # print(f"c {ctr} {contraries.get(ctr)}")
        for rule in rules:
            out.write(f"r {rule[0]} {' '.join(rule[1])}\n")
            # print(f"r {rule[0]} {' '.join(rule[1])}")


def generate_configs(
    sentences,
    max_rules_per_head_list,
    max_rule_size_list,
    assumption_ratios,
    cycle_props,
    count,
    directory,
    identifier,
):
    for sentence in sentences:
        for assumption_ratio in assumption_ratios:
            for max_rules_per_head in max_rules_per_head_list:
                for max_rule_size in max_rule_size_list:
                    for cycle_prop in cycle_props:
                        for i in range(count):
                            yield {
                                "sentence": sentence,
                                "assumption_ratio": assumption_ratio,
                                "max_rules_per_head": max_rules_per_head,
                                "max_rule_size": max_rule_size,
                                "cycle_prop": cycle_prop,
                                "count": i,
                                "directory": directory,
                                "identifier": identifier,
                            }


def executor_main(config):
    sentence = config["sentence"]
    assumption_ratio = config["assumption_ratio"]
    max_rules_per_head = config["max_rules_per_head"]
    max_rule_size = config["max_rule_size"]
    cycle_prop = config["cycle_prop"]
    i = config["count"]
    directory = config["directory"]
    identifier = config["identifier"]

    number_assumptions = int(round(assumption_ratio * sentence))
    number_rules_per_head = range(1, max_rules_per_head + 1)
    n_spb = range(1, max_rule_size + 1)

    filename = f"{directory}/{identifier}_{sentence}_{assumption_ratio}_{max_rules_per_head}_{max_rule_size}_{cycle_prop}_{i}.aba"
    print(filename)
    framework = create_framework(
        sentence, number_assumptions, number_rules_per_head, n_spb, cycle_prop
    )
    query = random.randint(1, number_assumptions)
    with open(f"{filename}.asm", "w") as out:
        print(f"{filename}.asm")
        out.write(f"{query}")
    print_ICCMA_format(framework[0], framework[2], framework[3], sentence, filename)


def ICCMA23_benchmarks(
    sentences=[1000, 2000, 3000, 4000, 5000],
    max_rules_per_head_list=[5, 10],
    max_rule_size_list=[5, 10],
    assumption_ratios=[0.1, 0.3],
    cycle_props=[0, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0],
    count=10,
    directory="iccma23_aba_benchmarks",
    identifier="aba",
):
    random.seed(811543731122527)
    os.makedirs(directory, exist_ok=True)

    with concurrent.futures.ProcessPoolExecutor(max_workers=os.cpu_count()) as executor:
        print(f"Starting generation in {directory}..")
        configs = generate_configs(
            sentences=sentences,
            max_rules_per_head_list=max_rules_per_head_list,
            max_rule_size_list=max_rule_size_list,
            assumption_ratios=assumption_ratios,
            cycle_props=cycle_props,
            count=count,
            directory=directory,
            identifier=identifier,
        )
        list(executor.map(executor_main, configs))
        print("Done")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-d", "--directory", required=True, help="Output directory for benchmarks")
    parser.add_argument("-i", "--identifier", required=True, help="Identifier for the benchmark files")
    parser.add_argument("--sentences", nargs="+", type=int, required=True, help="List of sentence counts")
    parser.add_argument("--max-rules-per-head", nargs="+", type=int, required=True, help="List of maximum rules per head")
    parser.add_argument("--max-rule-size", nargs="+", type=int, required=True, help="List of maximum rule sizes")
    parser.add_argument("--assumption-ratios", nargs="+", type=float, required=True, help="List of assumption ratios")
    parser.add_argument("--cycle-props", nargs="+", type=float, required=True, help="List of cycle proportions")
    parser.add_argument("--count", type=int, required=True, help="Number of benchmarks to generate")
    args = parser.parse_args()

    ICCMA23_benchmarks(
        sentences=args.sentences,
        max_rules_per_head_list=args.max_rules_per_head,
        max_rule_size_list=args.max_rule_size,
        assumption_ratios=args.assumption_ratios,
        cycle_props=args.cycle_props,
        count=args.count,
        directory=args.directory,
        identifier=args.identifier,
    )
