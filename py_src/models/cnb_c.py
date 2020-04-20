from ctypes import *


cnblib = CDLL("models/libcnb.so")

create_cnb_with_alpha = cnblib.create_cnb_with_alpha

cnb_train_batch = cnblib.train_batch

predict_class = cnblib.predict_class

cnb_model_from_vals = cnblib.cnb_model_from_vals

cnb_model_to_vals = cnblib.cnb_model_to_vals

free_cnb = cnblib.free_cnb
free_cnb.argtypes = [c_void_p]
free_cnb.restype = None

free = cnblib.free
free.argtypes = [c_void_p]
free.restype = None


class CNB_C:
    def __init__(self, class_num, categories, alpha = 1):
        self.class_num = class_num
        self.cat_num = len(categories)
        self.categories = categories
        CatArray = c_uint8 * self.cat_num
        categories_param = CatArray(*categories)
        create_cnb_with_alpha.argtypes = [c_uint8, POINTER(c_uint8 * self.cat_num), c_size_t, c_double]
        create_cnb_with_alpha.restype = c_void_p
        self.cnb = create_cnb_with_alpha(class_num, categories_param, self.cat_num, alpha)

    def free_model(self):
        free_cnb(self.cnb)

    def train_batch(self, labels, data):
        LabelsArray = c_uint8 * len(labels)
        labels_param = LabelsArray(*labels)
        DataArray = POINTER(c_uint8 * self.cat_num) * len(data)
        data_list = []
        data_row_pointer_type = POINTER(c_uint8 * self.cat_num)
        for d in data:
            DataRow = c_uint8 * self.cat_num
            row_param = data_row_pointer_type(DataRow(*d))
            data_list.append(row_param)
        data_param = DataArray(*data_list)
        cnb_train_batch.argtypes = [c_void_p, DataArray, POINTER(c_uint8 * len(labels)), c_size_t]
        cnb_train_batch.restype = None
        cnb_train_batch(self.cnb, data_param, labels_param, len(data))

    def predict_class(self, data):
        DataArray = c_uint8 * len(data)
        data_param = DataArray(*data)
        predict_class.argtypes = [c_void_p, POINTER(c_uint8 * self.cat_num)]
        predict_class.restype = POINTER(c_double * self.class_num)
        c_res = predict_class(self.cnb, data_param)
        res = [r for r in c_res.contents]
        free(c_res)
        return res

    def model_val_num(self):
        return 4 + self.cat_num + self.class_num + (self.class_num * sum(self.categories))

    def model_to_vals(self):
        cnb_model_to_vals.argtypes = [c_void_p]
        cnb_model_to_vals.restype = POINTER(c_uint32 * self.model_val_num())
        c_res = cnb_model_to_vals(self.cnb)
        res = [r for r in c_res.contents]
        free(c_res)
        return res

    def store_model2(self, file_name):
        file = open(file_name, "w")
        file.write("\n".join(map(str, self.model_to_vals())))
        file.close()

    @staticmethod
    def model_from_vals(model_vals):
        class_num = int(model_vals[0])
        alpha = float(model_vals[1])/ model_vals[2]
        cat_num = model_vals[3]
        cats = model_vals[4:4 + cat_num]
        model = CNB_C(class_num, cats, alpha)
        free_cnb(model.cnb)
        ValsArray = c_uint32 * len(model_vals)
        vals_param = ValsArray(*model_vals)
        cnb_model_from_vals.argtypes = [POINTER(c_uint32 * len(model_vals)), c_size_t]
        cnb_model_from_vals.restype = c_void_p
        model.cnb = cnb_model_from_vals(vals_param, len(model_vals))
        return model

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = CNB_C.model_from_vals(model_vals)
        file.close()
        print("Loaded Categorical Bayes Classifier")
        return model



