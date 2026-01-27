#include "movegen.hpp"
#include "search.hpp"

reachability::static_vector<Board, 4> movegen(const Board board,const char piece) {
    return reachability::blocks::call_with_block<reachability::blocks::SRS> (piece, [&]<reachability::block B>() {
        auto tmp = reachability::search::binary_bfs<B, reachability::coord{4,20}, 0uz>(board);
        reachability::static_vector<Board, 4> ret{std::span<Board, B.SHAPES>(tmp.begin(), tmp.end())};
        return ret;
    });
}
