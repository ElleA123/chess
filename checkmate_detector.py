"""
Not checkmate: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
Checkmate: 2kr1bnr/p2nqppp/B1p1b3/8/3P1B2/2N5/PPP2PPP/R3K1NR b KQ - 1 9
"""

from copy import deepcopy

def parse_fen_row(fen_row):
    """Gets a row from the row in FEN."""
    numbers = ["1", "2", "3", "4", "5", "6", "7", "8"]
    row_list = []
    for character in fen_row:
        if character not in numbers:
            row_list.append(character)
        else:
            number_of_spaces = int(character)
            for i in range(number_of_spaces):
                row_list.append("0")
    return row_list
    
def change_square_to_code(square):
    """Moves from algebraic notation to the indexing used in the code."""
    FILE_SWITCH = {"a": 0, "b": 1, "c": 2, "d": 3, "e": 4, "f": 5, "g": 6, "h": 7}
    RANK_SWITCH = {"1": 7, "2": 6, "3": 5, "4": 4, "5": 3, "6": 2, "7": 1, "8": 0}
    file_number = FILE_SWITCH[square[0]]
    rank_number = RANK_SWITCH[square[1]]
    code_square = (rank_number, file_number)
    return code_square

def change_code_to_square(code_square):
    """Moves from the indexing used in the code to algebraic notation."""
    FILE_SWITCH = {0: "a", 1: "b", 2: "c", 3: "d", 4: "e", 5: "f", 6: "g", 7: "h"}
    RANK_SWITCH = {0: "8", 1: "7", 2: "6", 3: "5", 4: "4", 5: "3", 6: "2", 7: "1"}
    file_number = FILE_SWITCH[square[1]]

def get_board_state(fen):
    """Creates a dict about the position from the FEN position."""
    split_fen = fen.split()
    board_state = {"board": [],
                   "to_move": None,
                   "who_can_castle": None,
                   "en_passant": None,
                   "halfmove_count": None,
                   "move_number": None}

    # Board
    fen_board_rows = split_fen[0].split("/")
    for row in range(8):
        row_list = parse_fen_row(fen_board_rows[row])
        board_state["board"].append(row_list)

    # To move
    board_state["to_move"] = split_fen[1]

    # Who can castle
    board_state["who_can_castle"] = []
    if split_fen[2] != "-":
        for character in split_fen[2]:
            board_state["who_can_castle"].append(character)

    # En passant
    if split_fen[3] == "-":
        board_state["en_passant"] = False
    else:
        board_state["en_passant"] = change_square_to_code(split_fen[3])
        
        

    # Halfmove count
    if len(split_fen) >= 5:
        board_state["halfmove_count"] = int(split_fen[4])
    else:
        board_state["halfmove_count"] = 0

    # Move number
    if len(split_fen) >= 6:
        board_state["move_number"] = int(split_fen[5])
    else:
        board_state["move_number"] = 0
    
    return board_state

def change_turn(board_state):
    """ Changes whose turn it is in board_state."""
    flipped_board_state = deepcopy(board_state)
    if flipped_board_state["to_move"] == "w":
        flipped_board_state["to_move"] = "b"
    else:
        flipped_board_state["to_move"] = "w"
    return flipped_board_state

def print_board(board):
    """Prints the board as a grid."""
    for row in board:
        print (" ".join(row))

def create_blank_board():
    """Creates an empty board."""
    return [["0" for c in range(8)] for r in range(8)]

def check_for_check(board_state, color):
    """Checks if the color given is in check."""
    if color == "w":
        target = "K"
    else:
        target = "k"

    search_board_state = deepcopy(board_state)
    search_board_state["to_move"] = color
    current_moves = find_all_legal_moves(change_turn(search_board_state),
                                         False,
                                         False)
    for move in current_moves:
        (current_row, current_column) = move["square"]
        if board_state["board"][current_row][current_column] == target:
            return True
    return False

def get_other_color(color):
    """gets the opposite color. Duh."""
    if color == "w":
        return "b"
    else:
        return "w"

