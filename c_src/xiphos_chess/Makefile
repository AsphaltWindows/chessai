CC = clang
CFLAGS = -Wall -Wextra -O3 -shared -fpic -o libxiphoschess.so
LIBS = -lm

TARGET = libxiphoschess
SRCS = *.c

all: sse bmi2 nopopcnt

sse:
	$(CC) $(CFLAGS) -msse $(SRCS) -o $(TARGET)-sse.so $(LIBS)

bmi2:
	$(CC) $(CFLAGS) -D_BMI2 -mbmi2 $(SRCS) -o $(TARGET)-bmi2.so $(LIBS)

nopopcnt:
	$(CC) $(CFLAGS) -D_NOPOPCNT $(SRCS) -o $(TARGET)-nopopcnt.so $(LIBS)

clean:
	rm $(TARGET)-*
