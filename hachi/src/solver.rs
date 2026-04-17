use minilp::{ComparisonOp, LinearExpr, OptimizationDirection, Problem};

/// Solve for Nash equilibrium of a zero-sum NxN payoff matrix
/// Returns (row_strategy, col_strategy, game_value)
pub fn nash_equilibrium(payoff: &Vec<Vec<f64>>) -> (Vec<f64>, Vec<f64>, f64) {
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
    assert!((value - 0.0).abs() < 1e-4,  "Game should be symmetric");
}

#[test]
fn test_solver_dominant_strategy() {
    // Row 0 dominates: always better than Row 1 and Row 2
    // Col 2 dominates: always better for col player than Col 0 and Col 1
    // Nash equilibrium should be pure: Row 0, Col 0 with value 3.0
    let payoff = vec![
        vec![3.0, 2.0, 1.0],  // Row 0
        vec![2.0, 1.0, 0.0],  // Row 1
        vec![1.0, 0.0, -1.0], // Row 2
    ];
    /* Viewed from column perspective:
    let payoff = vec![
        vec![-3.0, -2.0, -1.0],  // Row 0
        vec![-2.0, -1.0,  0.0],  // Row 1
        vec![-1.0,  0.0,  1.0], // Row 2
    ];
    So clearly move 2 dominates for the column player.
    */

    let (row, col, value) = nash_equilibrium(&payoff);

    println!("Row strategy:    {:?}", row.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Column strategy: {:?}", col.iter().map(|x| format!("{:.4}", x)).collect::<Vec<_>>());
    println!("Game value:      {:.6}", value);

    assert!((row[0] - 1.0).abs() < 1e-4, "Row should play strategy 0 with prob 1");
    assert!((col[2] - 1.0).abs() < 1e-4, "Col should play strategy 0 with prob 1");
    assert!((value - 1.0).abs() < 1e-4,  "Game value should be 3.0");
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
        assert!((p - 0.2).abs() < 1e-4, "Row strategy {i} should be 0.2, got {p:.6}");
    }
    for (i, &p) in col.iter().enumerate() {
        assert!((p - 0.2).abs() < 1e-4, "Col strategy {i} should be 0.2, got {p:.6}");
    }
    assert!(value.abs() < 1e-4, "Game value should be 0.0, got {value:.6}");
}