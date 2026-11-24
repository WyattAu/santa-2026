# santa-2026

Solver for [Kaggle Santa 2026 — The Elf Calculator](https://www.kaggle.com/competitions/santa-2026).
Write minimal arithmetic programs: 48-register calculator, ops + - * / over
positive integers (division must be exact), a value ceiling, every target
produced at least once. Score = total instruction count over 1000 puzzles;
lower wins. $50k pool, closes **2027-01-21**.

Join on Kaggle (account needed), download `puzzles.csv` into `data/`.

- `cargo run --bin verify` — replay every program in `data/submission.csv`
  against `data/puzzles.csv`, exact-rule check, per-puzzle step counts.
- `cargo run --bin search` — the solver (BFS/dominance skeleton; the
  competitive core is meet-in-the-middle over register-set states).

Rules source: competition Data page. The verifier is ground truth — every
candidate program passes it before submission.
