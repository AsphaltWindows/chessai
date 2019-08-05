from ctypes import *

khlib = CDLL("models/libkhist.so")

khist_create = khlib.create_khist
khist_create.restype = c_void_p

khist_train_full = khlib.train_full
khist_train_full.restype = None

khist_train_incremental = khlib.train_incremental
khist_train_incremental.restype = None

khist_assign_cluster = khlib.assign_cluster
khist_assign_cluster.restype = POINTER(c_double * 2)

khist_model_from_vals = khlib.khist_model_from_vals

khist_model_to_vals = khlib.khist_model_to_vals
khist_model_to_vals.argtypes = [c_void_p]

khist_free = khlib.free_khist
khist_free.argtypes = [c_void_p]
khist_free.restype = None

free = khlib.free
free.argtypes = [c_void_p]
free.restype = None


class KH_C:

    def __init__(self, start_modes, cluster_num, categories):
        self.cat_num = len(categories)
        self.cluster_num = cluster_num
        self.categories = categories
        CatArray = c_uint8 * self.cat_num
        categories_param = CatArray(*categories)
        ModesArray = POINTER(c_uint8 * self.cat_num) * len(start_modes)
        mode_list = []
        mode_pointer_type = POINTER(c_uint8 * self.cat_num)
        for m in start_modes:
            ModeRow = c_uint8 * self.cat_num
            mode_param = mode_pointer_type(ModeRow(*m))
            mode_list.append(mode_param)
        modes_param = ModesArray(*mode_list)
        khist_create.argtypes = [POINTER(CatArray), c_size_t, c_uint8, ModesArray, c_size_t]
        self.khist = khist_create(categories_param, self.cat_num, cluster_num, modes_param, len(start_modes))

    def train_full(self, data):
        DataArray = POINTER(c_uint8 * self.cat_num) * len(data)
        data_list = []
        data_row_pointer_type = POINTER(c_uint8 * self.cat_num)
        for d in data:
            DataRow = c_uint8 * self.cat_num
            row_param = data_row_pointer_type(DataRow(*d))
            data_list.append(row_param)
        data_param = DataArray(*data_list)
        khist_train_full.argtypes = [c_void_p, DataArray, c_size_t]
        khist_train_full(self.khist, data_param, len(data))

    def train_incremental(self, data):
        DataArray = POINTER(c_uint8 * self.cat_num) * len(data)
        data_list = []
        data_row_pointer_type = POINTER(c_uint8 * self.cat_num)
        for d in data:
            DataRow = c_uint8 * self.cat_num
            row_param = data_row_pointer_type(DataRow(*d))
            data_list.append(row_param)
        data_param = DataArray(*data_list)
        khist_train_incremental.argtypes = [c_void_p, DataArray, c_size_t]
        khist_train_incremental(self.khist, data_param, len(data))

    def assign_cluster(self, row):
        RowArray = c_uint8 * self.cat_num
        row_param = RowArray(*row)
        khist_assign_cluster.argtypes = [c_void_p, POINTER(c_uint8 * len(row))]
        c_res = khist_assign_cluster(self.khist, row_param, len(row))
        res = (int(c_res.contents[0]), c_res.contents[1])
        free(c_res)
        return res

    def model_val_num(self):
        return 2 + self.cat_num + self.cluster_num + sum(self.categories) * self.cluster_num

    def model_to_vals(self):
        khist_model_to_vals.restype = POINTER(c_uint32 * self.model_val_num())
        c_res = khist_model_to_vals(self.khist)
        res = [r for r in c_res.contents]
        free(c_res)
        return res

    def store_model2(self, file_name):
        file = open(file_name, "w")
        file.write("\n".join(map(str, self.model_to_vals())))
        file.close()

    def free_khist(self):
        khist_free(self.khist)

    @staticmethod
    def model_from_vals(model_vals):
        cat_num = model_vals[0]
        cluster_num = model_vals[1]
        categories = model_vals[2:2+cat_num]
        model = KH_C([], cluster_num, categories)
        khist_free(model.khist)
        ValsArray = c_uint32 * len(model_vals)
        vals_param = ValsArray(*model_vals)
        khist_model_from_vals.argtypes = [POINTER((c_uint32 * len(model_vals))), c_size_t]
        khist_model_from_vals.restype = c_void_p
        model.khist = khist_model_from_vals(vals_param, len(model_vals))
        return model

    @staticmethod
    def load_model(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = KH_C.model_from_vals(model_vals)
        file.close()
        return model
