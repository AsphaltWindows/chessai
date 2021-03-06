from ctypes import *


bcclib = CDLL("models/libbcc.so")

create_bcc = bcclib.create_bcc

bcc_train_batch = bcclib.bcc_train_batch

bcc_predict_class = bcclib.bcc_predict_class

bcc_from_file_with_name = bcclib.bcc_from_file_with_name
bcc_from_file_with_name.argtypes = [c_char_p, c_uint8]
bcc_from_file_with_name.restype = c_void_p

bcc_to_file_with_name = bcclib.bcc_to_file_with_name
bcc_to_file_with_name.argtypes = [c_void_p, c_char_p]

free_bcc = bcclib.free_bcc
free_bcc.argtypes = [c_void_p]
free_bcc.restype = None

free = bcclib.free
free.argtypes = [c_void_p]
free.restype = None

class BCC_C:
    def __init__(self, class_num, categories, alpha = 1.0, use_probs = 0):
        self.class_num = class_num
        self.cat_num = len(categories)
        self.categories = categories
        CatArray = c_uint8 * self.cat_num
        categories_param = CatArray(*categories)
        create_bcc.argtypes = [POINTER(c_uint8 * self.cat_num), c_size_t, c_uint8, c_double, c_int]
        create_bcc.restype = c_void_p
        self.bcc = create_bcc(categories_param, self.cat_num, class_num, alpha, use_probs)

    def free_model(self):
        free_bcc(self.bcc)

    def train_batch(self, labels, data):
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

        bcc_train_batch.argtypes = [c_void_p, DataArray, LabelArray, c_size_t]
        bcc_train_batch.restype = None
        bcc_train_batch(self.bcc, data_param, label_param, len(data))

    def predict_class(self, data):
        start_labels = [0] * self.class_num
        DataArray = c_uint8 * len(data)
        LabelArray = c_double * self.class_num
        # labels_uncasted = (c_double * self.class_num)()
        data_param = DataArray(*data)
        label_param = LabelArray(*start_labels)
        bcc_predict_class.argtypes = [c_void_p, c_uint8 * self.cat_num, c_double * self.class_num]
        bcc_predict_class(self.bcc, data_param, label_param)
        res = [r for r in label_param]
        return res

    def model_to_file(self, filename):
        bcc_to_file_with_name(self.bcc, c_char_p(bytes(filename, 'utf-8')))

    @staticmethod
    def model_from_file(filename, categories, class_num, use_probs):
        model = BCC_C(class_num, categories, 1, 1, 1)
        free_bcc(model.bcc)
        model.bcc = bcc_from_file_with_name(c_char_p(bytes(filename, 'utf-8')), use_probs)
        return model
