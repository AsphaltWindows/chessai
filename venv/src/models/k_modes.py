import random as rand
import numpy as np
import copy as cp


class KModes:
    def __init__(self, classes):
        self.classes = classes
        self.row_size = len(classes)
        self.cluster_num = 0
        self.cluster_modes = []
        self.data_labels = []
        self.data = []

    def train_batch(self, start_modes, cluster_num, batch_data):
        self.cluster_modes = start_modes
        self.cluster_num = cluster_num
        self.data = batch_data
        self.data_labels = [0 for d in batch_data]

        if len(batch_data) > cluster_num:
            print("Clustering " + str(len(batch_data)) + " data points.")

            additional_modes = cluster_num - start_modes
            indices = set()
            while len(indices) != additional_modes:
                indices.add(rand.randrange(0, len(batch_data)))

            for idx in indices:
                self.cluster_modes.append(batch_data[idx])

            self.cluster_data()
            self.recalculate_modes()
            last_loss = self.calculate_loss()
            last_labels = cp.deepcopy(self.data_labels)
            last_modes = cp.deepcopy(self.cluster_modes)
            self.cluster_data()
            self.recalculate_modes()

            while last_loss > self.calculate_loss():
                last_labels = cp.deepcopy(self.data_labels)
                last_modes = cp.deepcopy(self.cluster_modes)
                self.cluster_data()
                self.recalculate_modes()

            self.data_labels = last_labels
            self.cluster_modes = last_modes

    def cluster_data(self):
        for idx, d in enumerate(self.data):
            self.data_labels[idx] = rand.choice(self.assign_cluster(d))

    def recalculate_modes(self):
        freqs = [[[0 for n in cl] for cl in self.classes] for m in self.cluster_modes]

        for didx, d in enumerate(self.data):
            for cidx, value in enumerate(d):
                freqs[self.data_labels[didx]][cidx][value] += 1

        for midx, f in enumerate(freqs):
            for cidx, clfreqs in enumerate(f):
                self.cluster_modes[midx][cidx] = np.argmax(clfreqs)

    def calculate_loss(self):
        loss = 0
        for didx, label in self.data_labels:
            loss += self.dissimilarity(self.data[didx], label)
        return loss

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

        for idx in range(1, self.cluster_num - 1):
            diss = self.dissimilarity(data, idx)
            if diss < min_score:
                cluster_indices = [idx]
                min_score = diss
            elif diss == min_score:
                cluster_indices.append(idx)

        return cluster_indices

    def store_model(self, file_name):
        file = open(file_name, "w")
        file.write(" ".join(map(str, self.classes)) + '\n')
        file.write(str(self.cluster_num))
        for m in self.cluster_modes:
            file.write(" ".join(map(str, m)) + '\n')
        file.close()


def k_modes(file_name):
    file = open(file_name, "r")
    fmodel = file.readlines()
    classes = list(map(int, fmodel[0].split(" ")))
    model = KModes(classes)
    cluster_num = int(fmodel[1])
    model.cluster_num = cluster_num
    for idx in range(0, cluster_num):
        model.cluster_modes[idx] = map(int, fmodel[idx + 2].split(" "))
    file.close()
    return model
