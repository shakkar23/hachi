#include "feature_extractor.hpp"

#include <algorithm>

namespace moe {

BoardFeatures extract_features(const Board& board) {
    BoardFeatures features;
    // Board only exposes compile-time cell access; unpack its row-major chunks.
    const auto occupied = [&board](int x, int y) {
        const auto chunk = board.get_under(y / Board::lines_per_under);
        return ((chunk >> ((y % Board::lines_per_under) * Board::width + x)) & 1) != 0;
    };

    for (int x = 0; x < Board::width; ++x) {
        auto& height = features.heights[x];
        for (int y = Board::height - 1; y >= 0; --y) {
            if (occupied(x, y)) {
                height = y + 1;
                break;
            }
        }
        features.max_height = std::max(features.max_height, height);
        features.aggregate_height += height;
        if (height < features.heights[features.well_x]) {
            features.well_x = x;
        }
        bool found_hole = false;
        for (int y = height - 1; y >= 0; --y) {
            if (!occupied(x, y)) {
                if (!found_hole) {
                    features.first_hole_depths[x] = height - 1 - y;
                    found_hole = true;
                }
                ++features.holes;
                features.coveredness += height - y;
            }
        }
        if (x > 0) {
            features.height_differences[x - 1] = height - features.heights[x - 1];
        }
    }

    int previous_x = -1;
    for (int x = 0; x < Board::width; ++x) {
        if (x == features.well_x) continue;
        if (previous_x >= 0) {
            const int difference = features.heights[x] - features.heights[previous_x];
            features.bumpiness += difference * difference;
        }
        previous_x = x;
    }
    for (int y = 0; y < Board::height; ++y) {
        bool previous = true;
        bool filled_except_well = true;
        for (int x = 0; x < Board::width; ++x) {
            const bool cell = occupied(x, y);
            features.row_transitions += cell != previous;
            previous = cell;
            if (x != features.well_x) filled_except_well &= cell;
        }
        features.row_transitions += !previous;
        if (y >= features.heights[features.well_x] && filled_except_well) {
            ++features.well_depth;
        }
    }
    return features;
}

std::array<float, BoardFeatures::COUNT> BoardFeatures::values() const {
    std::array<float, COUNT> result{};
    std::size_t i = 0;
    for (int value : heights) result[i++] = static_cast<float>(value);
    for (int value : height_differences) result[i++] = static_cast<float>(value);
    for (int value : first_hole_depths) result[i++] = static_cast<float>(value);
    for (int value : {max_height, aggregate_height, holes, coveredness,
                      row_transitions, bumpiness, well_x, well_depth}) {
        result[i++] = static_cast<float>(value);
    }
    return result;
}

const std::array<std::string_view, BoardFeatures::COUNT>& BoardFeatures::names() {
    static_assert(Board::width == 10, "Update feature names if board width changes");
    static constexpr std::array<std::string_view, COUNT> names{
        "height_0", "height_1", "height_2", "height_3", "height_4",
        "height_5", "height_6", "height_7", "height_8", "height_9",
        "height_difference_0", "height_difference_1", "height_difference_2",
        "height_difference_3", "height_difference_4", "height_difference_5",
        "height_difference_6", "height_difference_7", "height_difference_8",
        "first_hole_depth_0", "first_hole_depth_1", "first_hole_depth_2",
        "first_hole_depth_3", "first_hole_depth_4", "first_hole_depth_5",
        "first_hole_depth_6", "first_hole_depth_7", "first_hole_depth_8",
        "first_hole_depth_9", "max_height", "aggregate_height", "holes",
        "coveredness", "row_transitions", "bumpiness", "well_x", "well_depth"
    };
    return names;
}

} // namespace moe
