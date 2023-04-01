import tensorflow as tf
import numpy as np
import models.bdt_c as bdt
import models.cnbp_c as cnbp
import models.bcc_c as bcc
import models.cnbp_chain_c as cnbp_chain
import models.cnbp_sc_c as cnbp_sc

(x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()

print(len(x_train))
print(len(x_test))

print("Creating models")

print("Creating CNBP_CHAIN")
cnbp_chain_model = cnbp_chain.CNBP_CHAIN_C(10, [2 for _ in range(28*28)], 5)

print("Creating CNBP_SCATTER_CHAIN")
cnbp_sc_model = cnbp_sc.CNBP_SC_C(10, [2 for _ in range(28*28)], 10)

print("Creating CNBP")
cnbp_model = cnbp.CNBP_C(10, [2 for _ in range(28*28)])

print("Creating BCC")
bcc_model = bcc.BCC_C(10, [2 for _ in range(28*28)])

print("Creating cnbp meta")
cnbp_meta_model = cnbp.CNBP_C(2, [2 for _ in range(28*28)])

print("Creating BCC meta")
bcc_meta_model = bcc.BCC_C(2, [2 for _ in range(28*28)])

# print("Creating BDT")
# bdt_model = bdt.BDT_C([2 for _ in range(28*28)], 10, 2, 5000, 15, 1, 1, 0)

print("Training models")

for time in range(1):
    print("training " + str(time))
    for idx in range(len(x_test)):
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
        # bdt_model.train_batch([label], [input])
        cnbp_chain_model.train_batch([label], [input])
        cnbp_sc_model.train_batch([label], [input])
        bcc_model.train_batch([label], [input])

print("Training meta model")
result_labels = []
for time in range(1):
    print("training " + str(time))
    for idx in range(len(x_test)):
        input = []
        for d1 in x_train[idx]:
            for d2 in d1:
                if d2 > 125:
                    rounded = 1
                else:
                    rounded = 0
                input.append(rounded)
        output_cnbp = cnbp_model.predict_class(input)
        label = []
        if np.argmax(output_cnbp) == y_train[idx]:
            label = [0, 1]
        else:
            label = [1, 0]
        cnbp_meta_model.train_batch([label], [input])

        output_bcc = bcc_model.predict_class(input)
        label = []
        if np.argmax(output_bcc) == y_train[idx]:
            label = [0, 1]
        else:
            label = [1, 0]
        bcc_meta_model.train_batch([label], [input])


cnbp_correct = 0
cnbp_chain_correct = 0
cnbp_sc_correct = 0
bcc_correct = 0
cnbp_meta_correct = 0
bcc_meta_correct = 0
# bdt_correct = 0

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
    output_cnbp_sc = cnbp_sc_model.predict_class(input)

    if np.argmax(output_cnbp_sc) == y_test[idx]:
        cnbp_sc_correct += 1

    output_cnbp_chain = cnbp_chain_model.predict_class(input)

    if np.argmax(output_cnbp_chain) == y_test[idx]:
        cnbp_chain_correct += 1

    output_cnbp = cnbp_model.predict_class(input)
    output_cnbp_idx = 0

    if np.argmax(output_cnbp) == y_test[idx]:
        cnbp_correct += 1
        output_cnbp_idx = 1

    output_bcc = bcc_model.predict_class(input)
    output_bcc_idx = 0

    if np.argmax(output_bcc) == y_test[idx]:
        bcc_correct += 1
        output_bcc_idx = 1

    output_meta_cnbp = cnbp_meta_model.predict_class(input)

    if np.argmax(output_meta_cnbp) == output_cnbp_idx:
        cnbp_meta_correct += 1

    output_meta_bcc = bcc_meta_model.predict_class(input)

    if np.argmax(output_meta_bcc) == output_bcc_idx:
        bcc_meta_correct += 1

    # output_bdt = bdt_model.predict_class(input)
    #
    # if np.argmax(output_bdt) == y_test[idx]:
    #     bdt_correct += 1

print("CNBP_SCATTER_CHAIN correct percentage: " + str(cnbp_sc_correct/len(x_test)))
print("CNBP correct percentage: " + str(cnbp_correct/len(x_test)))
print("CNBP_CHAIN correct percentage: " + str(cnbp_chain_correct/len(x_test)))
print("BCC correct percentage: " + str(bcc_correct/len(x_test)))
print("CNBP meta correct percentage: " + str(cnbp_meta_correct/len(x_test)))
print("BCC meta correct percentage: " + str(bcc_meta_correct/len(x_test)))
# print("BDT correct percentage: " + str(bdt_correct/len(x_test)))

