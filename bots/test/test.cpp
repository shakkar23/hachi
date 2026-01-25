#include "../downstacker.hpp"
#include "util/rng.hpp"

#include <print>
#include <ranges>
#include <span>
#include <numeric>
#include <cmath>
#include <cassert>
#include <algorithm>
#include <chrono>
#include <charconv>
#include <thread>

struct Stats {
    double average;
    double stdDev;
    int min;
    int max;
};

int type_to_int(char type) {
    switch(type) {
        case 'S':
            return 0;
        case 'Z':
            return 1;
        case 'L':
            return 2;
        case 'J':
            return 3;
        case 'T':
            return 4;
        case 'O':
            return 5;
        case 'I':
            return 6;
        case ' ':
            return 7;
    }
    std::unreachable();
}

Stats calculate_stats(const std::vector<int>& data) {
    if (data.empty()) {
        return {0.0, 0.0, 0, 0};
    }

    Stats result;

    // 1. Min and Max
    // minmax_element returns a pair of iterators to the smallest and largest values
    auto [minIt, maxIt] = std::minmax_element(data.begin(), data.end());
    result.min = *minIt;
    result.max = *maxIt;

    // 2. Average (Mean)
    double sum = std::accumulate(data.begin(), data.end(), 0.0);
    result.average = sum / data.size();

    // 3. Standard Deviation
    // Formula: sqrt( sum( (x - mean)^2 ) / N )
    double varianceSum = 0;
    for (int x : data) {
        varianceSum += std::pow(x - result.average, 2);
    }
    result.stdDev = std::sqrt(varianceSum / data.size());

    return result;
}

void benchmark_downstacker(int num_sims, float piece_per_garbage, int max_sim_length) {
    RNG rng;
    std::vector<int> length_of_sims;
    length_of_sims.reserve(num_sims);
    
    std::vector<char> queue;
    queue.resize(max_sim_length + 5 + 1, ' ');
    std::chrono::duration<double, std::ratio<1, 1000>> total_time{};

    for(const auto _ : std::views::iota(0,num_sims)) {
        Board board{};
        char hold = ' ';
        uint8_t bag = 0b1111111;

        rng.makebag();
        for(auto& piece : queue) {
            piece = rng.getPiece();
        }
        for(int i = 0; i < 5; ++i) {
            bag &= ~(1 << type_to_int(queue[i]));

            if(bag == 0) {
                bag = 0b1111111;
            }
        }
        
        int sim_length = 0;
        float garbage_acc = 0.0f;
        int current_piece_index = 0;

        while(true) {
            if(sim_length >= max_sim_length)
                break;
            sim_length++;
            garbage_acc += 1.0f;

            assert(std::distance(queue.begin() + current_piece_index, queue.end()) > 0);
            
            auto queue_ptr = queue.begin() + current_piece_index;
            
            auto now = std::chrono::high_resolution_clock::now();
            auto piece = bot_downstacker(board, std::span<char,5>(queue_ptr, queue_ptr + 5), hold, bag, 10, 1000);
            auto after = std::chrono::high_resolution_clock::now();
            total_time = total_time + after - now;

            // dead or gave up
            if(piece.t == ' ') {
                break;
            }
            
            reachability::blocks::call_with_block<reachability::blocks::SRS>(piece.t, [&]<reachability::block B>() {
                reachability::static_for<B.BLOCK_PER_MINO>([&](const std::size_t mino_i) {
                    int px = piece.x + B.minos[piece.rot][mino_i][0];
                    int py = piece.y + B.minos[piece.rot][mino_i][1];
                    board.set(px, py);
                });
            });
            board.clear_full_lines();
            
            bool held = piece.t != queue[sim_length - 1];
            bool first_hold = hold == ' ' && held;
            
            if(held and !first_hold) {
                hold = piece.t;
            } else if(first_hold) {
                hold = piece.t;

                bag &= ~(1 << type_to_int(queue[current_piece_index + 5]));

                if(bag == 0) {
                    bag = 0b1111111;
                }
                current_piece_index++;
            }

            bag &= ~(1 << type_to_int(queue[current_piece_index+5]));

            if(bag == 0) {
                bag = 0b1111111;
            }
            
            current_piece_index++;
            
            while(garbage_acc > piece_per_garbage) {
                board = board.move < reachability::coord{ 0,1 } > ();

                Board single_garbage{};
                auto x = rng.getRand(10);

                for(int i = 0; i < 10; ++i) {
                    if(i != x) {
                        single_garbage.set(i, 0);
                    }
                }

                board |= single_garbage;

                garbage_acc -= piece_per_garbage;
            }
            //std::println("{}", to_string(board));
            //std::this_thread::sleep_for(std::chrono::milliseconds(250));
        }
        // std::println("finished {}: {}", _, sim_length);
        length_of_sims.push_back(sim_length);
        sim_length = 0;
    }
    auto stats = calculate_stats(length_of_sims);
    std::println("average = {}", stats.average);
    std::println("stdDev = {}", stats.stdDev);
    std::println("min = {}", stats.min);
    std::println("max = {}", stats.max);
    std::println("total = {}", std::accumulate(length_of_sims.begin(), length_of_sims.end(), 0));
    std::println("time = {}", total_time);
}

