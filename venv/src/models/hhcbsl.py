import models.kh_c as khc
import models.cnb_c as cnbc
import copy as cp


class HierarchicalHistogramClusteredBayesSizeLimited:
    def __init__(self, categories, cluster_num, class_num, limit, alpha=1):
        self.cat_num = len(categories)
        self.cluster_num = cluster_num
        self.categories = categories
        self.alpha = alpha
        self.limit = limit
        self.class_num = class_num

        self.root = HHCB_Node(categories, cluster_num, class_num, [0], limit, 0, 0, alpha)

    def train_model(self, labels, data):
        self.root.train_node(labels, data)

    def predict_class(self, data):
        return self.root.predict_class(data)

    def model_to_vals(self):
        if self.alpha >= 1000:
            alphanum = int(self.alpha)
            alphadenom = 1
        elif self.alpha > 1:
            alphanum = int(self.alpha * 1000)
            alphadenom = 1000
        else:
            alphanum = int(self.alpha * 1000000)
            alphadenom = 1000000
        model_vals = [self.cat_num, self.cluster_num, self.class_num, self.limit, alphanum, alphadenom]
        model_vals += self.categories
        model_vals += self.root.node_to_vals()
        return model_vals

    def store_model2(self, file_name):
        file = open(file_name, "w")
        file.write("\n".join(map(str, self.model_to_vals())))
        file.close()

    @staticmethod
    def model_from_vals(model_vals):
        cat_num = model_vals[0]
        cluster_num = model_vals[1]
        class_num = model_vals[2]
        limit = model_vals[3]
        alpha = float(model_vals[4]) / model_vals[5]
        categories = model_vals[6: 6 + cat_num]
        model = HierarchicalHistogramClusteredBayesSizeLimited(categories, cluster_num, class_num, limit, alpha)
        model.root.free_node()
        model.root = HHCB_Node.node_from_vals(model_vals[6 + cat_num :])
        return model

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = HierarchicalHistogramClusteredBayesSizeLimited.model_from_vals(model_vals)
        file.close()
        return model


class HHCB_Node:
    def __init__(self, categories, cluster_num, class_num, hierarchy, limit, node_type, num_trained, alpha=1):
        self.cat_num = len(categories)
        self.categories = categories
        self.cluster_num = cluster_num
        self.class_num = class_num
        self.alpha = alpha
        self.node_type = node_type
        self.limit = limit
        self.num_trained = num_trained
        self.hierarchy = cp.deepcopy(hierarchy)

        #node_type 0 is leaf
        #node_type 1 is branch
        if self.node_type == 0:
            self.classifier = cnbc.CNB_C(class_num, categories, alpha)
            self.children = []
            self.cluster_model = None
        elif self.node_type > 0:
            self.classifier = None
            self.children = [HHCB_Node(categories, cluster_num, class_num, self.hierarchy + [cl], limit, 0, 0, alpha) for cl in range(0, cluster_num)]
            self.cluster_model = khc.KH_C([], cluster_num, categories)

    def free_node(self):
        if self.node_type == 0:
            if self.classifier is not None:
                self.classifier.free_model()
                self.classifier = None
        else:
            if self.cluster_model is not None:
                self.cluster_model.free_model()
                self.cluster_model = None
            for cidx in range(0, self.cluster_num):
                self.children[cidx].free_node()

    def predict_class(self, data):
        if self.node_type == 0:
            probs = self.classifier.predict_class(data)
            return probs
        else:
            return self.children[self.cluster_model.assign_cluster(data)[0]].predict_class(data)

    def train_node(self, labels, data):
        print("Training node: ", self.hierarchy)

        if self.node_type == 0:
            if self.num_trained >= self.limit:
                print("Leaf-Node is too big, forming a new branch")
                self.classifier.free_model()
                self.classifier = None
                self.node_type = 1
                self.children = [HHCB_Node(self.categories, self.cluster_num, self.class_num, self.hierarchy + [cl], self.limit, 0, 0, self.alpha) for cl in range(0, self.cluster_num)]
                self.cluster_model = khc.KH_C([], self.cluster_num, self.categories)
                self.num_trained = 0
                self.train_node(labels, data)
            else:
                print("Training leaf node classifier on " + str(len(labels)) + " data points")
                self.classifier.train_batch(labels, data)
                self.num_trained += len(labels)
        else:
            if self.num_trained == 0:
                print("Fully training new branch node clustering on " + str(len(labels)) + " data points")
                self.cluster_model.train_full(data)
            else:
                print("Incrementally training existing branch node clustering on " + str(len(labels)) + " data points")
                self.cluster_model.train_incremental(data)
            self.num_trained += len(labels)

            cl_labels = [[] for cl in range(0, self.cluster_num)]
            cl_data = [[] for cl in range(0, self.cluster_num)]

            for didx, d in enumerate(data):
                cluster = self.cluster_model.assign_cluster(d)[0]
                cl_labels[cluster].append(labels[didx])
                cl_data[cluster].append(data[didx])

            for cl in range(0, self.cluster_num):
                self.children[cl].train_node(cl_labels[cl], cl_data[cl])

    def node_to_vals(self):
        if self.alpha >= 1000:
            alphanum = int(self.alpha)
            alphadenom = 1
        elif self.alpha > 1:
            alphanum = int(self.alpha * 1000)
            alphadenom = 1000
        else:
            alphanum = int(self.alpha * 1000000)
            alphadenom = 1000000
        model_vals = [self.cat_num, self.cluster_num, self.class_num, self.limit, self.node_type, self.num_trained, alphanum, alphadenom, len(self.hierarchy)]
        model_vals += self.hierarchy
        model_vals += self.categories

        if self.node_type == 0:
            model_vals += self.classifier.model_to_vals()
        else:
            model_vals += self.cluster_model.model_to_vals()
            for child in self.children:
                model_vals += child.node_to_vals()
        return model_vals

    @staticmethod
    def node_from_vals_recur(model_vals):
        cat_num = model_vals[0]
        cluster_num = model_vals[1]
        class_num = model_vals[2]
        limit = model_vals[3]
        node_type = model_vals[4]
        num_trained = model_vals[5]
        alpha = float(model_vals[6]) / model_vals[7]
        hierarchy_num = model_vals[8]
        hierarchy = model_vals[9: 9 + hierarchy_num]
        categories = model_vals[9 + hierarchy_num: 9 + hierarchy_num + cat_num]
        at = 9 + hierarchy_num + cat_num

        node = HHCB_Node(categories, cluster_num, class_num, hierarchy, limit, node_type, num_trained, alpha)
        node.free_node()

        if node.node_type == 0:
            node.classifier = cnbc.CNB_C.model_from_vals(model_vals[at: at + 4 + cat_num + class_num + (class_num * sum(categories))])
            at = at + 4 + cat_num + class_num + (class_num * sum(categories))
            return node, at
        else:
            node.cluster_model = khc.KH_C.model_from_vals(model_vals[at: at + 2 + cat_num + cluster_num + sum(categories) * cluster_num])
            at = at + 2 + cat_num + cluster_num + sum(categories) * cluster_num
            for cl in range(0, cluster_num):
                node_valsused = HHCB_Node.node_from_vals_recur(model_vals[at:])
                node.children[cl] = node_valsused[0]
                at += node_valsused[1]
            return node, at

    @staticmethod
    def node_from_vals(model_vals):
        return HHCB_Node.node_from_vals_recur(model_vals)[0]
