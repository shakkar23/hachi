#include "downstacker.hpp"

#include "block.hpp"

#include "util/rng.hpp"

#include <algorithm>
#include <ranges>
#include <cassert>
#include <unordered_map>

struct Node {
    Board board;
    double eval_score;
    double line_clear_eval;
    RNG rng;
    std::array<char, 5> queue;
    Piece root_piece;
    char hold;
};


constexpr static int first_empty_row_from_bottom(Board board) {
    // Iterate through underlying integers from last to first
    for(int i = Board::last; i >= 0; --i) {
        uint64_t chunk = board.get_under(i);

        // For each row in this chunk (from bottom to top)
        for(int row_in_chunk = 0; row_in_chunk < Board::lines_per_under; ++row_in_chunk) {
            int y = i * Board::lines_per_under + (Board::lines_per_under - row_in_chunk - 1);

            // Skip if we're beyond actual height
            if(y >= Board::height) continue;

            // Create mask for this specific row
            uint64_t row_mask = (uint64_t(1) << Board::width) - 1;  // W ones
            row_mask <<= (row_in_chunk * Board::width);

            // For the last chunk, we might have partial rows
            if(i == Board::last && row_in_chunk >= (Board::height % Board::lines_per_under)) {
                continue;  // This row doesn't exist
            }

            // Check if this row is completely empty
            if((chunk & row_mask) == 0) {
                return y;
            }
        }
    }
    return -1;  // All rows have at least one occupied cell
}

Piece bot_downstacker(Board board, std::span<char, 5> queue, char hold, uint8_t bag, size_t beam_depth, size_t beam_width, size_t speculation_split_size) {

    std::array<char,5> real_queue;
    for(size_t i = 0; i < queue.size(); i++) {
        real_queue[i] = queue[i];
    }
    std::vector<Node> games = {
        Node{
            .board = board,
            .eval_score = std::numeric_limits<double>::min(),
            .line_clear_eval = 0,
            .rng = RNG(bag),
            .queue = real_queue,
            .root_piece = {' ',4,20,0},
            .hold = hold
        }
    };
	std::vector<Node> next_games;

    games.reserve(beam_width * 40);
    next_games.reserve(beam_width * 40);

    using namespace reachability;

    auto go = [](std::vector<Node> &next_games, const Node& node, bool held, bool set_root_piece = false) {
        char current_piece = held ? node.queue[0] : (node.hold == ' ' ? node.queue[1] : node.hold);
        bool first_hold = node.hold == ' ' && held;

        blocks::call_with_block<blocks::SRS>(current_piece, [&]<block B>() {
            auto moves = search::binary_bfs<blocks::SRS, coord{4,20}, 0uz>(node.board, current_piece);
            for(size_t rot = 0; rot < moves.size(); ++rot) {

                static_for<Board::height>([&](auto y) {
                    static_for<Board::width>([&](auto x) {
                        if(moves[rot].template get<x,y>()) {
                            Node n = node;
                            static_for<B.BLOCK_PER_MINO>([&](const std::size_t mino_i) {
                                int px = x + B.minos[rot][mino_i][0];
                                int py = y + B.minos[rot][mino_i][1];
                                n.board.set(px, py);
                            });
                            if(held && !first_hold) {
                                std::swap(n.hold, n.queue[0]);
                            } else if(first_hold) {
                                n.hold = n.queue[0];
                                std::shift_left(n.queue.begin(), n.queue.end(), 1);
                                n.queue.back() = n.rng.getPiece();
                            }
                            std::shift_left(n.queue.begin(), n.queue.end(), 1);
                            n.queue.back() = n.rng.getPiece();
                            
                            int lines_cleared = n.board.clear_full_lines();
                            n.eval_score = first_empty_row_from_bottom(n.board);
                            n.line_clear_eval += std::array{-100,200,300,400,2000}[lines_cleared];
                            if(set_root_piece) {
                                n.root_piece = Piece{
                                    .t = current_piece, 
                                    .x = (uint8_t)x,
                                    .y = (uint8_t)y,
                                    .rot = (uint8_t)rot
                                };
                            }
                            next_games.push_back(n);
                        }
                    });
                });
            }
        });
    };

    go(next_games, games.front(), false, true);
    go(next_games, games.front(), true, true);
    std::swap(games, next_games);
    next_games.clear();

	for (int depth = 0; depth < std::min(beam_depth, queue.size()); depth++) {

		if (games.size() > beam_width) {
			std::ranges::nth_element(games, games.begin() + beam_width, [](auto& l, auto& r) {
                return l.eval_score + l.line_clear_eval > r.eval_score + r.line_clear_eval;
            });

			auto threshold = games[beam_width].eval_score + games[beam_width].line_clear_eval;

			std::ranges::partition(games, [threshold](const auto& a) {
				return a.eval_score + a.line_clear_eval > threshold;
			});

			games.erase(games.begin() + beam_width, games.end());
		}
        for(auto& game : games) {
            go(next_games, game, false);
            go(next_games, game, true);
        }
        std::swap(games, next_games);
        next_games.clear();
    }


    auto pack_piece = [](Piece piece) {
        return  (uint32_t)piece.t << 0 |
                (uint32_t)piece.x << 8 |
                (uint32_t)piece.y << 16 |
                (uint32_t)piece.rot << 24;
    };

    Piece best_root_piece{ ' ',0,0,0 };
    
    if(!games.empty() && queue.size() < beam_depth) {
        std::ranges::nth_element(games, games.begin(), [](auto& l, auto& r) {
            return l.eval_score + l.line_clear_eval > r.eval_score + r.line_clear_eval;
        });
        best_root_piece = games.front().root_piece;
    }

    std::unordered_map<uint32_t, double> avg_map;
    double best_avg_score = std::numeric_limits<double>::min();

    for(int i = 0; i < speculation_split_size; ++i) {
        std::vector<Node> speculation_games = games;
        { // make all the rngs the same here
            RNG base_rng = RNG(bag);
            std::array<char, queue.size()> next_queue;
            for(auto &q : next_queue) {
                q = base_rng.getPiece();
            }
            for(auto &game : speculation_games) {
                game.rng = base_rng;
                game.queue = next_queue;
            }
        }
        std::unordered_map<uint32_t, double> max_map;
        
        for(int depth = queue.size(); depth < beam_depth; depth++) {

            if(speculation_games.size() > beam_width) {
                std::ranges::nth_element(speculation_games, speculation_games.begin() + beam_width, [](auto& l, auto& r) {
                    return l.eval_score + l.line_clear_eval > r.eval_score + r.line_clear_eval;
                });

                auto threshold = speculation_games[beam_width].eval_score + speculation_games[beam_width].line_clear_eval;

                std::ranges::partition(speculation_games, [threshold](const auto& a) {
                    return a.eval_score + a.line_clear_eval > threshold;
                });

                speculation_games.erase(speculation_games.begin() + beam_width, speculation_games.end());
            }
            for(auto& game : speculation_games) {
                go(next_games, game, false);
                go(next_games, game, true);
            }
            std::swap(speculation_games, next_games);
            next_games.clear();
        }


        for(auto& game : speculation_games) {
            uint32_t packed_piece = pack_piece(game.root_piece);

            max_map[packed_piece] = std::max(max_map[packed_piece], game.eval_score + game.line_clear_eval);

            avg_map[packed_piece] = avg_map[packed_piece] + max_map[packed_piece];

            if(avg_map[packed_piece] > best_avg_score) {
                best_root_piece = game.root_piece;
                best_avg_score = avg_map[packed_piece];
            }
        }
    }
    
    return best_root_piece;
}