import tensorflow as tf
import numpy as np
import models.bdt_c as bdt
import models.cnbp_c as cnbp

(x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()

print("Creating models")

print("Creating CNBP")
cnbp_model = cnbp.CNBP_C(10, [2 for _ in range(28*28)])

print("Creating BDT")
bdt_model = bdt.BDT_C([2 for _ in range(28*28)], 10, 2, 5000, 15, 1, 1, 0)

print("Training models")

for time in range(2):
    for idx in range(60000):
        input = []
        for d1 in x_train[idx]:
            for d2 in d1:
                if d2 > 125:
                    rounded = 1
                else:
                    rounded = 0
                input.append(rounded)
        label = []
        for labelidx in range(10):
            if y_train[idx] == labelidx:
                label.append(1)
            else:
                label.append(0)

        cnbp_model.train_batch([label], [input])
        bdt_model.train_batch([label], [input])

cnbp_correct = 0
bdt_correct = 0

print("Testing")

for idx in range(len(x_test)):
    input = []
    for d1 in x_test[idx]:
        for d2 in d1:
            if d2 > 125:
                rounded = 1
            else:
                rounded = 0
            input.append(rounded)
    output_cnbp = cnbp_model.predict_class(input)

    if np.argmax(output_cnbp) == y_test[idx]:
        cnbp_correct += 1

    output_bdt = bdt_model.predict_class(input)

    if np.argmax(output_bdt) == y_test[idx]:
        bdt_correct += 1

print("CNBP correct percentage: " + str(cnbp_correct/len(x_test)))
print("BDT correct percentage: " + str(bdt_correct/len(x_test)))

