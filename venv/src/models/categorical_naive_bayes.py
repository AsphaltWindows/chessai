class CategoricalNaiveBayes:
    def __init__(self, cat_num, classes, alpha = 1):
        self.categories = list(range(0, cat_num))
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
        for idx, c in enumerate(self.categories):
            self.cat_probabilities[idx] = self.cat_totals[idx] / float(sum(self.cat_totals))

            for idx2 in range(0, len(self.cat_class_totals[idx])):
                for idx3 in range(0, len(self.cat_class_totals[idx][idx2])):
                    self.cat_class_probabilities[idx][idx2][idx3] = (self.cat_class_totals[idx][idx2][idx3] + self.alpha) / float(sum(self.cat_class_totals[idx][idx2]) + self.classes[idx2])

    def predict_cat(self, data):
        probabilities = [self.cat_probabilities[i] for i in self.categories]

        for cat in self.categories:
            for idx, cl in enumerate(data):
                probabilities[cat] *= self.cat_class_probabilities[cat][idx][cl]

        return probabilities

    def store_model(self, file_name):
        file = open(file_name, "w")
        file.write(str(len(self.categories)) + '\n')
        file.write(" ".join(map(str, self.classes)) + '\n')
        file.write(" ".join(map(str, self.cat_totals)) + '\n')
        file.write(str(self.alpha) + '\n')

        freqs = []

        for cat in self.cat_class_totals:
            for cls in cat:
                for cl in cls:
                    freqs.append(cl)

        file.write(" ".join(map(str, freqs)) + '\n')
        file.close()


def categorical_naive_bayes(file_name):
    file = open(file_name, "r")
    fmodel = file.readlines()
    cat_num = int(fmodel[0])
    classes = map(int, fmodel[1].split(" "))
    cat_totals = map(int, fmodel[2].split(" "))
    alpha = int(fmodel[3])
    model = CategoricalNaiveBayes(cat_num, classes, alpha)
    model.cat_totals = cat_totals
    freqs = map(int, fmodel[4].split(" "))
    at = 0
    for idx1 in range(0, len(model.cat_class_totals)):
        for idx2 in range(0, len(model.cat_class_totals[idx1])):
            for idx3 in range(0, len(model.cat_class_totals[idx1][idx2])):
                model.cat_class_totals[idx1][idx2][idx3] = freqs[at]
                at += 1
    model.recalculate_probabilities()
    file.close()
    return model

