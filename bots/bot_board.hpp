#pragma once

#include <array>
#include <string>

#include "board.hpp"
#include "utils.hpp"

using Board = reachability::board_t<10,32>;

struct ColBoard {
    public:

    static inline ColBoard convert(Board board) {
        ColBoard ret{};
        reachability::static_for<Board::width>([&](auto y) {
            reachability::static_for<Board::width>([&](auto x) {
                if (board.get<x,y>())
                    ret.set(x,y);
            });
        });
        return ret;
    }

    inline bool get(size_t x, size_t y) const {
        return ((data[x] >> y) & 1) ? true : false;
    }
    inline void set(size_t x, size_t y) {
        data[x] |= 1 << y;
    }

    inline bool is_top_quarter() const {
        // do NOT touch anything this vectorizes
        constexpr uint32_t top_quarter_collider = ~((1 << 15) - 1);

        bool ret = false;

        for (size_t x = 0; x < Board::width; x++) {
            if (data[x] & top_quarter_collider)
                ret = true;
        }
        return ret;
    }

    inline bool is_top_half() const {
        // do NOT touch anything this vectorizes
        constexpr uint32_t top_half_collider = ~((1 << 10) - 1);

        bool ret = false;

        for (size_t x = 0; x < Board::width; x++) {
            if (data[x] & top_half_collider)
                ret = true;
        }
        return ret;
    }


    inline  bool is_low() const {
        constexpr uint32_t high_collider = ~((1 << 4) - 1);

        bool ret = true;

        for (size_t x = 0; x < Board::width; x++) {
            if (data[x] & high_collider)
                ret = false;
        }
        return ret;
    }

    inline  bool is_perfect_clear() const {

        bool ret = true;

        for (size_t x = 0; x < Board::width; x++) {
            if (data[x])
                ret = false;
        }
        return ret;
    }

    inline std::pair<int, int> cavities_overhangs() const {
        int cavities = 0;
        int overhangs = 0;

        for (int i = 0; i < Board::width; ++i) {
            auto col = data[i];
            col = ~col;
            col = col << std::countl_one(col);
            cavities += (data[i] != 0) * std::popcount(col);
        }

        for (int i = 1; i < Board::width; ++i) {
            auto col1 = data[i - 1];
            auto col2 = data[i];
            col1 = col1 >> (32 - std::countl_zero(col2));
            auto col3 = col1;
            col1 = ~col1;
            col1 = col1 << std::countl_one(col1);
            overhangs += (col3 != 0) * std::popcount(col1);
        }

        return { cavities, overhangs };
    }

    inline int well_position() const {

        int max_air = -1;
        int well_position = 0;

        for (int i = 0; i < Board::width; ++i) {
            auto& col = data[i];
            int air = std::countl_zero(col);
            if (air > max_air) {
                max_air = air;
                well_position = i;
            }
        }

        return well_position;
    }

    // lowest height, highest height
    inline std::pair<int, int> height_features() const {

        // air is the complement of height

        int max_air = -1;
        int min_air = 1 << 30;

        for (int i = 0; i < Board::width; ++i) {
            auto& col = data[i];
            int air = std::countl_zero(col);
            max_air = std::max(air, max_air);
            min_air = std::min(air, min_air);
        }

        return { 32 - max_air, 32 - min_air };
    }

    inline std::pair<int, int> n_covered_cells() const {
        int covered = 0;
        int covered_sq = 0;

        for (int i = 0; i < Board::width; ++i) {
            auto col = data[i];
            int col_covered = 0;
            int zeros = std::countl_zero(col);
            col <<= zeros; // 00011101 -> 11101
            int ones = std::countl_one(col);
            if (zeros + ones != 32) {
                col_covered += col_covered + ones;
            }
            covered += col_covered;
            covered_sq += col_covered * col_covered;
        }

        return { covered, covered_sq };
    }

    inline std::pair<int, int> get_bumpiness() const {
        int bumpiness = 0;
        int bumpiness_sq = 0;

        std::array<int, Board::width> air;

        for (int i = 0; i < Board::width; ++i) {
            air[i] = std::countl_zero(data[i]);
        }

        std::array<int, Board::width> bump;


        for (int i = 1; i < Board::width; ++i) {
            bump[i] = abs(air[i] - air[i - 1]);
        }

        for (int i = 1; i < Board::width; ++i) {
            bumpiness += bump[i];
            bumpiness_sq += bump[i] * bump[i];

        }
        return { bumpiness, bumpiness_sq };
    }

