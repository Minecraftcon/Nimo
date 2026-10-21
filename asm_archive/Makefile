AS = as
LD = ld
ASFLAGS = --64
LDFLAGS = -static

SRC = $(wildcard src/*.s)
OBJ = $(patsubst src/%.s, build/%.o, $(SRC))
TARGET = nimo

all: $(TARGET)

build/%.o: src/%.s
	@mkdir -p build
	$(AS) $(ASFLAGS) $< -o $@

$(TARGET): $(OBJ)
	$(LD) $(LDFLAGS) $(OBJ) -o $@
	@echo "Built $(TARGET) successfully! Run with ./$(TARGET) [filename]"

clean:
	rm -rf build $(TARGET)

.PHONY: all clean
