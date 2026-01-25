#pragma once

#include <random>

struct RNG {
public:
    RNG(const uint8_t bag_mask) {
        PPTRNG = std::random_device()();

        bag = std::array{
            'S',
            'Z',
            'J',
            'L',
            'T',
            'O',
            'I' };

        uint32_t buffer = 6;
        
        for(int i = 0; i < 7; i++) {
            if(bag_mask & (1 << i)) continue;

            std::swap(bag[buffer], bag[i]);
            buffer--;
        }

        for(int i = buffer; i >= 0; i--) {
            std::swap(bag[getRand(i + 1)], bag[i]);
        }
        bagiterator = 6 - buffer;
    }
    RNG() {
        PPTRNG = std::random_device()();

        makebag();
    }
    uint32_t PPTRNG{};
    std::array<char, 7> bag;
    uint8_t bagiterator;

    inline char getPiece() {
        if (bagiterator == 7) {
            makebag();
        }
        return bag[bagiterator++];
    }

    inline uint32_t getRand(uint32_t upperBound) {
        PPTRNG = PPTRNG * 0x5d588b65 + 0x269ec3;
        uint32_t uVar1 = PPTRNG >> 0x10;
        if (upperBound != 0) {
            uVar1 = uVar1 * upperBound >> 0x10;
        }
        return uVar1;
    }

    inline void makebag() {
        bagiterator = 0;
        uint32_t buffer = 0;

        std::array<char, 7> pieces = {
            'S',
            'Z',
            'J',
            'L',
            'T',
            'O',
            'I'};

        for (int_fast8_t i = 6; i >= 0; i--) {
            buffer = getRand(i + 1);
            bag[i] = pieces[buffer];
            std::swap(pieces[buffer], pieces[i]);
        }
    }

    inline void new_seed() {
        PPTRNG = std::random_device()();
    }
};