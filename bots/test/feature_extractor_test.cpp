#include "feature_extractor.hpp"

#include <cstdlib>
#include <iostream>

static void check(bool condition, const char* description) {
    if (!condition) {
        std::cerr << "FAILED: " << description << '\n';
        std::exit(EXIT_FAILURE);
    }
}

int main() {
    const auto empty = moe::extract_features(Board{});
    check(empty.max_height == 0 && empty.holes == 0 && empty.coveredness == 0,
          "empty board has no stack or holes");
    check(empty.row_transitions == 2 * Board::height, "empty rows have two wall transitions");
    check(empty.well_x == 0 && empty.well_depth == 0, "well ties choose leftmost column");

    Board cavity{};
    cavity.set(0, 0);
    cavity.set(0, 2);
    cavity.set(0, 3);
    cavity.set(1, 0);
    const auto holes = moe::extract_features(cavity);
    check(holes.heights[0] == 4 && holes.heights[1] == 1, "heights measured from floor");
    check(holes.height_differences[0] == -3, "signed adjacent differences");
    check(holes.aggregate_height == 5 && holes.max_height == 4, "height summaries");
    check(holes.holes == 1 && holes.coveredness == 3, "hole count and burial depth");
    check(holes.first_hole_depths[0] == 2 && holes.first_hole_depths[1] == 0,
          "surface depth to first hole, zero for solid columns");
    check(holes.well_x == 2 && holes.bumpiness == 10, "Rust-style well and squared bumpiness");

    Board well{};
    for (int y = 0; y < 4; ++y) {
        for (int x = 0; x < Board::width; ++x) {
            if (x != 4) well.set(x, y);
        }
    }
    const auto trench = moe::extract_features(well);
    check(trench.well_x == 4 && trench.well_depth == 4 && trench.bumpiness == 0,
          "four-row well with flat shoulders");
    check(trench.holes == 0 && trench.row_transitions == 2 * Board::height,
          "open well is not a hole");

    Board full{};
    for (int y = 0; y < Board::height; ++y) {
        for (int x = 0; x < Board::width; ++x) full.set(x, y);
    }
    const auto filled = moe::extract_features(full);
    check(filled.max_height == Board::height && filled.aggregate_height == 320,
          "full-height board includes hidden rows and partial final chunk");
    check(filled.holes == 0 && filled.first_hole_depths[0] == 0 && filled.row_transitions == 0,
          "full board has no holes or transitions");

    Board ceiling{};
    ceiling.set(9, Board::height - 1);
    const auto overhang = moe::extract_features(ceiling);
    check(overhang.heights[9] == 32 && overhang.holes == 31 && overhang.coveredness == 527,
          "top-row overhang covers entire column");
    check(overhang.first_hole_depths[9] == 1, "first hole immediately under ceiling");

    const auto values = holes.values();
    const auto& names = moe::BoardFeatures::names();
    check(values.size() == 37 && values[0] == 4 && values[10] == -3 && values[19] == 2,
          "stable flattened column feature order");
    check(values[29] == 4 && values[31] == 1 && values[36] == 0,
          "stable flattened scalar feature order");
    for (auto name : names) check(!name.empty(), "every feature has a name");
    std::cout << "Board feature checks passed\n";
}
