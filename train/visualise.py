def print_3x3_patterns(numbers):
    rank = 1
    for num in numbers:
        # Convert to 9-bit binary string
        binary = format(num, '09b')
        bits = [int(b) for b in binary]
        
        grid = [
            [bits[0], bits[3], bits[6]],  # row1
            [bits[1], bits[4], bits[7]],  # row2
            [bits[2], bits[5], bits[8]]   # row3
        ]
        
        print(f"\n#{rank}\n")
        rank += 1
        for row in grid:
            print(''.join(['▓' if cell else ' ' for cell in row]))
 
numbers = [
    511, 0, 455, 47, 488, 8, 505, 1, 507, 195,
    504, 479, 6, 321, 463, 63, 64, 128, 216, 219,
    73, 2, 510, 40, 503, 255, 508, 15, 72, 319,
    399, 7, 9, 31, 231, 441, 472, 447, 127, 192,
    11, 456, 475, 88, 487, 193, 3, 199, 203, 27,
    91, 19, 56, 59, 223, 383, 65, 79, 473, 478,
    55, 159, 217, 448, 287, 200, 24, 5, 379, 126,
    252, 71, 75, 251, 431, 449, 451, 67, 201, 23,
    39, 253, 496, 26, 89, 315, 415, 459, 60, 202,
    123, 155, 207, 13, 249, 28, 408, 409, 222, 245,
]
print_3x3_patterns(numbers)