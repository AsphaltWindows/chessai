class CategoricalNaiveBayes:
    def __init__(self, cat_num, classes, alpha=1):
        self.categories = [c for c in range(0, cat_num)]
        self.classes = classes
        self.alpha = alpha

        self.cat_totals = [0 for n in range(0, cat_num)]
        self.cat_probabilities = [0 for n in range(0, cat_num)]

        self.cat_class_totals = [[[0 for n in range(0, cl)] for cl in classes] for cat in self.categories]
        self.cat_class_probabilities = [[[0 for n in range(0, cl)] for cl in classes] for cat in self.categories]

    def train_batch(self, batch_data):
        for data in batch_data:
            self.cat_totals[data[0]] += 1
            for idx, cl in enumerate(data[1]):
                self.cat_class_totals[data[0]][idx][cl] += 1

        self.recalculate_probabilities()
        print("Categorical Naive Bayes trained on " + str(len(batch_data)) + " data points")

    def recalculate_probabilities(self):
        for idx, cat in enumerate(self.categories):
            if sum(self.cat_totals) == 0:
                self.cat_probabilities[idx] = 1 / float(len(self.categories))
            else:
                self.cat_probabilities[idx] = self.cat_totals[idx] / float(sum(self.cat_totals))

            for idx2 in range(0, len(self.cat_class_totals[idx])):
                for idx3 in range(0, len(self.cat_class_totals[idx][idx2])):
                    self.cat_class_probabilities[idx][idx2][idx3] = (self.cat_class_totals[idx][idx2][
                                                                         idx3] + self.alpha) / float(
                                                                                sum(self.cat_class_totals[idx][idx2]) +
                                                                                self.classes[idx2])

    def predict_cat(self, data):
        probabilities = [self.cat_probabilities[i] for i in self.categories]

        for cat in self.categories:
            for idx, cl in enumerate(data):
                probabilities[cat] *= self.cat_class_probabilities[cat][idx][cl]

        return probabilities

    def model_to_string(self):
        model_str = str(len(self.categories)) + '\n'
        model_str += " ".join(map(str, self.classes)) + '\n'
        model_str += str(self.alpha) + '\n'
        model_str += " ".join(map(str, self.cat_totals)) + '\n'

        freqs = []
        for cat in self.cat_class_totals:
            for cls in cat:
                for cl in cls:
                    freqs.append(cl)

        model_str += " ".join(map(str, freqs)) + '\n'
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
        cat_totals = list(map(int, model_lines[3].split(" ")))
        model = CategoricalNaiveBayes(cat_num, classes, alpha)
        model.cat_totals = cat_totals
        freqs = list(map(int, model_lines[4].split(" ")))
        at = 0
        for idx1 in range(0, len(model.cat_class_totals)):
            for idx2 in range(0, len(model.cat_class_totals[idx1])):
                for idx3 in range(0, len(model.cat_class_totals[idx1][idx2])):
                    model.cat_class_totals[idx1][idx2][idx3] = freqs[at]
                    at += 1
        model.recalculate_probabilities()
        return model

    @staticmethod
    def load_model(file_name):
        file = open(file_name, "r")
        model_lines = file.readlines()
        model = CategoricalNaiveBayes.model_from_lines(model_lines)
        file.close()
        print("Loaded Categorical Bayes Classifier")
        return model
