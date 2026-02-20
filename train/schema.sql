CREATE TABLE board_features (
    board_id VARCHAR PRIMARY KEY,
    max_height FLOAT,
    bumpiness FLOAT,
    well_x FLOAT,
    well_depth FLOAT,
    max_donated_height FLOAT,
    n_donations FLOAT,
    t_clears FLOAT,
    PRIMARY KEY board_id
);

CREATE TABLE action_features (
    action_id VARCHAR PRIMARY KEY,
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
    state_id VARCHAR PRIMARY KEY,
    board1 VARCHAR REFERENCES board_features(board_id),
    board2 VARCHAR REFERENCES board_features(board_id)
);

CREATE TABLE training_data (
    row_id SERIAL PRIMARY KEY,
    gamestate1 VARCHAR REFERENCES gamestate_features(state_id),
    gamestate2 VARCHAR REFERENCES gamestate_features(state_id),
    action1 VARCHAR REFERENCES action_features(action_id),
    action2 VARCHAR REFERENCES action_features(action_id),
    player1_score FLOAT,
    player2_score FLOAT
);