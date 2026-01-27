#pragma once

#include "util/bot_piece.hpp"
#include "util/bot_board.hpp"

#include <span>
#include <cstdint>

Piece bot_comboer(const Board board, const std::span<char, 5> queue, const char hold, const uint8_t bag, const size_t beam_depth, const size_t speculation_split_size);
