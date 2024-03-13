"""
Copyright <2023> <Tuomo Lehtonen, University of Helsinki>

Permission is hereby granted, free of charge, to any person obtaining a copy of this
software and associated documentation files (the "Software"), to deal in the Software
without restriction, including without limitation the rights to use, copy, modify,
merge, publish, distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice shall be included in all copies
or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT
OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.
"""

import sys, random
import argparse

def create_framework(n_sentences, n_assumptions, n_rules_per_head, size_of_bodies):
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

    rules = []
    for head in sentences:
        n_rules_in_this_head = random.choice(n_rules_per_head)
        for _ in range(n_rules_in_this_head):
            size_of_body = random.choice(size_of_bodies)
            #pool = set(assumptions+sentences)
            pool = assumptions+sentences
            pool.remove(head)
            #sorted(pool)
            body = random.sample(pool, size_of_body)
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

def print_ASP(assumptions, contraries, rules, query, out_filename):
    """
    Print the given framework in ASP format.
    """
    with open(out_filename, 'w') as out:
        for asm in assumptions:
            out.write(f"assumption(a{asm}).\n")
        for ctr in contraries:
            out.write(f"contrary(a{ctr},a{contraries.get(ctr)}).\n")
        for i, rule in enumerate(rules):
            out.write(f"head({str(i)},a{rule[0]}).\n")
            if rule[1]:
                for body in rule[1]:
                    out.write(f"body({str(i)},a{body}).\n")
        out.write(f"query(a{query}).\n")

def ICCMA23_benchmarks(sens, directory="iccma23_aba_benchmarks", identifier="aba"):
   random.seed(811543731122527)
   n_rules_max = [5,10]
   rule_size_max = [5,10]
   asmpt_ratio = [0.1,0.3]
   for sen in sens:
       for k in asmpt_ratio:
           for rph_max in n_rules_max:
               for spb_max in rule_size_max:
                   for i in range(10):
                       n_a = int(round(k*sen))
                       n_rph = range(1,rph_max+1)
                       n_spb = range(1,spb_max+1)
                       filename = f"{directory}/{identifier}_{sen}_{k}_{rph_max}_{spb_max}_{i}.aba"
                       print(filename)
                       framework = create_framework(sen, n_a, n_rph, n_spb)
                       print_ICCMA_format(framework[0], framework[2], framework[3], sen, filename)

ICCMA23_benchmarks(sens=[25,100,500,2000,5000])
