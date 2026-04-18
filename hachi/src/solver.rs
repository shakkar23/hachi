use minilp::{ComparisonOp, LinearExpr, OptimizationDirection, Problem};

/// Solve zero-sum NxN games approximately
/// Returns (row_strategy, col_strategy, game_value).
pub fn nash_equilibrium(payoff: &Vec<Vec<f64>>) -> (Vec<f64>, Vec<f64>, f64) {
    fictitious_play(payoff, 1 << 16, 1e-2)
}

pub fn fictitious_play(
    payoff: &Vec<Vec<f64>>,
    max_iters: usize,
    tol: f64,
) -> (Vec<f64>, Vec<f64>, f64) {
    let n = payoff.len();
    assert!(n > 0 && payoff.iter().all(|r| r.len() == n), "Must be NxN matrix");

    // Flat row-major + transpose, both contiguous
    let mut payoff_rm = vec![0.0f64; n * n];
    let mut payoff_cm = vec![0.0f64; n * n]; // transpose: payoff_cm[j*n + i] = payoff[i][j]
    for i in 0..n {
        for j in 0..n {
            payoff_rm[i * n + j] = payoff[i][j];
            payoff_cm[j * n + i] = payoff[i][j];
        }
    }

    let mut row_scores = vec![0.0; n];
    let mut col_scores = vec![0.0; n];
    let mut row_counts = vec![0u64; n];
    let mut col_counts = vec![0u64; n];

    let mut row_action = 0usize;
    let mut col_action = 0usize;

    let mut best_upper = f64::INFINITY;
    let mut best_lower = f64::NEG_INFINITY;

    for t in 1..=max_iters {
        // Row player: add column `col_action` of payoff to row_scores.
        // Column of row-major == row of transpose → contiguous slice.
        row_counts[row_action] += 1;
        let col_slice = &payoff_cm[col_action * n..(col_action + 1) * n];
        for i in 0..n {
            row_scores[i] += col_slice[i];
        }

        let (row_best_response, max_row_score) = argmax(&row_scores);
        row_action = row_best_response;

        // Column player: add row `row_action` of payoff to col_scores → already contiguous in row-major.
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

    let value = (0..n)
        .map(|i| {
            let row = &payoff_rm[i * n..(i + 1) * n];
            (0..n).map(|j| row_probs[i] * col_probs[j] * row[j]).sum::<f64>()
        })
        .sum();

    (row_probs, col_probs, value)
}

/// Solve for Nash equilibrium of a zero-sum NxN payoff matrix using LP
/// Returns (row_strategy, col_strategy, game_value)
pub fn nash_equilibrium_exact(payoff: &Vec<Vec<f64>>) -> (Vec<f64>, Vec<f64>, f64) {
    let n = payoff.len();
    assert!(n > 0 && payoff.iter().all(|r| r.len() == n), "Must be NxN matrix");

    // Maximiser
    // sum_i p_i * payoff[i][j] >= v 
    // sum p_i = 1 
    // p_i >= 0
    let (row_probs, row_value) = {
        let mut problem = Problem::new(OptimizationDirection::Maximize);

        // p_i >= 0, objective coefficient 0
        let p: Vec<_> = (0..n).map(|_| problem.add_var(0.0, (0.0, f64::INFINITY))).collect();
        // v is free: use (-inf, inf)
        let v = problem.add_var(1.0, (f64::NEG_INFINITY, f64::INFINITY));

        // For each column j: sum_i p_i * payoff[i][j] >= v
        // sum_i p_i * payoff[i][j] - v >= 0
        for j in 0..n {
            let mut lhs = LinearExpr::empty();
            for i in 0..n {
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

    // Minimiser
    // sum_j q_j * payoff[i][j] <= w  
    // sum q_j = 1 
    // q_j >= 0
    let col_probs = {
        let mut problem = Problem::new(OptimizationDirection::Minimize);

        let q: Vec<_> = (0..n).map(|_| problem.add_var(0.0, (0.0, f64::INFINITY))).collect();
        let w = problem.add_var(1.0, (f64::NEG_INFINITY, f64::INFINITY));

        // For each row i: sum_j q_j * payoff[i][j] <= w
        // sum_j q_j * payoff[i][j] - w <= 0
        for i in 0..n {
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
    assert!((value - 0.0).abs() < 1e-2,  "Game should be symmetric");
}

#[test]
fn test_solver_dominant_strategy() {
    // Row 0 dominates: always better than Row 1 and Row 2
    // Col 2 dominates: always better for col player than Col 0 and Col 1
    // Nash equilibrium should be pure: Row 0, Col 0 with value 3.0
    let payoff = vec![
        vec![0.3, 0.2, 0.1],  // Row 0
        vec![0.2, 0.1, 0.0],  // Row 1
        vec![0.1, 0.0, -0.1], // Row 2
    ];
    /* Viewed from column perspective:
    let payoff = vec![
        vec![-0.3, -0.2, -0.1],  // Row 0
        vec![-0.2, -0.1,  0.0],  // Row 1
        vec![-0.1,  0.0,  0.1], // Row 2
    ];
    So clearly move 2 dominates for the column player.
    */

    let (row, col, value) = nash_equilibrium(&payoff);

    println!("Row strategy:    {:?}", row.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Column strategy: {:?}", col.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Game value:      {:.6}", value);

    assert!((row[0] - 1.0).abs() < 1e-2, "Row should play strategy 0 with prob 1");
    assert!((col[2] - 1.0).abs() < 1e-2, "Col should play strategy 2 with prob 1");
    assert!((value - 0.1).abs() < 1e-2,  "Game value should be 0.1");
}

#[test]
fn test_solver_rps5() {
    // Rock-Paper-Scissors-Lizard-Spock (RPSLS)
    // Each move beats 2 others, loses to 2 others
    // Win/loss = +1/-1, draw = 0
    // Nash equilibrium: uniform 1/5 each, value = 0
    //
    //           R     P     Sc    L     Sp
    let payoff = vec![
        vec![ 0.0, -1.0, -1.0,  1.0,  1.0], // Rock    (crushes Lizard, crushes Scissors)
        vec![ 1.0,  0.0, -1.0, -1.0,  1.0], // Paper   (covers Rock, disproves Spock)
        vec![ 1.0,  1.0,  0.0, -1.0, -1.0], // Scissors(cuts Paper, decapitates Lizard)
        vec![-1.0,  1.0,  1.0,  0.0, -1.0], // Lizard  (poisons Spock, eats Paper)
        vec![-1.0, -1.0,  1.0,  1.0,  0.0], // Spock   (smashes Scissors, vaporizes Rock)
    ];

    let (row, col, value) = nash_equilibrium(&payoff);

    println!("Row strategy:    {:?}", row.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Column strategy: {:?}", col.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Game value:      {:.6}", value);

    // Uniform mix, zero-sum symmetric game
    for (i, &p) in row.iter().enumerate() {
        assert!((p - 0.2).abs() < 1e-2, "Row strategy {i} should be 0.2, got {p:.6}");
    }
    for (i, &p) in col.iter().enumerate() {
        assert!((p - 0.2).abs() < 1e-2, "Col strategy {i} should be 0.2, got {p:.6}");
    }
    assert!(value.abs() < 1e-2, "Game value should be 0.0, got {value:.6}");
}

