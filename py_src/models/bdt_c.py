from ctypes import *


bdtlib = CDLL("models/libbdt.so")

create_bdt = bdtlib.create_bdt

bdt_train_batch = bdtlib.bdt_train_batch

bdt_predict_class = bdtlib.bdt_predict_class

bdt_from_file_with_name = bdtlib.bdt_from_file_with_name
bdt_from_file_with_name.argtypes = [c_char_p, c_uint8]
bdt_from_file_with_name.restype = c_void_p

bdt_to_file_with_name = bdtlib.bdt_to_file_with_name
bdt_to_file_with_name.argtypes = [c_void_p, c_char_p]

free_bdt = bdtlib.free_bdt
free_bdt.argtypes = [c_void_p]
free_bdt.restype = None

free = bdtlib.free
free.argtypes = [c_void_p]
free.restype = None


class BDT_C:
    def __init__(self, categories, class_num, branch_factor, split_threshold, split_limit, forget_factor, alpha, use_probs):
        self.class_num = class_num
        self.cat_num = len(categories)
        self.categories = categories
        CatArray = c_uint8 * self.cat_num
        categories_param = CatArray(*categories)
        create_bdt.argtypes = [
            POINTER(c_uint8 * self.cat_num),
            c_size_t,
            c_uint8,
            c_uint8,
            c_uint32,
            c_uint32,
            c_double,
            c_double,
            c_uint8]
        create_bdt.restype = c_void_p
        self.bdt = create_bdt(
            categories_param,
            self.cat_num,
            self.class_num,
            branch_factor,
            split_threshold,
            split_limit,
            forget_factor,
            alpha,
            use_probs)

    def free_model(self):
        free_bdt(self.bdt)

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
        bdt_train_batch.argtypes = [c_void_p, DataArray, LabelArray, c_size_t]
        bdt_train_batch.restype = None
        bdt_train_batch(self.bdt, data_param, label_param, len(data))

    def predict_class(self, data):
        DataArray = c_uint8 * len(data)
        labels_uncasted = (c_double * self.class_num)()
        data_param = DataArray(*data)
        # bdt_predict_class.argtypes = [c_void_p, c_uint8 * self.cat_num, c_double * self.class_num]
        print("about to make call")
        bdt_predict_class(self.bdt, data_param, cast(labels_uncasted, POINTER(c_double)))
        print("exited call")
        res = [r for r in labels_uncasted]
        print(res)
        return res

    def model_to_file(self, filename):
        bdt_to_file_with_name(self.bdt, c_char_p(bytes(filename, 'utf-8')))

    @staticmethod
    def model_from_file(filename, categories, class_num, use_probs):
        model = BDT_C(categories, class_num, 0, 0, 0, 0, 0, 1)
        free_bdt(model.bdt)
        model.bdt = bdt_from_file_with_name(c_char_p(bytes(filename, 'utf-8')), use_probs)
        return model



