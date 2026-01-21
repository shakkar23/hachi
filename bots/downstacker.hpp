#pragma once

#include "bot_piece.hpp"
#include "bot_board.hpp"

#include "search.hpp"

#include <random>

Piece bot_downstacker(Board board, int meter, int combo, std::array<char, 5> queue);