from ctypes import *

class model_c:
    lib = CDLL("harness/libmodel.so")

    model_c = lib.model
    model_c.argtypes = [c_char_p]
    model_c.restype = c_void_p

    new_model_c = lib.new_model
    new_model_c.argtypes = [c_void_p, POINTER(c_char_p)]
    new_model_c.restype = None

    free_model_c = lib.free_model
    free_model_c.argtypes = [c_void_p]
    free_model_c.restype = None

    load_model_c = lib.load_model
    load_model_c.argtypes = [c_char_p, c_uint32, c_char_p, POINTER(c_char_p)]
    load_model_c.restype = c_void_p

    store_model_c = lib.store_model
    store_model_c.argtypes = [c_void_p, c_char_p, c_uint32, c_char_p]
    store_model_c.restype = None

    train_model_c = lib.train_model
    train_model_c.argtypes = [c_void_p, c_char_p]
    train_model_c.restype = None

    @staticmethod
    def load_model(
            model_type,
            model_version,
            model_directory,
            args):
        model_type_arg = c_char_p(bytes(model_type, 'utf-8'))
        model_version_arg = c_uint32(model_version)
        model_directory_arg = c_char_p(bytes(model_directory, 'utf-8'))

        args_ptr_list = []

        for arg in args:
            args_ptr_list.append(c_char_p(bytes(str(arg), 'utf-8')))

        args_array_type = c_char_p * len(args)
        args_array_arg = args_array_type(*args_ptr_list)
        res = model_c.load_model_c(
            model_type_arg,
            model_version_arg,
            model_directory_arg,
            args_array_arg) # or cast(args_array_arg, POINTER(c_char_p))

        return res

    @staticmethod
    def free_model(model):
        model_c.free_model_c(model)

    @staticmethod
    def train_model(
            model,
            games_directory):
        games_directory_arg = c_char_p(bytes(games_directory, 'utf-8'))
        model_c.train_model_c(model, games_directory_arg)

    @staticmethod
    def store_model(
            model,
            model_type,
            model_version,
            model_directory):
        model_type_arg = c_char_p(bytes(model_type, 'utf-8'))
        model_version_arg = c_uint32(model_version)
        model_directory_arg = c_char_p(bytes(model_directory, 'utf-8'))
        model_c.store_model_c(model, model_type_arg, model_version_arg, model_directory_arg)








