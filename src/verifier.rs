//! Exact replay of an Elf Calculator program against the stated rules.
//! This is ground truth: any program that fails here does not ship.

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy)]
pub struct Instr {
    /// register index of the left operand
    pub a: usize,
    /// register index of the right operand
    pub b: usize,
    pub op: Op,
}

#[derive(Debug, Clone)]
pub struct Puzzle {
    /// initial register contents (the puzzle's starting values)
    pub init: Vec<u64>,
    /// targets that must each appear in some register at least once
    pub targets: Vec<u64>,
    /// the calculator stores only 48 values at a time
    pub max_registers: usize,
    /// values above this are rejected
    pub value_limit: u64,
}

#[derive(Debug, PartialEq)]
pub enum VerifyError {
    RegisterOutOfRange { idx: usize, have: usize },
    Overflow { limit: u64 },
    NegativeResult,
    InexactDivision,
    RegisterCap { cap: usize },
    MissingTargets { missing: Vec<u64> },
}

/// Replay `prog` on `puzzle`. Returns Ok(final register set) or the first
/// rule violation. Every instruction counts, even no-op-shaped ones.
pub fn verify(puzzle: &Puzzle, prog: &[Instr]) -> Result<Vec<u64>, VerifyError> {
    let mut regs = puzzle.init.clone();
    if regs.len() > puzzle.max_registers {
        return Err(VerifyError::RegisterCap { cap: puzzle.max_registers });
    }
    let mut seen: HashSet<u64> = regs.iter().copied().collect();
    for (step, ins) in prog.iter().enumerate() {
        let _ = step;
        if ins.a >= regs.len() || ins.b >= regs.len() {
            return Err(VerifyError::RegisterOutOfRange { idx: ins.a.max(ins.b), have: regs.len() });
        }
        let (x, y) = (regs[ins.a], regs[ins.b]);
        let v = match ins.op {
            Op::Add => x.checked_add(y),
            Op::Mul => x.checked_mul(y),
            Op::Sub if x > y => Some(x - y),
            Op::Sub => return Err(VerifyError::NegativeResult),
            Op::Div if y != 0 && x % y == 0 => Some(x / y),
            Op::Div => return Err(VerifyError::InexactDivision),
        };
        let v = v.ok_or(VerifyError::Overflow { limit: puzzle.value_limit })?;
        if v > puzzle.value_limit {
            return Err(VerifyError::Overflow { limit: puzzle.value_limit });
        }
        if regs.len() >= puzzle.max_registers && !seen.contains(&v) {
            return Err(VerifyError::RegisterCap { cap: puzzle.max_registers });
        }
        regs.push(v);
        seen.insert(v);
        // NOTE: the 48-value cap semantics (drop-oldest vs reject-new) must
        // be confirmed against the competition's Data page examples before
        // this verifier is trusted for submission.
    }
    let missing: Vec<u64> = puzzle
        .targets
        .iter()
        .filter(|t| !seen.contains(t))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(VerifyError::MissingTargets { missing });
    }
    Ok(regs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn puzzle() -> Puzzle {
        Puzzle {
            init: vec![2, 3],
            targets: vec![12, 72],
            max_registers: 48,
            value_limit: 10_000,
        }
    }

    #[test]
    fn happy_path() {
        let prog = vec![
            Instr { a: 0, b: 1, op: Op::Add },      // 5
            Instr { a: 2, b: 2, op: Op::Add },      // 10
            Instr { a: 1, b: 3, op: Op::Mul },      // 30? -> regs: 2,3,5,10,30
            Instr { a: 3, b: 1, op: Op::Mul },      // 30
            Instr { a: 4, b: 4, op: Op::Mul },      // 900
        ];
        // targets 12 and 72 not produced -> must fail
        assert!(matches!(verify(&puzzle(), &prog), Err(VerifyError::MissingTargets { .. })));
    }

    #[test]
    fn exact_targets_pass() {
        // 12 = 2*3+2*3? -> 6, then 12 = 6+6; 72 = 12*6
        let prog = vec![
            Instr { a: 0, b: 1, op: Op::Mul }, // 6
            Instr { a: 2, b: 2, op: Op::Add }, // 12
            Instr { a: 3, b: 2, op: Op::Mul }, // 72
        ];
        let regs = verify(&puzzle(), &prog).unwrap();
        assert_eq!(&regs[2..], &[6, 12, 72]);
    }

    #[test]
    fn inexact_division_rejected() {
        let prog = vec![Instr { a: 1, b: 0, op: Op::Div }]; // 3/2
        assert_eq!(verify(&puzzle(), &prog), Err(VerifyError::InexactDivision));
    }

    #[test]
    fn negative_result_rejected() {
        let prog = vec![Instr { a: 0, b: 1, op: Op::Sub }]; // 2-3
        assert_eq!(verify(&puzzle(), &prog), Err(VerifyError::NegativeResult));
    }
}
