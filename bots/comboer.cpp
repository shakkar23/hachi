#include "comboer.hpp"
#include "block.hpp"

#include "util/movegen.hpp"
#include "util/rng.hpp"

#include <random>
#include <algorithm>
#include <ranges>
#include <cassert>
#include <unordered_map>

struct ComboNode {
    Board board;
    RNG rng;
    std::array<char, 5> queue;
    Piece root_piece;
    char hold;
};


static void go(std::vector<ComboNode>&next_games, const ComboNode node, bool held, bool set_root_piece = false) {
    char current_piece = held ? node.queue[0] : (node.hold == ' ' ? node.queue[1] : node.hold);
    bool first_hold = node.hold == ' ' && held;
    using namespace reachability;
    auto moves = movegen(node.board, current_piece);
    for(size_t rot = 0; rot < moves.size(); ++rot) {
        
        static_for<Board::height>([&](auto y) {
            static_for<Board::width>([&](auto x) {
                if(moves[rot].template get<x,y>()) {
                    ComboNode n = node;

                    blocks::call_with_block<blocks::SRS>(current_piece, [&]<block B>() {
                        static_for<B.BLOCK_PER_MINO>([&](const std::size_t mino_i) {
                            int px = x + B.minos[rot][mino_i][0];
                            int py = y + B.minos[rot][mino_i][1];
                            n.board.set(px, py);
                        });
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

                    if(lines_cleared) {
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
                }
            });
        });
    }
};

Piece bot_comboer(const Board board, const std::span<char, 5> queue, const char hold, const uint8_t bag, const size_t beam_depth, const size_t speculation_split_size) {

    std::array<char,5> real_queue;
    for(size_t i = 0; i < queue.size(); i++) {
        real_queue[i] = queue[i];
    }
    std::vector<ComboNode> games;
    games.emplace_back(
            board,
            RNG(bag),
            real_queue,
            Piece{.t=' ',.x=0,.y=0,.rot=0},
            hold
    );
	std::vector<ComboNode> next_games;

    //games.reserve(20 * 40);
    //next_games.reserve(20 * 40);
    go(next_games, games.at(0), false, true);
    go(next_games, games.at(0), true, true);

    if(next_games.empty()) {
        return games.at(0).root_piece;
    }
    std::swap(games, next_games);
    next_games.clear();

	for (int depth = 0; depth < std::min(beam_depth, queue.size()-1); depth++) {

        for(auto &game : games) {
            go(next_games, game, false);
            go(next_games, game, true);
        }
        if(next_games.empty()) {
            return games.at(0).root_piece;
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
    
    if(queue.size() - 1 < beam_depth) {
        best_root_piece = games.at(0).root_piece;
    }

    std::unordered_map<uint32_t, double> avg_map;
        std::unordered_map<uint32_t, int> n_map;
    double best_avg_score = std::numeric_limits<double>::min();

    for(int I = 0; I < speculation_split_size; ++I) {
        std::vector<ComboNode> speculation_games = games;
        { // make all the rngs the same here
            RNG base_rng = RNG(bag);
            for(auto &game : speculation_games) {
                game.rng = base_rng;

                // no hold has happened yet
                if(hold == ' ') {
                    game.queue[0] = game.queue.back();
                    for(int i = 1; i < game.queue.size(); ++i) {
                        game.queue[i] = game.rng.getPiece();
                    }
                } else {
                    for(int i = 0; i < game.queue.size(); ++i) {
                        game.queue[i] = game.rng.getPiece();
                    }
                }
            }
        }
        
        int depth = queue.size() - 1;

        for(;depth < beam_depth; depth++) {

            for(auto& game : speculation_games) {
                go(next_games, game, false);
                go(next_games, game, true);
            }
            if(next_games.empty()) {
                break;
            }

            std::swap(speculation_games, next_games);
            next_games.clear();
        }

        n_map.clear();
        for(auto& game : speculation_games) {
            uint32_t piece = pack_piece(game.root_piece);

            int n_int = ++n_map[piece];
            double n = (double)n_int;
            avg_map[piece] = avg_map[piece] * (n-1) / n + depth / n;
            n_map[piece] += 1;

            if(avg_map[piece] > best_avg_score) {
                best_root_piece = game.root_piece;
                best_avg_score = avg_map[piece];
            }
        }
    }
    
    return best_root_piece;
}