all: libtrainmodel libselectmove

libtrainmodel: types.h model.h model.c
	clang -Wall -Wextra -O3 -shared -fpic -o libtrainmodel.so model.c -lm

libselectmove: types.h model.h model.c player.h player.c
	clang -Wall -Wextra -O3 -shared -fpic -o libselectmove.so player.c -lm



