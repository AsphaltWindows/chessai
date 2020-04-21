import sys

import harness.model as m
import chess.categorical_input as ci

model_type = sys.argv[1]
model_dir = sys.argv[2]
model_version = int(sys.argv[3])
games_dir = sys.argv[4]
model_args = sys.argv[5].split(",")

if model_version == 0:
    model_args = ([3] + [70] + ci.game_classes()) + model_args

print("Loading Model")
print("Model type: " + model_type)
print("Model version: " + str(model_version))
print("Model directory: " + model_dir)
print(str(model_args))

model_object = m.model_c.load_model(
    model_type,
    model_version,
    model_dir,
    model_args)

print("Training Model")
print("Games Directory: " + games_dir)

m.model_c.train_model(
    model_object,
    games_dir)

model_version += 1

print("Storing Model")
print("Model type: " + model_type)
print("Model version: " + str(model_version))
print("Model directory: " + model_dir)

m.model_c.store_model(
    model_object,
    model_type,
    model_version,
    model_dir)

print("Destroying model object")

m.model_c.free_model(model_object)


