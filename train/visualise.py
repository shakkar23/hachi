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
    511, 0, 63, 455, 504, 507, 255, 73, 219, 505, 447, 127, 510, 479, 508, 463, 
    487, 503, 319, 64, 1, 9, 8, 216, 11, 472, 509, 200, 223, 72, 31, 91, 475, 
    383, 47, 3, 217, 473, 27, 65, 95, 192, 15, 201, 495, 195, 75, 252, 448, 441, 
    203, 456, 315, 462, 126, 287, 231, 476, 251, 88, 483, 457, 89, 399, 59, 193, 
    497, 55, 199, 191, 128, 79, 384, 25, 449, 123, 67, 119, 488, 19, 7, 451, 2, 
    69, 248, 254, 321, 351, 379, 464, 459, 207, 208, 6, 506
]
print_3x3_patterns(numbers)