
all: Shaders.metallib

Shaders.metallib: shaders/Shaders.metal
	@xcrun -sdk macosx metal  \
	    -frecord-sources=flat \
	    shaders/Shaders.metal \
	    -o Shaders.metallib

clean:
	@rm -fv Shaders.metallib Shaders.metallibsym

.PHONY: all clean
