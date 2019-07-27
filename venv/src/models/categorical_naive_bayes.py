class CategoricalNaiveBayes:
    def __init__(self, class_num, categories, alpha=1):
        self.classes = [c for c in range(0, class_num)]
        self.categories = categories
        self.alpha = alpha

        self.class_totals = [0 for n in range(0, class_num)]
        self.class_probabilities = [0 for n in range(0, class_num)]

        self.class_category_totals = [[[0 for n in range(0, cat)] for cat in categories] for cl in self.classes]
        self.class_category_probabilities = [[[0 for n in range(0, cat)] for cat in categories] for cl in self.classes]

    def train_batch(self, batch_data):
        for data in batch_data:
            self.class_totals[data[0]] += 1
            for idx, cat in enumerate(data[1]):
                self.class_category_totals[data[0]][idx][cat] += 1

        self.recalculate_probabilities()
        print("Categorical Naive Bayes trained on " + str(len(batch_data)) + " data points")

    def recalculate_probabilities(self):
        for idx, cl in enumerate(self.classes):
            if sum(self.class_totals) == 0:
                self.class_probabilities[idx] = 1 / float(len(self.classes))
            else:
                self.class_probabilities[idx] = self.class_totals[idx] / float(sum(self.class_totals))

            for idx2 in range(0, len(self.class_category_totals[idx])):
                for idx3 in range(0, len(self.class_category_totals[idx][idx2])):
                    self.class_category_probabilities[idx][idx2][idx3] = (self.class_category_totals[idx][idx2][
                                                                         idx3] + self.alpha) / float(
                        sum(self.class_category_totals[idx][idx2]) +
                        self.categories[idx2])

    def predict_class(self, data):
        probabilities = [self.class_probabilities[i] for i in self.classes]

        for cl in self.classes:
            for idx, cat in enumerate(data):
                probabilities[cl] *= self.class_category_probabilities[cl][idx][cat]

        return probabilities

    def model_to_string(self):
        model_str = str(len(self.classes)) + '\n'
        model_str += " ".join(map(str, self.categories)) + '\n'
        model_str += str(self.alpha) + '\n'
        model_str += " ".join(map(str, self.class_totals)) + '\n'

        freqs = []
        for cl in self.class_category_totals:
            for cats in cl:
                for cat in cats:
                    freqs.append(cat)

        model_str += " ".join(map(str, freqs)) + '\n'
        return model_str

    def store_model(self, file_name):
        file = open(file_name, "w")
        file.write(self.model_to_string())
        file.close()

    @staticmethod
    def model_from_lines(model_lines):
        class_num = int(model_lines[0])
        cats = list(map(int, model_lines[1].split(" ")))
        alpha = int(model_lines[2])
        cat_totals = list(map(int, model_lines[3].split(" ")))
        model = CategoricalNaiveBayes(class_num, cats, alpha)
        model.class_totals = cat_totals
        freqs = list(map(int, model_lines[4].split(" ")))
        at = 0
        for idx1 in range(0, len(model.class_category_totals)):
            for idx2 in range(0, len(model.class_category_totals[idx1])):
                for idx3 in range(0, len(model.class_category_totals[idx1][idx2])):
                    model.class_category_totals[idx1][idx2][idx3] = freqs[at]
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
