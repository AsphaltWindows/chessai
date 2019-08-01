import models.categorical_naive_bayes as cnb
import models.k_modes as km
import models.cnb_c as cnbc
import models.km_c as kmc


class ClusteredBayes:
    def __init__(self, start_modes, class_num, cluster_num, categories, alpha=1):
        self.class_num = class_num
        self.cluster_num = cluster_num
        self.categories = categories
        self.alpha = alpha
        self.clustering = km.KModes(start_modes, cluster_num, categories)
        # self.clustering = kmc.KM_C(start_modes, cluster_num, categories)
        self.classifiers = [None for cl in range(0, cluster_num)]

    def train_batch(self, labels, batch_data):

        self.clustering.train_batch(batch_data)

        # self.classifiers = [cnb.CategoricalNaiveBayes(self.class_num, self.categories, self.alpha) for cl in range(0, self.cluster_num)]
        self.classifiers = [cnbc.CNB_C(self.class_num, self.categories, self.alpha) for cl in
                            range(0, self.cluster_num)]

        bucketed_data = [[] for cl in range(0, self.cluster_num)]
        bucketed_labels = [[] for cl in range(0, self.cluster_num)]

        for didx, data in enumerate(batch_data):
            cluster = self.clustering.assign_cluster(data)[0]
            bucketed_data[cluster].append(data)
            bucketed_labels[cluster].append(labels)

        for cidx, bucket in enumerate(bucketed_data):
            self.classifiers[cidx].train_batch(bucketed_labels[cidx], bucket)

    def predict_class(self, data):
        predictions = []
        for cidx in self.clustering.assign_cluster(data)[0]:
            predictions.append(self.classifiers[cidx].predict_class(data))
        num_preds = len(predictions)
        normalized = [0 for n in range(0, self.class_num)]
        for pred in predictions:
            total = sum(pred)
            for pidx, prob in enumerate(pred):
                normalized[pidx] += prob / (total * float(num_preds))
        return normalized

    def model_to_string(self):
        model_str = str(self.class_num) + '\n'
        model_str += " ".join(map(str, self.categories)) + '\n'
        model_str += str(self.alpha) + '\n'
        model_str += self.clustering.model_to_string()
        for c in self.classifiers:
            model_str += c.model_to_string()
        return model_str

    def model_val_num(self):
        return 5 + \
               len(self.categories) + \
               self.clustering.model_val_num() + \
               sum([cl.model_val_num() for cl in self.classifiers])

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
        model_vals = [self.class_num, len(self.categories), self.cluster_num, int(alphanum), alphadenom]
        model_vals += self.categories
        model_vals += self.clustering.model_to_vals()
        for cl in self.classifiers:
            model_vals += cl.model_to_vals()

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
        class_num = model_vals[0]
        cat_num = model_vals[1]
        cluster_num = model_vals[2]
        alpha = float(model_vals[3]) / model_vals[4]
        categories = model_vals[5: 5 + cat_num]
        at = 5 + cat_num
        model = ClusteredBayes([], class_num, cluster_num, categories, alpha)
        clustering = km.KModes.model_from_vals(model_vals[at: at + 2 + cat_num + cat_num * cluster_num])
        # clustering = kmc.KM_C.model_from_vals(model_vals[at: at + 2 + cat_num + cat_num * cluster_num])
        # model.clustering.free_kmodes()
        at += 2 + cat_num + cat_num * cluster_num
        model.clustering = clustering
        vals_per_classifier = 4 + cat_num + class_num + class_num * sum(categories)
        for cl in range(0, cluster_num):
            # model.classifiers[cl] = cnb.CategoricalNaiveBayes.model_from_vals(model_vals[at:at + vals_per_classifier])
            model.classifiers[cl] = cnbc.CNB_C.model_from_vals(model_vals[at:at + vals_per_classifier])
            at += vals_per_classifier
        return model

    @staticmethod
    def model_from_lines(model_lines):
        class_num = int(model_lines[0])
        categories = list(map(int, model_lines[1].split(" ")))
        alpha = int(model_lines[2])
        clustering = km.KModes.model_from_lines(model_lines[3:])
        # clustering = kmc.KM_C.model_from_lines(model_lines[3:])
        cluster_num = clustering.cluster_num
        model = ClusteredBayes([], class_num, cluster_num, categories, alpha)
        model.clustering = clustering
        start_at = 5 + cluster_num
        for cl in range(0, cluster_num):
            # model.classifiers[cl] = cnb.CategoricalNaiveBayes.model_from_lines(model_lines[start_at:])
            model.classifiers[cl] = cnbc.CNB_C.model_from_lines(model_lines[start_at:])
            start_at += 5
        return model

    @staticmethod
    def load_model(file_name):
        file = open(file_name, "r")
        model_lines = file.readlines()
        model = ClusteredBayes.model_from_lines(model_lines)
        file.close()
        print("Loaded Clustered Bayes Classifier")
        return model

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = ClusteredBayes.model_from_vals(model_vals)
        file.close()
        print("Loaded Clustered Bayes Classifier")
        return model
