#pragma once

#include "util/bot_piece.hpp"
#include "util/bot_board.hpp"

#include <span>
#include <cstdint>

Piece bot_downstacker(Board board, std::span<char, 5> queue, char hold, uint8_t bag, size_t beam_depth, size_t beam_width, size_t speculation_split_size);
