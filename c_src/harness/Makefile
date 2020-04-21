all: libmodel libplayer

libmodel: types.h ../models/cnbp/cnbp.c ../models/bdt/bdt.c model.h model.c
	clang -Wall -Wextra -O3 -shared -fpic -o libmodel.so ../models/cnbp/cnbp.c ../models/bdt/bdt.c model.c -lm

libplayer: types.h model.h model.c player.h player.c
	clang -Wall -Wextra -O3 -shared -fpic -o libplayer.so player.c -lm



