import math

class CategoricalNaiveBayes:
    def __init__(self, class_num, categories, alpha=1):
        self.classes = [c for c in range(0, class_num)]
        self.categories = categories
        self.alpha = alpha

        self.class_totals = [0 for n in range(0, class_num)]
        self.class_probabilities = [0 for n in range(0, class_num)]

        self.class_category_totals = [[[0 for n in range(0, cat)] for cat in categories] for cl in self.classes]
        self.class_category_probabilities = [[[0 for n in range(0, cat)] for cat in categories] for cl in self.classes]

    def train_batch(self, labels, batch_data):
        for didx, data in enumerate(batch_data):
            self.class_totals[labels[didx]] += 1
            for idx, val in enumerate(data):
                self.class_category_totals[labels[didx]][idx][val] += 1

        self.recalculate_probabilities()
        print("Categorical Naive Bayes trained on " + str(len(batch_data)) + " data points")

    def recalculate_probabilities(self):
        for idx, cl in enumerate(self.classes):
            if sum(self.class_totals) == 0:
                self.class_probabilities[idx] = -1 * math.log(float(len(self.classes)))
            else:
                self.class_probabilities[idx] = math.log(self.class_totals[idx]) - math.log(float(sum(self.class_totals)))

            for idx2 in range(0, len(self.class_category_totals[idx])):
                for idx3 in range(0, len(self.class_category_totals[idx][idx2])):
                    self.class_category_probabilities[idx][idx2][idx3] = math.log(
                        self.class_category_totals[idx][idx2][idx3] +
                        self.alpha) -\
                        math.log(
                            float(
                                sum(self.class_category_totals[idx][idx2]) +
                                self.categories[idx2])
                        )

    def predict_class(self, data):
        probabilities = [self.class_probabilities[i] for i in self.classes]

        for cl in self.classes:
            for idx, cat in enumerate(data):
                probabilities[cl] += self.class_category_probabilities[cl][idx][cat]

        return probabilities

    def model_val_num(self):
        return 4 + len(self.categories) + len(self.classes) + (len(self.classes) * sum(self.categories))

    def model_to_vals(self):
        model_vals = [len(self.classes)]
        if self.alpha >= 1000:
            alphanum = self.alpha
            alphadenom = 1
        elif self.alpha > 1:
            alphanum = self.alpha * 1000
            alphadenom = 1000
        else:
            alphanum = self.alpha * 1000000
            alphadenom = 1000000
        model_vals += [int(alphanum), alphadenom, len(self.categories)]
        model_vals += self.categories
        model_vals += self.class_totals
        for cl in range(0, len(self.classes)):
            for cat in range(0, len(self.categories)):
                for val in range(0, self.categories[cat]):
                    model_vals.append(self.class_category_totals[cl][cat][val])

        return model_vals

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

    def store_model2(self, file_name):
        file = open(file_name, "w")
        file.write("\n".join(map(str, self.model_to_vals())))
        file.close()

    @staticmethod
    def model_from_vals(model_vals):
        class_num = int(model_vals[0])
        alpha = float(model_vals[1])/ model_vals[2]
        cat_num = model_vals[3]
        print(cat_num)
        cats = model_vals[4:4 + cat_num]
        model = CategoricalNaiveBayes(class_num, cats, alpha)
        class_totals = model_vals[4 + cat_num:4 + cat_num + class_num]
        model.class_totals = class_totals
        at = 4 + cat_num + class_num
        for cl in range(0, class_num):
            for cat in range(0, cat_num):
                for val in range(0, cats[cat]):
                    model.class_category_totals[cl][cat][val] = model_vals[at]
                    at += 1
        model.recalculate_probabilities()
        return model

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

    @staticmethod
    def load_model2(file_name):
        file = open(file_name, "r")
        model_vals = list(map(int, file.readlines()))
        model = CategoricalNaiveBayes.model_from_vals(model_vals)
        file.close()
        print("Loaded Categorical Bayes Classifier")
        return model
