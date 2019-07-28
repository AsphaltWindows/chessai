from ctypes import *


lib = CDLL("models/libkmodes.so")

create_kmodes = lib.create_kmodes

train_batch = lib.train_batch

assign_cluster = lib.assign_cluster

kmodes_model_from_vals = lib.kmodes_model_from_vals

kmodes_model_to_vals = lib.kmodes_model_to_vals

free_kmodes = lib.free_kmodes
free_kmodes.argtypes = [c_void_p]
free_kmodes.restype = None

free = lib.free
free.argtypes = [c_void_p]
free.restype = None


class KM_C:
    def __init__(self, start_modes, cluster_num, categories):
        self.categories = categories
        self.cat_num = len(categories)
        self.cluster_num = cluster_num
        self.cluster_modes = start_modes

    def train_batch(self, batch_data):


    def assign_cluster(self, data):


    def model_to_vals(self):


    def store_model2(self, file_name):


    @staticmethod
    def model_from_vals(model_vals):


    @staticmethod
    def load_model(file_name):

