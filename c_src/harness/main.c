#include "model.h"
#include "player.h"

int main() {
    em_t * model;
    player_t * pl;
    const char * model_type = "bdt";
    const char * game_directory = "/home/iv/dev/chessai/randomplay/bdt10/round5/";
    const char * model_directory = "/home/iv/dev/chessai/randomplay/bdt10/";
    uint32_t model_version = 5;
    const char * args[1] = {
            (const char *) ("0")
    };

    printf("hello.\n");


    model = load_model(
            model_type,
            model_version,
            model_directory,
            args);

    pl = player(
            model,
            "wld_score",
            WHITE);

    train_model(
            model,
            game_directory);

    store_model(
            model,
            model_type,
            model_version + 1,
            model_directory);

}