std::array<const char*,4> backup_params = {NULL,"20","1.5","800"};

int main(int argc, const char** argv) {
    if(argc < 4) {
        std::println("usage: program num_sims piece_per_garbage max_sim_length");
        std::println("doing test run of {} {} {}", backup_params[1], backup_params[2], backup_params[3]);
        argc = 4;
        argv = (const char**)backup_params.data();
        //return 1;
    }
    
    std::string num_sims_str = argv[1];
    int num_sims = 0;
    std::string piece_per_garbage_str = argv[2];
    float piece_per_garbage = 0;
    std::string max_sim_length_str = argv[3];
    int max_sim_length = 0;

    auto result = std::from_chars(num_sims_str.data(), num_sims_str.data() + num_sims_str.size(), num_sims);
    if (result.ec == std::errc())
        ;
    else if (result.ec == std::errc::invalid_argument) {
        std::print("num_sims is not a number.\n");
        return 1;
    }
    else if (result.ec == std::errc::result_out_of_range) {
        std::print("num_sims number is larger than an int.\n");
        return 1;
    }

    result = std::from_chars(piece_per_garbage_str.data(), piece_per_garbage_str.data() + piece_per_garbage_str.size(), piece_per_garbage);
    if (result.ec == std::errc())
        ;
    else if (result.ec == std::errc::invalid_argument) {
        std::print("piece_per_garbage is not a number.\n");
        return 1;
    }
    else if (result.ec == std::errc::result_out_of_range) {
        std::print("piece_per_garbage number is larger than an int.\n");
        return 1;
    }
    result = std::from_chars(max_sim_length_str.data(), max_sim_length_str.data() + max_sim_length_str.size(), max_sim_length);
    if (result.ec == std::errc())
        ;
    else if (result.ec == std::errc::invalid_argument) {
        std::print("max_sim_length is not a number.\n");
        return 1;
    }
    else if (result.ec == std::errc::result_out_of_range) {
        std::print("max_sim_length number is larger than an int.\n");
        return 1;
    }
    
    benchmark_downstacker(num_sims, piece_per_garbage, max_sim_length);
    return 0;
}
int l() {
    std::array queue = {'S','Z','T','L','O'};
    auto now = std::chrono::high_resolution_clock::now();
    for(int i =0; i < 10'000; i++)
        bot_downstacker(Board{}, std::span<char,5>{queue}, ' ', 0b1111111, 5,300);
    auto after = std::chrono::high_resolution_clock::now();
    std::println("{}", std::chrono::duration<double, std::ratio<1,1000>>(after - now));
    return 0;
}