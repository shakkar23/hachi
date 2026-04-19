use minilp::{ComparisonOp, LinearExpr, OptimizationDirection, Problem};

const PRECISION: f64 = 1e-2;

/// Solve zero-sum MxN games approximately
/// Returns (row_strategy, col_strategy, game_value).
pub fn nash_equilibrium(payoff: &Vec<Vec<f64>>) -> (Vec<f64>, Vec<f64>, f64) {
    fictitious_play(payoff, 1 << 16, PRECISION)
}

pub fn fictitious_play(
    payoff: &Vec<Vec<f64>>,
    max_iters: usize,
    tol: f64,
) -> (Vec<f64>, Vec<f64>, f64) {
    let m = payoff.len();
    assert!(m > 0, "Matrix must have at least one row");
    let n = payoff[0].len();
    assert!(n > 0, "Matrix must have at least one column");
    assert!(
        payoff.iter().all(|r| r.len() == n),
        "All rows must have the same length"
    );

    // Flat row-major (m x n) + transpose (n x m), both contiguous.
    // payoff_rm[i*n + j] = payoff[i][j]
    // payoff_cm[j*m + i] = payoff[i][j]
    let mut payoff_rm = vec![0.0f64; m * n];
    let mut payoff_cm = vec![0.0f64; m * n];
    for i in 0..m {
        for j in 0..n {
            payoff_rm[i * n + j] = payoff[i][j];
            payoff_cm[j * m + i] = payoff[i][j];
        }
    }

    let mut row_scores = vec![0.0; m];
    let mut col_scores = vec![0.0; n];
    let mut row_counts = vec![0u64; m];
    let mut col_counts = vec![0u64; n];

    let mut row_action = 0usize;
    let mut col_action = 0usize;

    let mut best_upper = f64::INFINITY;
    let mut best_lower = f64::NEG_INFINITY;

    for t in 1..=max_iters {
        // Row player: add column `col_action` of payoff to row_scores.

        row_counts[row_action] += 1;
        let col_slice = &payoff_cm[col_action * m..(col_action + 1) * m];
        for i in 0..m {
            row_scores[i] += col_slice[i];
        }

        let (row_best_response, max_row_score) = argmax(&row_scores);
        row_action = row_best_response;

        col_counts[col_action] += 1;
        let row_slice = &payoff_rm[row_action * n..(row_action + 1) * n];
        for j in 0..n {
            col_scores[j] += row_slice[j];
        }

        let (column_best_response, min_col_score) = argmin(&col_scores);
        col_action = column_best_response;

        let tf = t as f64;
        best_upper = best_upper.min(max_row_score / tf);
        best_lower = best_lower.max(min_col_score / tf);

        if t % 100 == 0 && (best_upper - best_lower) < tol {
            break;
        }
    }

    let total_row: u64 = row_counts.iter().sum();
    let total_col: u64 = col_counts.iter().sum();
    let row_probs: Vec<f64> = row_counts.iter().map(|&c| c as f64 / total_row as f64).collect();
    let col_probs: Vec<f64> = col_counts.iter().map(|&c| c as f64 / total_col as f64).collect();

    let value = (0..m)
        .map(|i| {
            let row = &payoff_rm[i * n..(i + 1) * n];
            (0..n).map(|j| row_probs[i] * col_probs[j] * row[j]).sum::<f64>()
        })
        .sum();

    (row_probs, col_probs, value)
}