def move_piece(board_state, move):
    """Makes a move using a dict about the move."""
    new_board_state = deepcopy(board_state)
    new_board = new_board_state["board"]
    (old_row, old_column) = move["original_square"]
    (new_row, new_column) = move["square"]
    
    piece_moved = str(new_board[old_row][old_column])
    new_board[old_row][old_column] = "0"
    new_board[new_row][new_column] = piece_moved
    
    if "captured_square" in move:
        (captured_row, captured_column) = captured_square
        new_board[captured_row][captured_column] = "0"
    
    if "other_move" in move: # Used for castling.
        ((other_old_row, other_old_column),
         (other_new_row, other_new_column)) = move["other_move"]
        other_piece_moved = str(new_board[other_old_row][other_old_column])
        new_board[other_old_row][other_old_column] = "0"
        new_board[other_new_row][other_new_column] = other_piece_moved

    castles_to_remove = [] # Only one castle per game.
    if piece_moved == "K":
        castles_to_remove.append("K")
        castles_to_remove.append("Q")
    if piece_moved == "k":
        castles_to_remove.append("k")
        castles_to_remove.append("q")
    if move["original_square"] == (7, 7) or move["square"] == (7, 7):
        castles_to_remove.append("K")
    if move["original_square"] == (7, 0) or move["square"] == (7, 0):
        castles_to_remove.append("Q")
    if move["original_square"] == (0, 7) or move["square"] == (0, 7):
        castles_to_remove.append("k")
    if move["original_square"] == (0, 0) or move["square"] == (0, 0):
        castles_to_remove.append("q")

    for castle in castles_to_remove:
        if castle in new_board_state["who_can_castle"]:
            new_board_state["who_can_castle"].remove(castle)

    if "new_en_passant" in move:
        new_board_state["en_passant"] = move["new_en_passant"]
    else:
        new_board_state["en_passant"] = False
    
    new_board_state["board"] = new_board
    return change_turn(new_board_state)

def find_players_pieces(board_state):
    """Gets a list of piece values for the player."""
    if board_state["to_move"] == "w":
        pieces = ["R", "N", "B", "K", "Q", "P"]
    else:
        pieces = ["r", "n", "b", "k", "q", "p"]
    
    player_pieces = []
    for row in range(8):
        for column in range(8):
            if board_state["board"][row][column] in pieces:
                player_pieces.append((row, column))

    return player_pieces

def find_all_legal_moves(board_state, check_not_attacked, check_check):
    player_pieces = find_players_pieces(board_state)
    """Finds all legal moves on the board."""
    all_legal_moves = []
    for (row, column) in player_pieces:
        current_legal_moves = find_legal_moves_for_piece(
            row,
            column,
            board_state,
            check_not_attacked)
        all_legal_moves.extend(current_legal_moves)
    if check_check == True:
        non_checked_moves = []
        for move in all_legal_moves:
            new_board_state = move_piece(board_state, move)
            check_color = get_other_color(new_board_state["to_move"])
            if check_for_check(new_board_state, check_color) == False:  
                non_checked_moves.append(move)          
        return non_checked_moves
    return all_legal_moves
                                                         
def find_moves_with_piece_rules(board_state, row, column, piece):
    """Gives all moves for a certain piece in a certain square."""
    possible_moves = []
    if piece == "r" or piece == "R":
        return find_moves_for_rook(row, column)
    elif piece == "b" or piece == "B":
        return find_moves_for_bishop(row, column)
    elif piece == "q" or piece == "Q":
        return find_moves_for_queen(row, column)
    elif piece == "k" or piece == "K":
        return find_moves_for_king(row, column, piece, board_state["who_can_castle"])
    elif piece == "n" or piece == "N":
        return find_moves_for_knight(row, column)
    else:
        return find_moves_for_pawn(row, column, piece, board_state["en_passant"])

