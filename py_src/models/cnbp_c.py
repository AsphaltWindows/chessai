from ctypes import *


cnbplib = CDLL("models/libcnbp.so")

create_cnbp_with_alpha = cnbplib.create_cnbp_with_alpha

cnbp_train_batch = cnbplib.train_batch

predict_class = cnbplib.predict_class

cnbp_from_file_with_name = cnbplib.bdt_from_file_with_name
cnbp_from_file_with_name.argtypes = [c_char_p, c_uint8]
cnbp_from_file_with_name.restype = c_void_p

cnbp_to_file_with_name = cnbplib.bdt_to_file_with_name
cnbp_to_file_with_name.argtypes = [c_void_p, c_char_p]

free_cnbp = cnbplib.free_cnbp
free_cnbp.argtypes = [c_void_p]
free_cnbp.restype = None

free = cnbplib.free
free.argtypes = [c_void_p]
free.restype = None


class CNBP_C:
    def __init__(self, class_num, categories, alpha = 1, use_probs = 0):
        self.class_num = class_num
        self.cat_num = len(categories)
        self.categories = categories
        CatArray = c_uint8 * self.cat_num
        categories_param = CatArray(*categories)
        create_cnbp_with_alpha.argtypes = [c_uint8, POINTER(c_uint8 * self.cat_num), c_size_t, c_double, c_int]
        create_cnbp_with_alpha.restype = c_void_p
        self.cnbp = create_cnbp_with_alpha(class_num, categories_param, self.cat_num, alpha, use_probs)

    def free_model(self):
        free_cnbp(self.cnbp)

    def train_batch(self, labels, data):
        print("training batch")
        DataArray = POINTER(c_uint8 * self.cat_num) * len(data)
        LabelArray = POINTER(c_double * self.class_num) * len(data)

        data_list = []
        data_row_pointer_type = POINTER(c_uint8 * self.cat_num)
        for d in data:
            DataRow = c_uint8 * self.cat_num
            row_param = data_row_pointer_type(DataRow(*d))
            data_list.append(row_param)
        data_param = DataArray(*data_list)

        label_list = []
        label_row_pointer_type = POINTER(c_double * self.class_num)
        for l in labels:
            LabelRow = c_double * self.class_num
            row_param = label_row_pointer_type(LabelRow(*l))
            label_list.append(row_param)
        label_param = LabelArray(*label_list)

        print("Trying to train batch")
        cnbp_train_batch.argtypes = [c_void_p, DataArray, LabelArray, c_size_t]
        cnbp_train_batch.restype = None
        cnbp_train_batch(self.cnbp, data_param, label_param, len(data))

    def predict_class(self, data):
        DataArray = c_uint8 * len(data)
        data_param = DataArray(*data)
        predict_class.argtypes = [c_void_p, POINTER(c_uint8 * self.cat_num)]
        predict_class.restype = POINTER(c_double * self.class_num)
        c_res = predict_class(self.cnbp, data_param)
        res = [r for r in c_res.contents]
        free(c_res)
        return res

    def model_to_file(self, filename):
        cnbp_to_file_with_name(self.cnbp, c_char_p(bytes(filename, 'utf-8')))

    @staticmethod
    def model_from_file(filename, categories, class_num, use_probs):
        model = CNBP_C(class_num, categories, 1, 1)
        free_cnbp(model.cnbp)
        model.cnbp = cnbp_from_file_with_name(c_char_p(bytes(filename, 'utf-8')), use_probs)
        return model


