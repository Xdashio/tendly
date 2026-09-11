# Spike: local-model classification quality + cost

The second-highest-risk spike in the project. If a small local model can't
reliably tell the difference between, say, "YouTube for research" and
"YouTube for procrastination" given context, the local-first pitch in
[docs/PROBLEM_STATEMENT.md](../../docs/PROBLEM_STATEMENT.md) weakens
significantly, and we may need to lean harder on the BYOK cloud path than
planned.

## Run it

```bash
# 1. Install Ollama: https://ollama.com/download
# 2. Pull a small model to start
ollama pull llama3.2:3b

# 3. Install the one Python dependency
pip install requests

# 4. Run the harness
python3 classify_harness.py --model llama3.2:3b
```

If you have time/RAM to spare, also try a larger model for comparison,
e.g. `ollama pull llama3.1:8b` then rerun with `--model llama3.1:8b`.

## What we're trying to learn

1. **Accuracy** against `test_cases.json` — pay special attention to cases
   2/3 and 5/6, which are deliberately the *same app* with *different
   content* and opposite expected labels. This is the exact thing
   ActivityWatch's raw category system can't do, and the exact thing
   Drifty claims to do well — it's the core value proposition of the
   whole classification layer.
2. **Latency** — needs to be fast enough to run roughly every 3 minutes in
   the background without becoming annoying or draining battery.
3. **RAM usage** of the Ollama process while running — compare loosely
   against Drifty's stated ~5–7GB for their local mode.
4. **Model size trade-off** if you can test more than one size — this
   directly decides what we recommend as the default model in v0.3.

Report back the printed summary and the generated `results_<model>.json`
file, and flag any cases where the model's raw output looked confused
(e.g. couldn't produce valid JSON, or reasoning in the wrong direction).