/// Solve for Nash equilibrium of a zero-sum MxN payoff matrix using LP
/// Returns (row_strategy, col_strategy, game_value)
pub fn nash_equilibrium_exact(payoff: &Vec<Vec<f64>>) -> (Vec<f64>, Vec<f64>, f64) {
    let m = payoff.len();
    assert!(m > 0, "Matrix must have at least one row");
    let n = payoff[0].len();
    assert!(n > 0, "Matrix must have at least one column");
    assert!(
        payoff.iter().all(|r| r.len() == n),
        "All rows must have the same length"
    );

    // Maximiser (row player has m actions)
    // sum_i p_i * payoff[i][j] >= v   for each column j (n constraints)
    // sum p_i = 1
    // p_i >= 0
    let (row_probs, row_value) = {
        let mut problem = Problem::new(OptimizationDirection::Maximize);

        // p_i >= 0, objective coefficient 0
        let p: Vec<_> = (0..m).map(|_| problem.add_var(0.0, (0.0, f64::INFINITY))).collect();
        // v is free: use (-inf, inf)
        let v = problem.add_var(1.0, (f64::NEG_INFINITY, f64::INFINITY));

        // For each column j: sum_i p_i * payoff[i][j] - v >= 0
        for j in 0..n {
            let mut lhs = LinearExpr::empty();
            for i in 0..m {
                lhs.add(p[i], payoff[i][j]);
            }
            lhs.add(v, -1.0);
            problem.add_constraint(lhs, ComparisonOp::Ge, 0.0);
        }

        // sum_i p_i = 1
        let mut sum_p = LinearExpr::empty();
        for &pi in &p {
            sum_p.add(pi, 1.0);
        }
        problem.add_constraint(sum_p, ComparisonOp::Eq, 1.0);

        let sol = problem.solve().expect("Row LP infeasible");
        let probs: Vec<f64> = p.iter().map(|&pi| sol[pi].max(0.0)).collect();
        let val = sol[v];
        (probs, val)
    };

    // Minimiser (column player has n actions)
    // sum_j q_j * payoff[i][j] <= w   for each row i (m constraints)
    // sum q_j = 1
    // q_j >= 0
    let col_probs = {
        let mut problem = Problem::new(OptimizationDirection::Minimize);

        let q: Vec<_> = (0..n).map(|_| problem.add_var(0.0, (0.0, f64::INFINITY))).collect();
        let w = problem.add_var(1.0, (f64::NEG_INFINITY, f64::INFINITY));

        // For each row i: sum_j q_j * payoff[i][j] - w <= 0
        for i in 0..m {
            let mut lhs = LinearExpr::empty();
            for j in 0..n {
                lhs.add(q[j], payoff[i][j]);
            }
            lhs.add(w, -1.0);
            problem.add_constraint(lhs, ComparisonOp::Le, 0.0);
        }

        // sum_j q_j = 1
        let mut sum_q = LinearExpr::empty();
        for &qj in &q {
            sum_q.add(qj, 1.0);
        }
        problem.add_constraint(sum_q, ComparisonOp::Eq, 1.0);

        let sol = problem.solve().expect("Col LP infeasible");
        q.iter().map(|&qj| sol[qj].max(0.0)).collect::<Vec<f64>>()
    };

    (row_probs, col_probs, row_value)
}

#[inline]
fn argmax(v: &[f64]) -> (usize, f64) {
    let mut best = 0;
    let mut best_val = v[0];
    for (i, &x) in v.iter().enumerate().skip(1) {
        if x > best_val { best = i; best_val = x; }
    }
    (best, best_val)
}

#[inline]
fn argmin(v: &[f64]) -> (usize, f64) {
    let mut best = 0;
    let mut best_val = v[0];
    for (i, &x) in v.iter().enumerate().skip(1) {
        if x < best_val { best = i; best_val = x; }
    }
    (best, best_val)
}

#[test]
fn test_solver() {
    // RPS test case
    let payoff = vec![
        vec![ 0.0, -1.0,  1.0],  // Rock
        vec![ 1.0,  0.0, -1.0],  // Paper
        vec![-1.0,  1.0,  0.0],  // Scissors
    ];

    let (row, col, value) = nash_equilibrium(&payoff);

    println!("Row strategy:    {:?}", row.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Column strategy: {:?}", col.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Game value:      {:.6}", value);

    assert!((row[0] - 0.33).abs() < 1e-1, "Rock should have probability 1/3");
    assert!((col[0] - 0.33).abs() < 1e-1, "Paper should have probability 1/3");
    assert!((value - 0.0).abs() < PRECISION*2.0,  "Game should be symmetric");
}

