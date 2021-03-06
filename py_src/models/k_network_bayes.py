import models.categorical_naive_bayes as cnb
import models.k_modes as km

import random as rand


class KNetworkBayes:
    def __init__(self, class_num, categories, clusters_per_node, node_num, layer_num, alpha=1):
        self.class_num = class_num
        self.categories = categories
        self.clusters_per_node = clusters_per_node
        self.node_num = node_num
        self.layer_num = layer_num
        self.alpha = alpha
        self.layers = [[None for n in range(0, node_num)] for l in range(0, self.layer_num)]
        self.classifier = None

    def train_batch(self, labels, batch_data):
        categories = self.categories
        data = batch_data
        clustered_data = []
        clustered_categories = []
        clustered_categories += self.categories
        for lidx, l in enumerate(self.layers):
            print("Clustering seed of layer " + str(lidx))
            initial_clustering = km.KModes(categories)
            initial_clustering.train_batch([], self.clusters_per_node * self.node_num, data)
            modes = initial_clustering.cluster_modes
            rand.shuffle(modes)
            print("Clustering nodes of layer " + str(lidx))
            for n in range(0, self.node_num):
                initial_modes = modes[n * self.clusters_per_node: (n+1) * self.clusters_per_node]
                node = km.KModes(categories)
                node.train_batch(initial_modes, self.clusters_per_node, data)
                l[n] = node
            print("Generating labels from nodes of layer " + str(lidx))
            categories = [self.clusters_per_node for n in range(0, self.node_num)]
            clustered_categories += categories
            new_data = []
            for didx, d in enumerate(data):
                new_data.append([])
                for nidx, n in enumerate(l):
                    value = n.assign_cluster(d)[0]
                    new_data[didx].append(value)
            clustered_data.append(new_data)
            data = new_data
            print(str(data[0]))

        self.classifier = cnb.CategoricalNaiveBayes(self.class_num, clustered_categories, self.alpha)

        for didx, d in enumerate(batch_data):
            for cd in clustered_data:
                d += cd[didx]

        self.classifier.train_batch(labels, batch_data)

    def predict_class(self, data):
        enriched_data = data
        for_layer = data
        for l in self.layers:
            new_for_layer = []
            for n in l:
                value = n.assign_cluster(for_layer)[0]
                new_for_layer.append(value)
            enriched_data += new_for_layer
            for_layer = new_for_layer

        return self.classifier.predict_class(enriched_data)

    def model_to_string(self):
        model_str = str(self.class_num) + '\n'
        model_str += " ".join(map(str, self.categories)) + '\n'
        model_str += str(self.clusters_per_node) + '\n'
        model_str += str(self.node_num) + '\n'
        model_str += str(self.layer_num) + '\n'
        model_str += str(self.alpha) + '\n'
        for l in self.layers:
            for n in l:
                model_str += n.model_to_string()
        model_str += self.classifier.model_to_string()
        return model_str

    def model_to_vals(self):
        if self.alpha >= 1000:
            alphanum = self.alpha
            alphadenom = 1
        elif self.alpha > 1:
            alphanum = self.alpha * 1000
            alphadenom = 1000
        else:
            alphanum = self.alpha * 1000000
            alphadenom = 1000000
        model_vals = [self.class_num,
                      len(self.categories),
                      self.clusters_per_node,
                      self.node_num,
                      self.layer_num,
                      int(alphanum),
                      alphadenom]
        for l in self.layers:
            for n in l:
                model_vals += n.model_to_vals()
        model_vals += self.classifier.model_to_vals()
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
    def model_from_lines(model_lines):
        class_num = int(model_lines[0])
        categories = list(map(int, model_lines[1].split(" ")))
        clusters_per_node = int(model_lines[2])
        node_num = int(model_lines[3])
        layer_num = int(model_lines[4])
        alpha = int(model_lines[5])
        model = KNetworkBayes(class_num, categories, clusters_per_node, node_num, layer_num, alpha)
        at = 6
        lines_per_km = 2 + clusters_per_node
        for l in range(0, layer_num):
            for n in range(0, node_num):
                model.layers[l][n] = km.KModes.model_from_lines(model_lines[at:])
                at += lines_per_km
        model.classifier = cnb.CategoricalNaiveBayes.model_from_lines(model_lines[at:])
        return model

    @staticmethod
    def model_from_vals(model_vals):
        class_num = model_vals[0]
        cat_num = model_vals[1]
        clusters_per_node = model_vals[2]
        node_num = model_vals[3]
        layer_num = model_vals[4]
        alpha = float(model_vals[5])/model_vals[6]
        categories = model_vals[7:7 + cat_num]
        at = 7 + cat_num

        model = KNetworkBayes(class_num, categories, clusters_per_node, node_num, layer_num, alpha)
        netcatnum = cat_num
        for l in range(0, layer_num):
            vals_per_km = 2 + clusters_per_node * netcatnum + netcatnum
            for n in range(0, node_num):
                model.layers[l][n] = km.KModes.model_from_vals(model_vals[at:at + vals_per_km])
                at += vals_per_km
            netcatnum = node_num
        model.classifier = cnb.CategoricalNaiveBayes.model_from_vals(model_vals[at:])
        return model

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = KNetworkBayes.model_from_vals(model_vals)
        file.close()
        print("Loaded Categorical Bayes Classifier")
        return model


    @staticmethod
    def load_model(file_name):
        file = open(file_name, "r")
        model_lines = file.readlines()
        model = KNetworkBayes.model_from_lines(model_lines)
        file.close()
        print("Loaded K Network Bayes Classifier")
        return model

