#include <stdio.h>
#include "libavutil/murmur3.h"
static void show(const char *label, const uint8_t *h) { printf("%-28s", label); for (int i = 0; i < 16; i++) printf("%02x", h[i]); printf("\n"); }
int main(void) {
    uint8_t in[32], h[16];
    for (int i = 0; i < 32; i++) in[i] = i;
    struct AVMurMur3 *c = av_murmur3_alloc();
    av_murmur3_init(c); av_murmur3_update(c, in, 16); av_murmur3_final(c, h); show("16 bytes in one update", h);
    av_murmur3_init(c); av_murmur3_update(c, in, 15); av_murmur3_update(c, in + 15, 1); av_murmur3_final(c, h); show("15 + 1", h);
    av_murmur3_init(c); av_murmur3_update(c, in, 15); av_murmur3_update(c, in + 15, 1); av_murmur3_update(c, in + 16, 16); av_murmur3_final(c, h); show("15 + 1 + 16 (32 bytes)", h);
    av_murmur3_init(c); av_murmur3_update(c, in, 32); av_murmur3_final(c, h); show("32 bytes in one update", h);
    return 0;
}
