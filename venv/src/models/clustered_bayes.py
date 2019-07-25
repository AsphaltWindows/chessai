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

        self.classifiers = [cnb.CategoricalNaiveBayes(self.cat_num, self.classes, self.alpha)]

        bucketed_data = [[] for cl in range(0, cluster_num)]

        for didx, data in enumerate(batch_data):
            data[self.clustering.data_labels[didx]].append(data)

        for cidx, bucket in enumerate(bucketed_data):
            self.classifiers[cidx].train_batch(bucket)

    def predict_cat(self, data):
        predictions = []
        for cidx in self.clustering.assign_cluster(data):
            predictions.append(self.classifiers[cidx].predict_cat(data))
        num_preds = len(predictions)
        normalized = [0 for n in range(0, num_preds)]
        for pred in predictions:
            total = sum(pred)
            for pidx, prob in enumerate(pred):
                normalized[pidx] += prob / (total * float(num_preds))
        return normalized

    def store_model(self, file_name):
        return None


def categorical_naive_bayes(file_name):
    return None

