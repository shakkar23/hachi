#pragma once

#include "util/bot_board.hpp"

#include <array>
#include <cstddef>
#include <string_view>

namespace moe {

// Board-only inputs: no expert outputs or search scores. Coordinates use y=0
// at the floor, and include all Board::height rows (including hidden rows).
struct BoardFeatures {
    std::array<int, Board::width> heights{};
    std::array<int, Board::width - 1> height_differences{}; // right minus left
    // Occupied cells from the surface down to the first hole; zero if no hole.
    std::array<int, Board::width> first_hole_depths{};
    int max_height{};
    int aggregate_height{};
    int holes{}; // Empty cells below the highest occupied cell in each column.
    int coveredness{}; // Sum of (column height - hole y) over all holes.
    int row_transitions{}; // Across all rows, treating both side walls as filled.
    int bumpiness{}; // Rust sunbeam definition: squared differences skipping well.
    int well_x{}; // Leftmost shortest column.
    int well_depth{}; // Rows above well height filled in every other column.

    static constexpr std::size_t COUNT = 3 * Board::width - 1 + 8;

    // Raw, unnormalized values. This order is the matrix's feature dimension;
    // names() uses exactly the same order. Existing indices stay fixed when
    // adding features by appending them.
    [[nodiscard]] std::array<float, COUNT> values() const;
    [[nodiscard]] static const std::array<std::string_view, COUNT>& names();
};

[[nodiscard]] BoardFeatures extract_features(const Board& board);

} // namespace moe
