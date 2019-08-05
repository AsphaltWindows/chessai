import random as rand
import numpy as np
import copy as cp


class KHist:
    def __init__(self, start_hists, cluster_num, categories):
        self.categories = categories
        self.cat_num = len(categories)
        self.cluster_num = cluster_num
        self.active_hists = len(start_hists)
        self.cluster_hists = [[[0 for val in range(0, catvals)] for catvals in self.categories] for hist in range(0, self.cluster_num)]
        self.cluster_totals = [0 for cl in range(0, self.cluster_num)]

        for active in range(0, self.active_hists):
            self.cluster_totals[active] = 1

        for hidx, hist in enumerate(start_hists):
            for cat, val in enumerate(hist):
                self.cluster_hists[hidx][cat][val] += 1

    def train_full(self, batch_data):

        print("K-Histograms-Python clustering training on " + str(len(batch_data)) + " data points")

        #debug
        # printstr = ""
        # for cl in range(0, self.cluster_num):
        #     printstr += "Cluster " + str(cl) + ":\n"
        #     for cat in range(0, self.cat_num):
        #         printstr += "Category " + str(cat) + ": "
        #         for val in range(0, self.categories[cat]):
        #             printstr += str(self.cluster_hists[cl][cat][val])
        #         printstr += "\n"
        # print(printstr)

        if len(batch_data) + self.active_hists > self.cluster_num:
            print("Clustering " + str(len(batch_data)) + " data points.")

            print("Selecting additional " + str(self.cluster_num - self.active_hists) + " initial histograms")
            while self.cluster_num > self.active_hists:
                histogram = [[0 for val in range(0, catvals)] for catvals in self.categories]
                rand_hist_centroid = batch_data[rand.randrange(0, len(batch_data))]

                for cat, val in enumerate(rand_hist_centroid):
                    self.cluster_hists[self.active_hists][cat][val] += 1

                self.cluster_totals[self.active_hists] = 1
                self.active_hists += 1

            # #debug
            # printstr = ""
            # for cl in range(0, self.cluster_num):
            #     printstr += "Cluster " + str(cl) + ":\n"
            #     for cat in range(0, self.cat_num):
            #         printstr += "Category " + str(cat) + ": "
            #         for val in range(0, self.categories[cat]):
            #             printstr += str(self.cluster_hists[cl][cat][val])
            #         printstr += "\n"
            # print(printstr)

            dlabels = [-1 for d in batch_data]
            iterations = 0
            cost = 0.0

            print("Performing initial K-Histograms-Python clustering")
            for didx, data in enumerate(batch_data):
                cc = self.assign_cluster(data)
                dlabels[didx] = int(cc[0])
                cost += cc[1]

                # print(str(didx) + ", assigned label: " + str(cc[0]) + ", cost: " + str(cc[1]))

                self.cluster_totals[int(cc[0])] += 1

                for cat in range(0, self.cat_num):
                    self.cluster_hists[int(cc[0])][cat][data[cat]] += 1

            # #debug
            # printstr = ""
            # for cl in range(0, self.cluster_num):
            #     printstr += "Cluster " + str(cl) + ":\n"
            #     for cat in range(0, self.cat_num):
            #         printstr += "Category " + str(cat) + ": "
            #         for val in range(0, self.categories[cat]):
            #             printstr += str(self.cluster_hists[cl][cat][val])
            #         printstr += "\n"
            # print(printstr)

            print("Initial K-Histograms-Python clustering achieved with cost: " + str(cost))

            reassigned_num = 1
            while reassigned_num != 0:
                reassigned_num = 0
                cost = 0

                for didx, d in enumerate(batch_data):
                    cc = self.assign_cluster(d)
                    # print(str(didx) + ", assigned label: " + str(cc[0]) + ", cost: " + str(cc[1]))
                    old_label = dlabels[didx]
                    new_label = int(cc[0])
                    cost += cc[1]

                    if new_label != old_label:
                        reassigned_num += 1
                        dlabels[didx] = new_label
                        self.cluster_totals[old_label] -= 1
                        self.cluster_totals[new_label] += 1
                        for cat, val in enumerate(d):
                            self.cluster_hists[old_label][cat][val] -= 1
                            self.cluster_hists[new_label][cat][val] += 1

                iterations += 1

                print("Completed K-Histograms-Python reclustering, " + str(reassigned_num) + " data points changed labels, total cost is " + str(cost))

    def train_incremental(self, batch_data):

        print("K-Histograms-Python incremental training on " + str(len(batch_data)) + "data points")

        if self.active_hists != self.cluster_num:
            print("K-Histograms-Python can not be trained incrementally, because not all histograms are initialzied")

        cost = 0

        for didx, data in enumerate(batch_data):
            cc = self.assign_cluster(data)
            cost += cc[1]

            for cat, val in enumerate(data):
                self.cluster_hists[int(cc[0])][cat][val] += 1

        print("Completed K-Histograms-Python clustering for " + str(len(batch_data)) + " data points with cost: " + str(cost))

    def assign_cluster(self, data):
        mincost = 0
        mincost_indices = []

        for clidx in range(0, self.cluster_num):
            diffs = 0

            for cat in range(0, self.cat_num):
                diffs += (self.cluster_totals[clidx] - self.cluster_hists[clidx][cat][data[cat]])

            cost = float(diffs) / float(self.cluster_totals[clidx])

            if cost < mincost or len(mincost_indices) == 0:
                mincost = cost
                mincost_indices = [clidx]
            elif cost == mincost:
                mincost_indices.append(clidx)

        mrandval = rand.randrange(0, len(mincost_indices))
        return mincost_indices[mrandval], mincost
        # return mincost_indices[0], mincost

    def model_val_num(self):
        return 2 + len(self.categories) + len(self.cluster_num) + len(self.categories) * self.cluster_num

    def model_to_vals(self):
        model_vals = [len(self.categories), self.cluster_num]
        model_vals += self.categories
        model_vals += self.cluster_totals
        for cl in self.cluster_hists:
            for catvals in cl: 
                model_vals += catvals
        return model_vals

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
        model = KHist([], cluster_num, cats)

        for cl in range(0, cluster_num):
            model.cluster_totals[cl] = model_vals[at]
            at += 1

        for cl in range(0, cluster_num):
            for cat in range(0, cat_num):
                for val in range(0, cats[cat]):
                    model.cluster_hists[cl][cat][val] = model_vals[at]
                    at += 1

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = KHist.model_from_vals(model_vals)
        file.close()
        return model
