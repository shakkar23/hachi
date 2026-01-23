#pragma once

#include "bot_piece.hpp"
#include "bot_board.hpp"

#include "search.hpp"

#include <random>
#include <span>

Piece bot_downstacker(Board board, std::span<char, 5> queue, char hold);