def find_move_step(row, column, row_delta, column_change, max_steps):
    """Finds movable squares for pieces that can move different lengths (B, R, and Q)."""
    possible_moves = []
    for step_num in range(1, max_steps+1):
        
        current_square = (row + (step_num * row_delta), column + (step_num * column_change))
        (current_row, current_column) = current_square
        if current_row > 7 or current_row < 0 or current_column > 7 or current_column < 0:
            break
        
        current_empty_squares = []
        for empty_square_step in range(1, step_num):
            current_empty_square = (row + (empty_square_step * row_delta), column + (empty_square_step * column_change))
            current_empty_squares.append(current_empty_square)
        
        possible_moves.append({"original_square": (row, column),
                               "square": current_square,
                               "empty": current_empty_squares,
                               "enemy": [],
                               "empty_or_enemy": [current_square]})
    return possible_moves

def find_moves_for_steps(row, column, max_step, step_list):
    """Uses find_move_step to get a list of all movable squares."""
    possible_moves = []

    for step in step_list:
        (row_delta, column_change) = step
        current_move = find_move_step(row, column, row_delta, column_change, max_step)
        possible_moves.extend(current_move)
    return possible_moves

def find_moves_for_rook(row, column):
    """Finds moves for a rook."""
    return find_moves_for_steps(row, column, 8, [(1, 0), (-1, 0), (0, 1), (0, -1)])

def find_moves_for_bishop(row, column):
    """Finds moves for a bishop."""
    return find_moves_for_steps(row, column, 8, [(1, 1), (1, -1), (-1, 1), (-1, -1)])

def find_moves_for_queen(row, column):
    """Finds moves for a queen."""
    return find_moves_for_bishop(row, column) + find_moves_for_rook(row, column)

def find_moves_for_king(row, column, piece, castling):
    """Finds moves for a king, including castles."""
    step_list = []
    
    for row_delta in range(-1, 2):
        for column_change in range(-1, 2):
            if row_delta == 0 and column_change == 0:
                continue
            step_list.append((row_delta, column_change))
    
    possible_moves = find_moves_for_steps(row, column, 1, step_list)

    if piece == "K":
        if "K" in castling:
            possible_moves.append({"original_square": (7, 4),
                                   "square": (7, 6),
                                   "other_move": ((7, 7), (7, 5)),
                                   "empty": [(7, 5), (7, 6)],
                                   "enemy": [],
                                   "empty_or_enemy": [],
                                   "not_attacked": [(7, 4), (7, 5), (7, 6)]})
        if "Q" in castling:
            possible_moves.append({"original_square": (7, 4),
                                   "square": (7, 2),
                                   "other_move": ((7, 0), (7, 3)),
                                   "empty": [(7, 2), (7, 3)],
                                   "enemy": [],
                                   "empty_or_enemy": [],
                                   "not_attacked": [(7, 2), (7, 3), (7, 4)]})
    elif piece == "k":
        if "k" in castling:
            possible_moves.append({"original_square": (0, 4),
                                   "square": (0, 6),
                                   "other_move": ((0, 7), (0, 5)),
                                   "empty": [(0, 5), (0, 6)],
                                   "enemy": [],
                                   "empty_or_enemy": [],
                                   "not_attacked": [(0, 4), (0, 5), (0, 6)]})
        if "q" in castling:
            possible_moves.append({"original_square": (0, 4),
                                   "square": (0, 2),
                                   "other_move": ((0, 0), (0, 3)),
                                   "empty": [(0, 2), (0, 3)],
                                   "enemy": [],
                                   "empty_or_enemy": [],
                                   "not_attacked": [(0, 2), (0, 3), (0, 4)]})

    return possible_moves
    

def find_moves_for_knight(row, column):
    """Finds moves for a knight."""
    step_list = [(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)]
    return find_moves_for_steps(row, column, 1, step_list)

