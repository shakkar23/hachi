CREATE TABLE board_features (
    board_id SERIAL PRIMARY KEY,
    max_height INT,
    bumpiness INT,
    well_x INT,
    well_depth INT,
    max_donated_height INT,
    n_donations INT,
    t_clears INT,
    PRIMARY KEY board_id
);

CREATE TABLE action_features (
    action_id SERIAL PRIMARY KEY,
    piece_type VARCHAR,
    piece_x INT,
    piece_y INT,
    t_spin INT,
    all_spin INT,
    combo INT,
    attack INT,
    damage_cancel INT,
    damage_sent INT
);

CREATE TABLE gamestate_features (
    state_id SERIAL PRIMARY KEY,
    board1 INTEGER REFERENCES board_features(board_id),
    board2 INTEGER REFERENCES board_features(board_id)
);

CREATE TABLE training_data (
    row_id SERIAL PRIMARY KEY,
    gamestate1 INTEGER REFERENCES gamestate_features(state_id),
    gamestate2 INTEGER REFERENCES gamestate_features(state_id),
    action1 INTEGER REFERENCES action_features(action_id),
    action2 INTEGER REFERENCES action_features(action_id),
    player1_score FLOAT,
    player2_score FLOAT
);

CREATE OR REPLACE VIEW denormalized_data AS
SELECT 
    td.row_id,
    td.player1_score,
    td.player2_score,
    
    -- Gamestate 1 features
    gs1.state_id AS gamestate1_id,
    b1.board_id AS board1_id,
    b1.max_height AS board1_max_height,
    b1.bumpiness AS board1_bumpiness,
    b1.well_x AS board1_well_x,
    b1.well_depth AS board1_well_depth,
    b1.max_donated_height AS board1_max_donated_height,
    b1.n_donations AS board1_n_donations,
    b1.t_clears AS board1_t_clears,
    b2.board_id AS board2_id,
    b2.max_height AS board2_max_height,
    b2.bumpiness AS board2_bumpiness,
    b2.well_x AS board2_well_x,
    b2.well_depth AS board2_well_depth,
    b2.max_donated_height AS board2_max_donated_height,
    b2.n_donations AS board2_n_donations,
    b2.t_clears AS board2_t_clears,
    
    -- Gamestate 2 features
    gs2.state_id AS gamestate2_id,
    b3.board_id AS board3_id,
    b3.max_height AS board3_max_height,
    b3.bumpiness AS board3_bumpiness,
    b3.well_x AS board3_well_x,
    b3.well_depth AS board3_well_depth,
    b3.max_donated_height AS board3_max_donated_height,
    b3.n_donations AS board3_n_donations,
    b3.t_clears AS board3_t_clears,
    b4.board_id AS board4_id,
    b4.max_height AS board4_max_height,
    b4.bumpiness AS board4_bumpiness,
    b4.well_x AS board4_well_x,
    b4.well_depth AS board4_well_depth,
    b4.max_donated_height AS board4_max_donated_height,
    b4.n_donations AS board4_n_donations,
    b4.t_clears AS board4_t_clears,
    
    -- Action 1 features
    a1.action_id AS action1_id,
    a1.piece_type AS action1_piece_type,
    a1.piece_x AS action1_piece_x,
    a1.piece_y AS action1_piece_y,
    a1.t_spin AS action1_t_spin,
    a1.all_spin AS action1_all_spin,
    a1.combo AS action1_combo,
    a1.attack AS action1_attack,
    a1.damage_cancel AS action1_damage_cancel,
    a1.damage_sent AS action1_damage_sent,
    
    -- Action 2 features
    a2.action_id AS action2_id,
    a2.piece_type AS action2_piece_type,
    a2.piece_x AS action2_piece_x,
    a2.piece_y AS action2_piece_y,
    a2.t_spin AS action2_t_spin,
    a2.all_spin AS action2_all_spin,
    a2.combo AS action2_combo,
    a2.attack AS action2_attack,
    a2.damage_cancel AS action2_damage_cancel,
    a2.damage_sent AS action2_damage_sent

FROM training_data td
LEFT JOIN gamestate_features gs1 ON td.gamestate1 = gs1.state_id
LEFT JOIN board_features b1 ON gs1.board1 = b1.board_id
LEFT JOIN board_features b2 ON gs1.board2 = b2.board_id
LEFT JOIN gamestate_features gs2 ON td.gamestate2 = gs2.state_id
LEFT JOIN board_features b3 ON gs2.board1 = b3.board_id
LEFT JOIN board_features b4 ON gs2.board2 = b4.board_id
LEFT JOIN action_features a1 ON td.action1 = a1.action_id
LEFT JOIN action_features a2 ON td.action2 = a2.action_id;