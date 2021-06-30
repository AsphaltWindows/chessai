from ctypes import *


cnbp_chainlib = CDLL("models/libcnbp_chain.so")

create_cnbp_chain = cnbp_chainlib.create_cnbp_chain

cnbp_chain_train_batch = cnbp_chainlib.cnbp_chain_train_batch

cnbp_chain_predict_class = cnbp_chainlib.cnbp_chain_predict_class

cnbp_chain_from_file_with_name = cnbp_chainlib.cnbp_chain_from_file_with_name
cnbp_chain_from_file_with_name.argtypes = [c_char_p, c_uint8]
cnbp_chain_from_file_with_name.restype = c_void_p

cnbp_chain_to_file_with_name = cnbp_chainlib.cnbp_chain_to_file_with_name
cnbp_chain_to_file_with_name.argtypes = [c_void_p, c_char_p]

free_cnbp_chain = cnbp_chainlib.free_cnbp_chain
free_cnbp_chain.argtypes = [c_void_p]
free_cnbp_chain.restype = None

free = cnbp_chainlib.free
free.argtypes = [c_void_p]
free.restype = None

class CNBP_CHAIN_C:
    def __init__(self, class_num, categories, nodes_num, alpha = 1.0, use_probs = 0):
        self.class_num = class_num
        self.cat_num = len(categories)
        self.categories = categories
        CatArray = c_uint8 * self.cat_num
        categories_param = CatArray(*categories)
        create_cnbp_chain.argtypes = [POINTER(c_uint8 * self.cat_num), c_size_t, c_uint8, c_size_t, c_double, c_int]
        create_cnbp_chain.restype = c_void_p
        self.cnbp_chain = create_cnbp_chain(categories_param, self.cat_num, class_num, nodes_num, alpha, use_probs)

    def free_model(self):
        free_cnbp_chain(self.cnbp_chain)

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

        cnbp_chain_train_batch.argtypes = [c_void_p, DataArray, LabelArray, c_size_t]
        cnbp_chain_train_batch.restype = None
        cnbp_chain_train_batch(self.cnbp_chain, data_param, label_param, len(data))

    def predict_class(self, data):
        start_labels = [0] * self.class_num
        DataArray = c_uint8 * len(data)
        LabelArray = c_double * self.class_num
        # labels_uncasted = (c_double * self.class_num)()
        data_param = DataArray(*data)
        label_param = LabelArray(*start_labels)
        cnbp_chain_predict_class.argtypes = [c_void_p, c_uint8 * self.cat_num, c_double * self.class_num]
        cnbp_chain_predict_class(self.cnbp_chain, data_param, label_param)
        res = [r for r in label_param]
        return res

    def model_to_file(self, filename):
        cnbp_chain_to_file_with_name(self.cnbp_chain, c_char_p(bytes(filename, 'utf-8')))

    @staticmethod
    def model_from_file(filename, categories, class_num, use_probs):
        model = CNBP_CHAIN_C(class_num, categories, 1, 1, 1)
        free_cnbp_chain(model.cnbp_chain)
        model.cnbp_chain = cnbp_chain_from_file_with_name(c_char_p(bytes(filename, 'utf-8')), use_probs)
        return model


