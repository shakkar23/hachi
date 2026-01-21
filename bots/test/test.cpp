#include "../downstacker.hpp"

#include <print>

int main() {
    auto piece = bot_downstacker(Board(), 0,0, {'T','S','Z','I','O'});
    std::println("{}, {}, {}, {}", piece.x, piece.y, piece.rot, piece.t);
    return 0;
}