#[test]
fn test_solver_dominant_strategy() {
    // Row 0 dominates: always better than Row 1 and Row 2
    // Col 2 dominates: always better for col player than Col 0 and Col 1
    // Nash equilibrium should be pure: Row 0, Col 2 with value 0.1
    let payoff = vec![
        vec![0.3, 0.2, 0.1],  // Row 0
        vec![0.2, 0.1, 0.0],  // Row 1
        vec![0.1, 0.0, -0.1], // Row 2
    ];

    let (row, col, value) = nash_equilibrium(&payoff);

    println!("Row strategy:    {:?}", row.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Column strategy: {:?}", col.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Game value:      {:.6}", value);

    assert!((row[0] - 1.0).abs() < PRECISION*2.0, "Row should play strategy 0 with prob 1");
    assert!((col[2] - 1.0).abs() < PRECISION*2.0, "Col should play strategy 2 with prob 1");
    assert!((value - 0.1).abs() < PRECISION*2.0,  "Game value should be 0.1");
}

#[test]
fn test_solver_rps5() {
    // Rock-Paper-Scissors-Lizard-Spock (RPSLS)
    //           R     P     Sc    L     Sp
    let payoff = vec![
        vec![ 0.0, -1.0, -1.0,  1.0,  1.0],
        vec![ 1.0,  0.0, -1.0, -1.0,  1.0],
        vec![ 1.0,  1.0,  0.0, -1.0, -1.0],
        vec![-1.0,  1.0,  1.0,  0.0, -1.0],
        vec![-1.0, -1.0,  1.0,  1.0,  0.0],
    ];

    let (row, col, value) = nash_equilibrium(&payoff);

    for (i, &p) in row.iter().enumerate() {
        assert!((p - 0.2).abs() < PRECISION*2.0, "Row strategy {i} should be 0.2, got {p:.6}");
    }
    for (i, &p) in col.iter().enumerate() {
        assert!((p - 0.2).abs() < PRECISION*2.0, "Col strategy {i} should be 0.2, got {p:.6}");
    }
    assert!(value.abs() < PRECISION*2.0, "Game value should be 0.0, got {value:.6}");
}

#[test]
fn test_non_square_tall() {
    let payoff = vec![
        vec![ 1.0, -1.0],
        vec![-1.0,  1.0],
        vec![ 0.0,  0.0],
        vec![-1.0, -1.0],
    ];

    let (row, col, value) = nash_equilibrium(&payoff);
    println!("tall row: {:?}", row);
    println!("tall col: {:?}", col);
    println!("tall val: {}", value);

    assert_eq!(row.len(), 4);
    assert_eq!(col.len(), 2);
    // Row 3 is strictly dominated — should have ~0 probability.
    assert!(row[3] < PRECISION*2.0, "Dominated row should have near-zero probability, got {}", row[3]);
    // Column play must be ~uniform on the two columns.
    assert!((col[0] - 0.5).abs() < PRECISION*2.0);
    assert!((col[1] - 0.5).abs() < PRECISION*2.0);
    assert!(value.abs() < PRECISION*2.0);
}

#[test]
fn test_non_square_wide() {
    let payoff = vec![
        vec![ 1.0, -1.0, 0.0, 1.0],
        vec![-1.0,  1.0, 0.0, 1.0],
    ];

    let (row, col, value) = nash_equilibrium(&payoff);
    println!("wide row: {:?}", row);
    println!("wide col: {:?}", col);
    println!("wide val: {}", value);

    assert_eq!(row.len(), 2);
    assert_eq!(col.len(), 4);
    // Column 3 is strictly dominated for the column player.
    assert!(col[3] < PRECISION*2.0, "Dominated column should have near-zero probability, got {}", col[3]);
    assert!((row[0] - 0.5).abs() < PRECISION*2.0);
    assert!((row[1] - 0.5).abs() < PRECISION*2.0);
    assert!(value.abs() < PRECISION*2.0);
}

#[test]
fn test_non_square_exact() {
    let payoff = vec![
        vec![ 1.0, -1.0],
        vec![-1.0,  1.0],
        vec![ 0.0,  0.0],
    ];

    let (row, col, value) = nash_equilibrium_exact(&payoff);
    println!("exact non-square row: {:?}", row);
    println!("exact non-square col: {:?}", col);
    println!("exact non-square val: {}", value);

    assert_eq!(row.len(), 3);
    assert_eq!(col.len(), 2);
    assert!((col[0] - 0.5).abs() < 1e-6);
    assert!((col[1] - 0.5).abs() < 1e-6);
    assert!(value.abs() < 1e-6);
}