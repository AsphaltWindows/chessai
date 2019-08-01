import random as rand
import numpy as np
import copy as cp


class KModes:
    def __init__(self, start_modes, cluster_num, categories):
        self.categories = categories
        self.cat_num = len(categories)
        self.cluster_num = cluster_num
        self.active_modes = len(start_modes)
        self.cluster_modes = start_modes

    def train_batch(self, batch_data):

        print("K-Modes-Python clustering training on " + str(len(batch_data)) + " data points")

        if len(batch_data) > self.cluster_num:
            print("Clustering " + str(len(batch_data)) + " data points.")

            while self.cluster_num > self.active_modes:
                print("Selecting additional " + str(self.cluster_num - self.active_modes) + " initial modes")
                self.cluster_modes.append(batch_data[rand.randrange(0, len(batch_data))])
                self.active_modes += 1

            iterations = 0
            recluster_num = 1
            last_cost = 0

            while True:
                cost = 0

                frequencies = [[[0 for n in range(0, cat)] for cat in self.categories] for m in self.cluster_modes]

                for d in batch_data:
                    assignment = self.assign_cluster(d)
                    cost += assignment[1]
                    print("py_cost:" + str(assignment[1]) + ", cluster:" + str(assignment[0]))

                    for cat, val in enumerate(d):
                        frequencies[assignment[0]][cat][val] += 1

                for midx, m in enumerate(self.cluster_modes):
                    printstr = "Mode " + str(midx) + ": "
                    for val in m:
                        printstr += str(val)
                    print(printstr)

                if iterations >= 1 and cost >= last_cost:
                    print("Last cost: " + str(last_cost) + " cost: " + str(cost))
                    break

                iterations += 1
                last_cost = cost

                print("Finished clustering attempt number: " + str(iterations) + " cost: " + str(cost))

                tmp_modes = cp.deepcopy(self.cluster_modes)

                for clidx, cl in enumerate(self.cluster_modes):
                    for catidx, valnum in enumerate(self.categories):
                        mfreq = 0
                        mfreq_idx = []

                        for val in range(0, valnum):
                            f = frequencies[clidx][catidx][val]

                            if f > mfreq or len(mfreq_idx) == 0:
                                mfreq_idx = [val]
                                mfreq = f
                            elif f == mfreq:
                                mfreq_idx.append(val)

                        mrandval = rand.randrange(0, len(mfreq_idx))
                        if len(mfreq_idx) == 1:
                            self.cluster_modes[clidx][catidx] = mfreq_idx[0]
                        else:
                            #self.cluster_modes[clidx][catidx] = mfreq_idx[mrandval]
                            self.cluster_modes[clidx][catidx] = mfreq_idx[0]

            self.cluster_modes = tmp_modes

            for midx, m in enumerate(self.cluster_modes):
                printstr = "Mode " + str(midx) + ": "
                for val in m:
                    printstr += str(val)
                print(printstr)

    def assign_cluster(self, data):
        mincost = 0
        mincost_indices = []

        for clidx in range(0, self.cluster_num):
            cost = 0

            for cat in range(0, self.cat_num):
                if data[cat] != self.cluster_modes[clidx][cat]:
                    cost += 1

            if cost < mincost or len(mincost_indices) == 0:
                mincost = cost
                mincost_indices = [clidx]
            elif cost == mincost:
                mincost_indices.append(clidx)

        mrandval = rand.randrange(0, len(mincost_indices))
        #return mincost_indices[mrandval], mincost
        return mincost_indices[0], mincost


    def model_to_string(self):
        model_str = " ".join(map(str, self.categories)) + '\n'
        model_str += (str(self.cluster_num) + '\n')
        for m in self.cluster_modes:
            model_str += " ".join(map(str, m)) + '\n'
        return model_str

    def model_val_num(self):
        return 2 + len(self.categories) + len(self.categories) * self.cluster_num

    def model_to_vals(self):
        model_vals = [len(self.categories), self.cluster_num]
        model_vals += self.categories
        for cl in self.cluster_modes:
            model_vals += cl

        return model_vals

    def store_model(self, file_name):
        file = open(file_name, "w")
        file.write(self.model_to_string())
        file.close()

    def store_model2(self, file_name):
        file = open(file_name, "w")
        file.write("\n".join(map(str, self.model_to_vals())))
        file.close()

    @staticmethod
    def model_from_vals(model_vals):
        cat_num = model_vals[0]
        cluster_num = model_vals[1]
        cats = model_vals[2:cat_num + 2]
        at = cat_num + 2
        model = KModes([], cluster_num, cats)

        for idx in range(0, cluster_num):
            model.cluster_modes.append(model_vals[at:at + cat_num])
            at += cat_num
        return model

    @staticmethod
    def model_from_lines(model_lines):
        cats = list(map(int, model_lines[0].split(" ")))
        cluster_num = int(model_lines[1])
        model = KModes([], cluster_num, cats)
        for idx in range(0, cluster_num):
            model.cluster_modes.append(list(map(int, model_lines[idx + 2].split(" "))))
        return model

    @staticmethod
    def load_model(file_name):
        file = open(file_name, "r")
        model_lines = file.readlines()
        model = KModes.model_from_lines(model_lines)
        file.close()
        return model

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines())) 
        model = KModes.model_from_vals(model_vals)
        file.close()
        return model
