import tensorflow as tf
import models.bdt_c as bdt
import models.cnb_c as cnb

(x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()

print("WTF IS THIS")

x_train_list = []
y_train_list = []

x_test_list = []
y_test_list = []

for idx, x in enumerate(x_train):
    x_train_list.append([])
    for d in x:

        if d > 127:
            rounded = 1
        else:
            rounded = 0

        x_train_list[idx].append(rounded)

for y in enumerate(y_train):
    y_train_list.append(y)

for idx, x in enumerate(x_test):
    x_test_list.append([])
    for d in x:

        if d > 127:
            rounded = 1
        else:
            rounded = 0

        x_test_list[idx].append(rounded)

for y in enumerate(y_test):
    y_test_list.append(y)


print(len(x_train))
print(len(y_train))


cnb_model = cnb.CNB_C(10, [2 for _ in range(28*28)] )
bdt_model = bdt.BDT_C([2 for _ in range(28*28)], 10, 2, 500, 200, 2, 1, 0)

print("WTF 2")