def find_moves_for_pawn(row, column, color, en_passant):
    """Finds moves for a pawn, including en passant. Promotion isn't implemented."""
    possible_moves = []
    if color == "P":
        row_change = -1
    else:
        row_change = 1
    current_square = (row + row_change, column)
    possible_moves.append({"original_square": (row, column),
                           "square": current_square,
                           "empty": [current_square],
                           "enemy": [],
                           "empty_or_enemy": []})
    
    if column != 7:
        current_square = (row + row_change, column+1)
        possible_moves.append({"original_square": (row, column),
                               "square": current_square,
                               "empty": [],
                               "enemy": [current_square],
                               "empty_or_enemy": []})

    if column != 0:
        current_square = (row + row_change, column-1)
        possible_moves.append({"original_square": (row, column),
                               "square": current_square,
                               "empty": [],
                               "enemy": [current_square],
                               "empty_or_enemy": []})

    if (color == "P" and row == 6) or (color == "p" and row == 1):
        current_square = (row + (row_change*2), column)
        possible_moves.append({"original_square": (row, column),
                               "square": current_square,
                               "empty": [current_square, (row + row_change, column)],
                               "enemy": [],
                               "empty_or_enemy": [],
                               "new_en_passant": (row + row_change, column)})
    
    for column_change in [-1, 1]:
        if (row + row_change, column + column_change) == en_passant:
            possible_moves.append({"original_square": (row, column),
                                   "square": en_passant,
                                   "empty": [],
                                   "enemy": [((en_passant[0] - row_change), en_passant[1])],
                                   "empty_or_enemy": [],
                                   "captured_square": (row + row_change, column)})
        

    return possible_moves

def find_legal_moves_for_piece(row, column, board_state, check_not_attacked):
    """Filters all moves for legal ones based on the board."""
    all_possible_moves = find_moves_with_piece_rules(
        board_state,
        row,
        column,
        board_state["board"][row][column])
    legal_moves = []
    for move in all_possible_moves:
        if is_move_legal(move,
                         board_state,
                         board_state["board"][row][column],
                         check_not_attacked):
            legal_moves.append(move)
    return legal_moves

def is_move_legal(move, board_state, piece, check_not_attacked):
    """Determines if a move is legal."""
    board = board_state["board"]
    if piece in ["r", "n", "b", "q", "k", "p"]:
        enemies = ["R", "N", "B", "Q", "K", "P"]
    else:
        enemies = ["r", "n", "b", "q", "k", "p"]
    
    for square in move["empty"]:
        (r, c) = square
        if board[r][c] != "0":
            return False
        
    for square in move["enemy"]:
        (r, c) = square
        if board[r][c] not in enemies:
            return False

    for square in move["empty_or_enemy"]:
        (r, c) = square
        if (board[r][c] not in enemies) and board[r][c] != "0":
            return False

    if check_not_attacked == True and "not_attacked" in move:
        flipped_board_state = change_turn(board_state)
        opponent_moves = find_all_legal_moves(flipped_board_state, False, False)
        opponent_squares = [move["square"] for move in opponent_moves]
        for square in move["not_attacked"]:
            if square in opponent_squares:
                return False

    return True

def is_checkmate(board_state):
    """Checks if a position is checkmate."""
    all_moves = find_all_legal_moves(board_state, True, True)
    checked = check_for_check(board_state, board_state["to_move"])
    if all_moves == [] and checked == True:
        return True
    return False

def look_for_checkmate(board_state):
    """Still working on this."""
    if is_checkmate(board_state) == True:
        return (True, 0, [])
    
    all_legal_moves = find_all_legal_moves(board_state, True, True)
    for move in all_legal_moves:
        new_board_state = move_piece(board_state, move)
        if is_checkmate(new_board_state) == True:
            return (True, 1, [move])
    return (False, -1, [])

def main():
    fen = input("Put in the FEN for your board: ")
    board_state = get_board_state(fen)
    print_board(board_state["board"])
    checkmate = is_checkmate(board_state)
    
    if checkmate == False:
        print ("This position isn't checkmate.")
    else:
        print ("This position is checkmate.")

if __name__ == "__main__":
    main()
