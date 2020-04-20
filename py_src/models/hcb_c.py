from ctypes import *


hcblib = CDLL("models/libhcb.so")

create_hcb_with_alpha = hcblib.create_hcb_with_alpha

hcb_train_batch = hcblib.train_batch

predict_class = hcblib.predict_class

hcb_model_from_file = hcblib.hcb_model_from_file

hcb_model_to_file = hcblib.hcb_model_to_file

free_hcb = hcblib.free_hcb
free_hcb.argtypes = [c_void_p]
free_hcb.restype = None

free = hcblib.free
free.argtypes = [c_void_p]
free.restype = None


class CNB_C:
    def __init__(self, class_num, categories, alpha = 1):
        self.class_num = class_num
        self.cat_num = len(categories)
        self.categories = categories
        CatArray = c_uint8 * self.cat_num
        categories_param = CatArray(*categories)
        create_hcb_with_alpha.argtypes = [c_uint8, POINTER(c_uint8 * self.cat_num), c_size_t, c_double]
        create_hcb_with_alpha.restype = c_void_p
        self.hcb = create_hcb_with_alpha(class_num, categories_param, self.cat_num, alpha)

    def free_model(self):
        free_hcb(self.hcb)

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
        hcb_train_batch.argtypes = [c_void_p, DataArray, POINTER(c_uint8 * len(labels)), c_size_t]
        hcb_train_batch.restype = None
        hcb_train_batch(self.hcb, data_param, labels_param, len(data))

    def predict_class(self, data):
        DataArray = c_uint8 * len(data)
        data_param = DataArray(*data)
        predict_class.argtypes = [c_void_p, POINTER(c_uint8 * self.cat_num)]
        predict_class.restype = POINTER(c_double * self.class_num)
        c_res = predict_class(self.hcb, data_param)
        res = [r for r in c_res.contents]
        free(c_res)
        return res

    def model_val_num(self):
        return 4 + self.cat_num + self.class_num + (self.class_num * sum(self.categories))

    def model_to_file(self):
        hcb_model_to_file.argtypes = [c_void_p]
        hcb_model_to_file.restype = POINTER(c_uint32 * self.model_val_num())
        c_res = hcb_model_to_file(self.hcb)
        res = [r for r in c_res.contents]
        free(c_res)
        return res

    def store_model2(self, file_name):
        file = open(file_name, "w")
        file.write("\n".join(map(str, self.model_to_file())))
        file.close()

    @staticmethod
    def model_from_file(model_vals):
        class_num = int(model_vals[0])
        alpha = float(model_vals[1])/ model_vals[2]
        cat_num = model_vals[3]
        cats = model_vals[4:4 + cat_num]
        model = CNB_C(class_num, cats, alpha)
        free_hcb(model.hcb)
        ValsArray = c_uint32 * len(model_vals)
        vals_param = ValsArray(*model_vals)
        hcb_model_from_file.argtypes = [POINTER(c_uint32 * len(model_vals)), c_size_t]
        hcb_model_from_file.restype = c_void_p
        model.hcb = hcb_model_from_file(vals_param, len(model_vals))
        return model

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = CNB_C.model_from_file(model_vals)
        file.close()
        print("Loaded Categorical Bayes Classifier")
        return model



