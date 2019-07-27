import random as rand
import numpy as np
import copy as cp


class KModes:
    def __init__(self, categories):
        self.categories = categories
        self.row_size = len(categories)
        self.cluster_num = 0
        self.cluster_modes = []
        self.frequencies = []
        self.data_labels = []
        self.data = []

    def train_batch(self, start_modes, cluster_num, batch_data):
        self.cluster_modes = start_modes
        self.cluster_num = cluster_num
        self.data = batch_data
        self.data_labels = [0 for d in batch_data]
        self.frequencies = [[[0 for n in range(0, cat)] for cat in self.categories] for m in self.cluster_modes]

        if len(batch_data) > cluster_num:
            print("Clustering " + str(len(batch_data)) + " data points.")

            additional_modes = cluster_num - len(start_modes)
            indices = set()
            while len(indices) != additional_modes:
                indices.add(rand.randrange(0, len(batch_data)))

            for idx in indices:
                self.cluster_modes.append(batch_data[idx])

            print("Initial clustering")
            self.cluster_data()
            print("Calculating cost")
            last_cost = self.calculate_cost()
            print("Cost: " + str(last_cost))

            recluster_num = 1
            while True:
                print("Reclustering number: " + str(recluster_num) + " last_cost: " + str(last_cost))
                last_labels = cp.deepcopy(self.data_labels)
                last_modes = cp.deepcopy(self.cluster_modes)
                self.cluster_data()
                current_cost = self.calculate_cost()
                if current_cost >= last_cost:
                    break
                else:
                    last_cost = current_cost
                recluster_num += 1

            self.data_labels = last_labels
            self.cluster_modes = last_modes

    def cluster_data(self):
        self.frequencies = [[[0 for n in range(0, cat)] for cat in self.categories] for m in self.cluster_modes]
        for didx, d in enumerate(self.data):
            assign_to = rand.choice(self.assign_cluster(d)[0])
            self.data_labels[didx] = assign_to
            for cidx, value, in enumerate(d):
                self.frequencies[assign_to][cidx][value] += 1

        for midx, f in enumerate(self.frequencies):
            for cidx, catfreqs in enumerate(f):
                self.cluster_modes[midx][cidx] = np.argmax(catfreqs)

    def calculate_cost(self):
        cost = 0
        for didx, label in enumerate(self.data_labels):
            cost += self.dissimilarity(self.data[didx], label)
        return cost

    def dissimilarity(self, row, mode_idx):
        diss = 0
        mode = self.cluster_modes[mode_idx]
        for idx in range(0, self.row_size):
            if row[idx] != mode[idx]:
                diss += 1
        return diss

    def assign_cluster(self, data):
        cluster_indices = [0]
        min_score = self.dissimilarity(data, 0)
        for idx in range(1, self.cluster_num):
            diss = self.dissimilarity(data, idx)
            if diss < min_score:
                cluster_indices = [idx]
                min_score = diss
            elif diss == min_score:
                cluster_indices.append(idx)

        return cluster_indices, min_score

    def model_to_string(self):
        model_str = " ".join(map(str, self.categories)) + '\n'
        model_str += (str(self.cluster_num) + '\n')
        for m in self.cluster_modes:
            model_str += " ".join(map(str, m)) + '\n'
        return model_str

    def store_model(self, file_name):
        file = open(file_name, "w")
        file.write(self.model_to_string())
        file.close()

    @staticmethod
    def model_from_lines(model_lines):
        cats = list(map(int, model_lines[0].split(" ")))
        model = KModes(cats)
        cluster_num = int(model_lines[1])
        model.cluster_num = cluster_num
        for idx in range(0, cluster_num):
            model.cluster_modes.append(list(map(int, model_lines[idx + 2].split(" "))))
        return model

    @staticmethod
    def load_model(file_name):
        file = open(file_name, "r")
        model_lines = file.readlines()
        model = KModes.model_from_lines(model_lines)
        file.close()
        print("Loaded K-Modes model")
        return model
