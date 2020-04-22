from ctypes import *

class player_c:
    lib = CDLL("harness/libplayer.so")

    player_c = lib.player
    player_c.argtypes = [c_void_p, c_char_p, c_uint32]
    player_c.restype = c_void_p

    free_player_c = lib.free_player
    free_player_c.argtypes = [c_void_p]
    free_player_c.restype = None

    select_move_c = lib.select_move
    select_move_c.argtypes = [c_void_p, POINTER(POINTER(c_uint8)), c_size_t]
    select_move_c.restype = c_uint32

    switch_color_c = lib.switch_color
    switch_color_c.argtypes = [c_void_p]
    switch_color_c.restype = None

    set_color_c = lib.set_color
    set_color_c.argtypes = [c_void_p, c_uint32]
    set_color_c.restype = None

    @staticmethod
    def player(
            eval_model,
            select_move_type,
            color):
        select_move_type_arg = c_char_p(bytes(select_move_type, 'utf-8'))
        color_arg = c_uint32(color)

        return player_c.player_c(
            eval_model,
            select_move_type_arg,
            color_arg)

    @staticmethod
    def free_player(player):
        player_c.free_player_c(player)

    @staticmethod
    def select_move(
            player,
            positions):

        position_array_type = c_uint8 * len(positions[0])
        positions_array_type = len(positions) * POINTER(c_uint8)
        positions_arg_list = []

        for pos in positions:
            positions_arg_list.append(cast(position_array_type(*pos), POINTER(c_uint8)))

        positions_array = positions_array_type(*positions_arg_list)
        positions_arg = cast(positions_array, POINTER(POINTER(c_uint8)))
        positions_size_arg = c_size_t(len(positions))
        res = player_c.select_move_c(
            player,
            positions_arg,
            positions_size_arg)
        return int(res)

    @staticmethod
    def switch_color(
            player):
        player_c.switch_color_c(player)

    @staticmethod
    def set_color(
            player,
            color):
        player_c.set_color_c(player, c_uint32(color))

