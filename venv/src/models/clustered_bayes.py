import models.categorical_naive_bayes as cnb
import models.k_modes as km


class ClusteredBayes:
    def __init__(self, cat_num, classes, alpha = 1):
        self.cat_num = cat_num
        self.classes = classes
        self.alpha = alpha
        self.clustering = km.KModes(classes)
        self.classifiers = []

    def train_batch(self, start_modes, cluster_num, batch_data):

        self.clustering.train_batch(start_modes, cluster_num, [data[1] for data in batch_data])

        self.classifiers = [cnb.CategoricalNaiveBayes(self.cat_num, self.classes, self.alpha) for cl in range(0, cluster_num)]

        bucketed_data = [[] for cl in range(0, cluster_num)]

        for didx, data in enumerate(batch_data):
            bucketed_data[self.clustering.data_labels[didx]].append(data)

        for cidx, bucket in enumerate(bucketed_data):
            self.classifiers[cidx].train_batch(bucket)

    def predict_cat(self, data):
        predictions = []
        for cidx in self.clustering.assign_cluster(data)[0]:
            predictions.append(self.classifiers[cidx].predict_cat(data))
        num_preds = len(predictions)
        normalized = [0 for n in range(0, self.cat_num)]
        for pred in predictions:
            total = sum(pred)
            for pidx, prob in enumerate(pred):
                normalized[pidx] += prob / (total * float(num_preds))
        return normalized

    def model_to_string(self):
        model_str = str(self.cat_num) + '\n'
        model_str += " ".join(map(str, self.classes)) + '\n'
        model_str += str(self.alpha) + '\n'
        model_str += self.clustering.model_to_string()
        for c in self.classifiers:
            model_str += c.model_to_string()
        return model_str

    def store_model(self, file_name):
        file = open(file_name, "w")
        file.write(self.model_to_string())
        file.close()

    @staticmethod
    def model_from_lines(model_lines):
        cat_num = int(model_lines[0])
        classes = list(map(int, model_lines[1].split(" ")))
        alpha = int(model_lines[2])
        model = ClusteredBayes(cat_num, classes, alpha)
        model.clustering = km.KModes.model_from_lines(model_lines[3:])
        cluster_num = model.clustering.cluster_num
        start_at = 5 + cluster_num
        for cl in range(0, cluster_num):
            model.classifiers.append(cnb.CategoricalNaiveBayes.model_from_lines(model_lines[start_at:]))
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

