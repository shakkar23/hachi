#include "spinner.hpp"

#include "block.hpp"

#include "util/movegen.hpp"
#include "util/rng.hpp"

// dummy bot, unimplemented
Piece bot_spinner(const Board board, const std::span<char, 5> queue, const char hold, const uint8_t bag, const size_t beam_depth, const size_t speculation_split_size) {
    return Piece{' '};
}
