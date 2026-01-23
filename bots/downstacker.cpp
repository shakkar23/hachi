#include "downstacker.hpp"

#include "block.hpp"

#include "util/rng.hpp"

#include <algorithm>
#include <ranges>
#include <cassert>

struct Node {
    Board board;
    double eval_score;
    double line_clear_eval;
    RNG rng;
    std::array<char, 5> queue;
    Piece root_piece;
    char hold;
};

Piece bot_downstacker(Board board, std::span<char, 5> queue, char hold) {
    constexpr int beam_depth = 4;
    constexpr size_t beam_width = 100;

    ColBoard column_board = ColBoard::convert(board);

    std::array<char,5> real_queue;
    for(size_t i = 0; i < queue.size(); i++) {
        real_queue[i] = queue[i];
    }
    std::vector<Node> games = {
        Node{
            .board = board,
            .eval_score = std::numeric_limits<double>::min(),
            .line_clear_eval = 0,
            .rng = RNG(),
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
                            n.eval_score = ColBoard::convert(n.board).evaluate(lines_cleared);
                            n.line_clear_eval += std::array{-200,200,250,400,700}[lines_cleared];
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


    go(next_games, games.front(), true, true);
    go(next_games, games.front(), false, true);
    std::swap(games, next_games);
    next_games.clear();

	for (int depth = 1; depth < beam_depth; depth++) {

		if (games.size() > beam_width) {
			std::ranges::nth_element(games, games.begin() + beam_width, [](auto& l, auto& r) {
                return l.eval_score + l.line_clear_eval > r.eval_score + r.line_clear_eval;
            });

			auto threshold = games[beam_width].eval_score + games[beam_width].line_clear_eval;

			std::ranges::partition(games, [threshold](const auto& a) {
				// assume all the games have been evaluated
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
    
    if(games.empty()) {
        return Piece{' ',0,0,0};
    }

    std::ranges::nth_element(games, games.begin(), [](auto& l, auto& r) {
        return l.eval_score + l.line_clear_eval > r.eval_score + r.line_clear_eval;
    });

    return games.front().root_piece;
}