#!/usr/bin/env python3
"""
Spike: local-model Focus/Neutral/Drift classification quality + cost.

Runs test_cases.json (see docs/ARCHITECTURE.md#4-classification-layer for
the design this is validating) against a local Ollama model and reports
accuracy against the expected labels, plus latency.

This is the second highest-risk spike in the project: if local
classification quality is too poor to trust, the "free forever, no cloud
required" pitch weakens significantly (see docs/PROBLEM_STATEMENT.md).

Requirements:
    1. Install Ollama: https://ollama.com/download
    2. Pull a small model to test, e.g.:
         ollama pull llama3.2:3b
       (start small — this is exactly the RAM/quality tradeoff we're
       trying to measure. Try a couple of model sizes if you have time.)
    3. Make sure the Ollama server is running (it usually auto-starts;
       otherwise `ollama serve`).
    4. pip install requests

Usage:
    python3 classify_harness.py --model llama3.2:3b

What to report back:
    1. Overall accuracy against the `expected` labels in test_cases.json.
    2. Which specific cases failed — especially cases 2/3 and 5/6, which
       are the same app with different content and are the whole point of
       "context-aware" classification (this is exactly what ActivityWatch
       cannot do and what Drifty claims to do well).
    3. Average latency per classification — is this fast enough to run
       every ~3 minutes in the background without being annoying?
    4. RAM usage of the Ollama process while this runs (Task
       Manager/Activity Monitor/`htop`) — compare against Drifty's stated
       ~5-7GB for local mode as a rough benchmark.
    5. Try at least 2 model sizes if you can (e.g. a 1-3B and a 7-8B) and
       compare accuracy vs. resource cost — this directly informs which
       model we recommend as the default in v0.3.
"""

import argparse
import json
import time
from pathlib import Path

import requests

OLLAMA_URL = "http://localhost:11434/api/generate"

PROMPT_TEMPLATE = """You are classifying a single 3-minute block of computer activity as \
FOCUS, NEUTRAL, or DRIFT, for a personal time-tracking tool. Judge based on \
whether this activity plausibly serves the user's stated context — not \
based on the app name alone, since the same app can be focus or drift \
depending on content.

User's context: {context_profile}

Recent activity (most recent last): {recent_blocks}

Current activity to classify:
  App: {app}
  Window/tab title: {title}
  URL: {url}

Respond with ONLY a JSON object, no other text, in this exact format:
{{"classification": "focus" | "neutral" | "drift", "category": "<short category label>", "confidence": <0.0-1.0>}}
"""


def classify(model, case):
    block = case["block"]
    prompt = PROMPT_TEMPLATE.format(
        context_profile=case["context_profile"],
        recent_blocks=", ".join(case["recent_blocks"]) or "(none yet)",
        app=block["app"],
        title=block["title"],
        url=block.get("url") or "(none)",
    )

    start = time.time()
    resp = requests.post(
        OLLAMA_URL,
        json={"model": model, "prompt": prompt, "stream": False},
        timeout=60,
    )
    elapsed = time.time() - start
    resp.raise_for_status()
    raw_output = resp.json().get("response", "").strip()

    try:
        # Models sometimes wrap JSON in markdown fences despite instructions
        cleaned = raw_output.replace("```json", "").replace("```", "").strip()
        parsed = json.loads(cleaned)
    except json.JSONDecodeError:
        parsed = {"classification": None, "category": None, "confidence": None, "parse_error": True}

    return parsed, raw_output, elapsed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", default="llama3.2:3b", help="Ollama model tag to test")
    args = parser.parse_args()

    test_cases_path = Path(__file__).parent / "test_cases.json"
    test_cases = json.loads(test_cases_path.read_text())

    correct = 0
    total_latency = 0.0
    results = []

    print(f"Testing model: {args.model}\n")

    for case in test_cases:
        parsed, raw_output, elapsed = classify(args.model, case)
        predicted = parsed.get("classification")
        expected = case["expected"]
        is_correct = predicted == expected
        correct += is_correct
        total_latency += elapsed

        status = "PASS" if is_correct else "FAIL"
        print(f"[{status}] Case {case['id']}: expected={expected} predicted={predicted} "
              f"({elapsed:.1f}s) — {case['note']}")
        if not is_correct:
            print(f"      raw model output: {raw_output!r}")

        results.append({
            "case_id": case["id"],
            "expected": expected,
            "predicted": predicted,
            "correct": is_correct,
            "latency_seconds": elapsed,
            "raw_output": raw_output,
        })

    accuracy = correct / len(test_cases) * 100
    avg_latency = total_latency / len(test_cases)

    print(f"\n--- Summary ---")
    print(f"Model: {args.model}")
    print(f"Accuracy: {correct}/{len(test_cases)} ({accuracy:.0f}%)")
    print(f"Average latency: {avg_latency:.2f}s per classification")

    out_path = Path(__file__).parent / f"results_{args.model.replace(':', '_')}.json"
    out_path.write_text(json.dumps({
        "model": args.model,
        "accuracy_pct": accuracy,
        "avg_latency_seconds": avg_latency,
        "results": results,
    }, indent=2))
    print(f"\nFull results written to {out_path}")


if __name__ == "__main__":
    main()