    // Identify clean count to 4
    inline bool ct4() const {

        int garbage_height = height_features().first;

        bool quad_ready = false;

        for (int i = 0; i < Board::width; ++i) {
            auto& col = data[i];
            quad_ready = quad_ready && ((((col >> garbage_height) | 0b1111) & 0b1111) == 0b1111);
        }

        if (!quad_ready) {
            return false;
        }

        if (garbage_height == 0) {
            return true;
        }
        bool ret = true;
        for (int i = 0; i < Board::width; ++i) {
            if (!get(i, garbage_height - 1)) {
                // check we have exactly 4 rows filled above this hole
                if (std::countl_zero(data[i]) != 28 - garbage_height) {
                    ret = true;
                }
            }
        }

        return true;
    }

    inline int get_row_transitions() const {
        int transitions = 0;
        for (int i = 0; i < Board::width - 1; i += 2) {
            auto& col1 = data[i];
            auto& col2 = data[i + 1];
            transitions += std::popcount(col1 ^ col2);
        }
        return transitions;
    }

    inline double CC_eval(int lines, bool tspin, bool waste_t) const {

        double score = 0.0;

        std::pair<int, int> values;

        if (is_top_half())
            score += top_half;

        if (is_top_quarter())
            score += top_quarter;

        if (is_low())
            score += low;

        if (is_perfect_clear())
            score += perfect_clear;


        values = cavities_overhangs();

        score += values.first * cavity_cells;

        score += values.first * values.first * cavity_cells_sq;

        score += values.second * overhangs;

        score += values.second * values.second * overhangs_sq;

        values = height_features();

        score += values.second * height;

        // if (ct4(board)) score += counting;
        

        // values = n_covered_cells(board);

        // score += values.first * covered_cells;
        // score += values.second * covered_cells_sq;

        values = get_bumpiness();

        score += values.first * bumpiness;
        score += values.second * bumpiness_sq;

        score += well_columns[well_position()];
        score += clears[lines];

        score += get_row_transitions() * row_transitions;

        if (tspin) {
            score += tspins[lines];
        }

        if (waste_t) {
            score += wasted_t;
        }

        return (score + 10000) / 10000;
    }


    inline double evaluate(int lines_cleared) const {
        return CC_eval(lines_cleared, false, false);
    }

    inline double evaluate_lines_cleared(int lines_cleared) const {
        return clears[lines_cleared] / 10'000;
    }

    private:
    std::array<uint32_t, Board::width> data;

        [[maybe_unused]] constexpr static auto top_half = -130.0;
        [[maybe_unused]] constexpr static auto top_quarter = -499.0;
        [[maybe_unused]] constexpr static auto low = -50.0;
        [[maybe_unused]] constexpr static auto cavity_cells = -176.0;
        [[maybe_unused]] constexpr static auto cavity_cells_sq = -6.0;
        [[maybe_unused]] constexpr static auto overhangs = -47.0;
        [[maybe_unused]] constexpr static auto overhangs_sq = -9.0;
        [[maybe_unused]] constexpr static auto covered_cells = -26.0;
        [[maybe_unused]] constexpr static auto covered_cells_sq = 1.0;
        [[maybe_unused]] constexpr static auto bumpiness = -7.0;
        [[maybe_unused]] constexpr static auto bumpiness_sq = -28.0;
        [[maybe_unused]] constexpr static auto height = -46.0;
        [[maybe_unused]] constexpr static float well_columns[10] = { 31, 16, -41, 37, 49, 30, 56, 48, -27, 22 };
        [[maybe_unused]] constexpr static float clears[5] = { 0, -1700, -100, -50, 490 };
        [[maybe_unused]] constexpr static float tspins[4] = { 0, 126, 434, 220 };
        [[maybe_unused]] constexpr static float perfect_clear = 200.0;
        [[maybe_unused]] constexpr static float wasted_t = -52.0;
        [[maybe_unused]] constexpr static float tsd_shape = 180.0;
        [[maybe_unused]] constexpr static float well_depth = 91.0; // todo
        [[maybe_unused]] constexpr static float max_well_depth = 17.0; // todo
        [[maybe_unused]] constexpr static float row_transitions = -5.0;
        [[maybe_unused]] constexpr static float donuts = -10.0;
        [[maybe_unused]] constexpr static float v_shape = 50.0;
        [[maybe_unused]] constexpr static float s_shape = 80.0;
        [[maybe_unused]] constexpr static float l_shape = 80.0;
        [[maybe_unused]] constexpr static float l2_shape = 80.0;
        [[maybe_unused]] constexpr static float counting = 50.0;
};