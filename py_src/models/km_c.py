from ctypes import *

kmlib = CDLL("models/libkmodes.so")

create_kmodes = kmlib.create_kmodes
create_kmodes.restype = c_void_p

km_train_batch = kmlib.train_batch
km_train_batch.restype = None

assign_cluster = kmlib.assign_cluster
assign_cluster.restype = POINTER(c_uint8 * 2)

kmodes_model_from_vals = kmlib.kmodes_model_from_vals

kmodes_model_to_vals = kmlib.kmodes_model_to_vals
kmodes_model_to_vals.argtypes = [c_void_p]

free_kmodes = kmlib.free_kmodes
free_kmodes.argtypes = [c_void_p]
free_kmodes.restype = None

free = kmlib.free
free.argtypes = [c_void_p]
free.restype = None


class KM_C:
    def __init__(self, start_modes, cluster_num, categories):
        self.cat_num = len(categories)
        self.cluster_num = cluster_num
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
        create_kmodes.argtypes = [POINTER(CatArray), c_size_t, c_uint8, ModesArray, c_size_t]
        self.kmodes = create_kmodes(categories_param, self.cat_num, cluster_num, modes_param, len(start_modes))
        print("kmodes: " + str(self.kmodes))

    def train_batch(self, data):
        print("training kmodes batch")
        DataArray = POINTER(c_uint8 * self.cat_num) * len(data)
        data_list = []
        data_row_pointer_type = POINTER(c_uint8 * self.cat_num)
        for d in data:
            DataRow = c_uint8 * self.cat_num
            row_param = data_row_pointer_type(DataRow(*d))
            data_list.append(row_param)
        data_param = DataArray(*data_list)
        km_train_batch.argtypes = [c_void_p, DataArray, c_size_t]
        print("here we go")
        print(km_train_batch)
        print("kmodes: " + str(self.kmodes))
        km_train_batch(self.kmodes, data_param, len(data))

    def assign_cluster(self, row):
        RowArray = c_uint8 * self.cat_num
        row_param = RowArray(*row)
        assign_cluster.argtypes = [c_void_p, POINTER(c_uint8 * len(row))]
        c_res = assign_cluster(self.kmodes, row_param, len(row))
        res = (c_res.contents[0], c_res.contents[1])
        free(c_res)
        return res

    def model_val_num(self):
        return 2 + self.cat_num + self.cat_num * self.cluster_num

    def model_to_vals(self):
        kmodes_model_to_vals.restype = POINTER(c_uint32 * self.model_val_num())
        c_res = kmodes_model_to_vals(self.kmodes)
        res = [r for r in c_res.contents]
        free(c_res)
        return res

    def store_model2(self, file_name):
        file = open(file_name, "w")
        file.write("\n".join(map(str, self.model_to_vals())))
        file.close()

    def free_kmodes(self):
        free_kmodes(self.kmodes)

    @staticmethod
    def model_from_vals(model_vals):
        cat_num = model_vals[0]
        cluster_num = model_vals[1]
        categories = model_vals[2:2+cat_num]
        model = KM_C([], cluster_num, categories)
        free_kmodes(model.kmodes)
        ValsArray = c_uint32 * len(model_vals)
        vals_param = ValsArray(*model_vals)
        kmodes_model_from_vals.argtypes = [POINTER((c_uint32 * len(model_vals))), c_size_t]
        kmodes_model_from_vals.restype = c_void_p
        model.kmodes = kmodes_model_from_vals(vals_param, len(model_vals))
        return model

    @staticmethod
    def load_model(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = KM_C.model_from_vals(model_vals)
        file.close()
        return model
