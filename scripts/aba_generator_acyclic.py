import random
import argparse
import sys

def create_framework(n_sentences, n_assumptions, n_rules_per_head,
    size_of_bodies, cycle_prob):
    """
    Create a random framework.

    sentences contains the non-assumption sentences.
    n_rules_per_head should be a list exhausting the possible number of rules each head can have
    size_of_bodies should be a list exhausting the possible number of sentences in any rule body
    These should hold in order to get non-counterintuitive results:
    - n_assumptions < n_sentences
    - max(size_of_bodies) <= n_sentences+n_assumptions
    """

    assumptions = ["a" + str(i) for i in range(n_assumptions)]
    sentences = ["s" + str(i) for i in range(n_sentences-n_assumptions)]

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

def print_ASP(assumptions, contraries, rules, out_filename, query=None):
    """
    Print the given framework in ASP format.
    """
    with open(out_filename, 'w') as out:
        for asm in assumptions:
            out.write("assumption(" + asm + ").\n")
        for ctr in contraries:
            out.write("contrary(" + ctr + "," + contraries.get(ctr) + ").\n")
        for i, rule in enumerate(rules):
            out.write("head(" + str(i) + "," + rule[0] + ").\n")
            if rule[1]:
                for body in rule[1]:
                    out.write("body(" + str(i) + "," + body + ").\n")
        if query:
            out.write("query(" + query + ").")

n_sentences = int(sys.argv[1])
cycle_prob = float(sys.argv[2])
max_rules_per_head = 5
max_body_size = 5
n_a = int(round(0.15*n_sentences))
n_rph = range(1,max_rules_per_head+1)
n_spb = range(1,max_body_size)

framework = create_framework(n_sentences, n_a, n_rph, n_spb, cycle_prob)
print_ASP(framework[0], framework[2], framework[3], "generated_benchmark.asp", "s0")


parser = argparse.ArgumentParser()
parser.add_argument('-d', '--directory')
parser.add_argument('-i', '--identifier')
args = parser.parse_args()

directory = args.directory
identifier = args.identifier

sens = [1000,2000,3000,4000,5000]
n_rules_max = [2,5,8,13]
rule_size_max = [2,5,8,13]
asmpt_ratio = [0.15,0.3,0.7]
for sen in sens:
    for k in asmpt_ratio:
        for rph_max in n_rules_max:
            for spb_max in rule_size_max:
                for i in range(10):
                    n_a = int(round(k*sen))
                    n_rph = range(1,rph_max+1)
                    n_spb = range(1,spb_max+1)
                    filename = f"{directory}/{identifier}_{sen}_{k}_{rph_max}_{spb_max}_{i}.asp"
                    print(filename)
                    framework = create_framework(sen, n_a, n_rph, n_spb)
                    print_ASP(framework[0], framework[2], framework[3], filename)
