"""Probe keys, RAND axis."""
import math
import statistics
from decimal import Decimal

from lib.num import D, F, half_up
from lib.paths import binom_tail, curtate_expectation, dec, draw_curtate, mean_se, rng


def RAND_01(i, t):
    def run():
        r = rng(i["seed"])
        return [draw_curtate(r, i["qs"]) for _ in range(i["paths"])]
    xs = run()
    m, se = mean_se(xs)
    t("seeded mean", dec(m), f"{i['paths']} paths, seed {i['seed']} (rules 1–2)")
    t("standard error", dec(se), "sd / √n")
    same = run() == xs
    t("same seed, same answer", same, "run twice")
    return {"kind": "random", "r": dec(m), "se_r": dec(se), "p": D(i["p"]), "same_seed_same_answer": same}


def RAND_02(i, t):
    e = curtate_expectation(i["qs"])
    t("expected remaining lifetime", e, "Σ chance alive at each year's end (rule 1)")
    return {"kind": "relation", "value": {"expected": D(e), "exact": e}}


def RAND_03(i, t):
    r = rng(i["seed"])
    hits = []
    for _ in range(i["paths"]):
        k = sum(1 for _ in range(i["months"]) if r.random() < float(i["chance"]))
        hits.append(1 if D(i["amount"]) * k > D(i["threshold"]) else 0)
    m, se = mean_se(hits)
    kmin = int(D(i["threshold"]) / D(i["amount"])) + 1
    t("exact share", binom_tail(i["months"], F(D(i["chance"])), kmin), f"P(K ≥ {kmin}), K ~ Bin({i['months']}, {i['chance']})")
    t("seeded share", dec(m), f"{i['paths']} paths, seed {i['seed']} (rules 1–2)")
    return {"kind": "random", "r": dec(m), "se_r": dec(se), "p": D(i["p"])}


def _ends(i, seed):
    r = rng(seed)
    return [D(i["fund"]) * D(repr(math.exp(sum(r.gauss(float(i["mu"]), float(i["sigma"])) for _ in range(i["years"]))))) for _ in range(i["paths"])]


def RAND_04(i, t):
    a = _ends(i, i["seed"])
    b = [x + D(i["cash"]) for x in _ends(i, i["seed"])]
    diffs = [y - x for x, y in zip(a, b)]
    med = t("median B − A, common paths", half_up(statistics.median(diffs)), "rule 2")
    share = t("share B above", D(sum(1 for d in diffs if d > 0)) / len(diffs), "rule 2")
    ind = [x + D(i["cash"]) for x in _ends(i, i["seed"] + 1)]
    t("median B − A, independent seeds (contrast)", half_up(statistics.median([y - x for x, y in zip(a, ind)])), "seeds differ")
    return {"kind": "relation", "value": {"median_difference": med, "share_b_above": share, "paths": len(diffs), "common_random_numbers": True}}
