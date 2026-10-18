#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

/*
 * test_freetype_client.c
 * Standalone C client verifying drop-in FreeType 2 ABI compatibility.
 * Links directly against libhoney_shim to prove low-level OS replacement.
 */

typedef struct FT_Bitmap_ {
    unsigned int rows;
    unsigned int width;
    int pitch;
    unsigned char* buffer;
    unsigned short num_grays;
    unsigned char pixel_mode;
    unsigned char palette_mode;
    void* palette;
} FT_Bitmap;

typedef struct FT_GlyphSlotRec_* FT_GlyphSlot;
typedef struct FT_FaceRec_* FT_Face;

typedef struct FT_GlyphSlotRec_ {
    void* library;
    FT_Face face;
    FT_GlyphSlot next;
    unsigned int glyph_index;
    size_t generic[2];
    long metrics[8];
    long linearHoriAdvance;
    long linearVertAdvance;
    long advance_x;
    long advance_y;
    unsigned int format;
    FT_Bitmap bitmap;
    int bitmap_left;
    int bitmap_top;
} FT_GlyphSlotRec;

typedef struct FT_FaceRec_ {
    long num_faces;
    long face_index;
    long face_flags;
    long style_flags;
    long num_glyphs;
    char* family_name;
    char* style_name;
    int num_fixed_sizes;
    void* available_sizes;
    int num_charmaps;
    void* charmaps;
    size_t generic[2];
    long bbox[4];
    unsigned short units_per_EM;
    short ascender;
    short descender;
    short height;
    short max_advance_width;
    short max_advance_height;
    short underline_position;
    short underline_thickness;
    FT_GlyphSlot glyph;
} FT_FaceRec;

/* FreeType ABI Function Declarations provided by libhoney_shim */
extern int FT_Init_FreeType(void** alibrary);
extern int FT_Done_FreeType(void* library);
extern int FT_New_Face(void* library, const char* filepathname, long face_index, FT_Face* aface);
extern int FT_Done_Face(FT_Face face);
extern int FT_Set_Pixel_Sizes(FT_Face face, unsigned int pixel_width, unsigned int pixel_height);
extern unsigned int FT_Get_Char_Index(FT_Face face, unsigned long charcode);
extern int FT_Load_Glyph(FT_Face face, unsigned int glyph_index, int load_flags);
extern int FT_Render_Glyph(FT_GlyphSlot slot, unsigned int render_mode);
extern int FT_Load_Char(FT_Face face, unsigned long char_code, int load_flags);

int main(int argc, char* argv[]) {
    printf("=================================================================\n");
    printf("  Honey Rust Shim: FreeType 2 C ABI Drop-in Client Test          \n");
    printf("  Verifying low-level OS FreeType substitution without patches   \n");
    printf("=================================================================\n\n");

    const char* font_path = "/System/Library/Fonts/Menlo.ttc";
    if (argc > 1) {
        font_path = argv[1];
    }

    void* library = NULL;
    int err = FT_Init_FreeType(&library);
    if (err != 0) {
        fprintf(stderr, "Error: FT_Init_FreeType failed with code %d\n", err);
        return 1;
    }
    printf("[1] FT_Init_FreeType: OK (library handle = %p)\n", library);

    FT_Face face = NULL;
    err = FT_New_Face(library, font_path, 0, &face);
    if (err != 0) {
        printf("Note: Default font '%s' not accessible (code %d). Trying alternative...\n", font_path, err);
        font_path = "/System/Library/Fonts/Geneva.dfont";
        err = FT_New_Face(library, font_path, 0, &face);
        if (err != 0) {
            font_path = "/Library/Fonts/Arial.ttf";
            err = FT_New_Face(library, font_path, 0, &face);
        }
    }

    if (err != 0 || face == NULL) {
        fprintf(stderr, "Error: FT_New_Face could not load font file. (error %d)\n", err);
        FT_Done_FreeType(library);
        return 1;
    }

    printf("[2] FT_New_Face: Loaded '%s'\n", font_path);
    printf("    Num Glyphs: %ld | Units per EM: %d | Ascender: %d | Descender: %d\n",
           face->num_glyphs, face->units_per_EM, face->ascender, face->descender);

    unsigned int target_size = 24;
    FT_Set_Pixel_Sizes(face, 0, target_size);
    printf("[3] FT_Set_Pixel_Sizes: Set to %u px\n", target_size);

    char test_char = 'e';
    err = FT_Load_Char(face, test_char, 0);
    if (err != 0) {
        fprintf(stderr, "Error: FT_Load_Char failed for '%c' (code %d)\n", test_char, err);
        FT_Done_Face(face);
        FT_Done_FreeType(library);
        return 1;
    }

    FT_GlyphSlot slot = face->glyph;
    err = FT_Render_Glyph(slot, 0);
    if (err != 0) {
        fprintf(stderr, "Error: FT_Render_Glyph failed (code %d)\n", err);
        FT_Done_Face(face);
        FT_Done_FreeType(library);
        return 1;
    }

    printf("[4] FT_Render_Glyph: Glyph '%c' rendered successfully by Honey Rust Core!\n", test_char);
    printf("    Bitmap Width: %u px | Rows: %u px | Pitch: %d\n",
           slot->bitmap.width, slot->bitmap.rows, slot->bitmap.pitch);
    printf("    Bearing Left: %d px | Bearing Top: %d px | Advance X: %.2f px\n\n",
           slot->bitmap_left, slot->bitmap_top, (float)slot->advance_x / 64.0f);

    /* Render ASCII loupe */
    printf("[5] Visual Loupe of Rendered Glyph (Pure Grayscale Antialiased):\n");
    const char* shade = " .:-=+*#%@";
    uint64_t total_energy = 0;

    for (unsigned int r = 0; r < slot->bitmap.rows; r++) {
        printf("    %02u | ", r);
        for (unsigned int c = 0; c < slot->bitmap.width; c++) {
            uint8_t val = slot->bitmap.buffer[r * slot->bitmap.pitch + c];
            total_energy += val;
            int shade_idx = (val * 9) / 255;
            printf("%c%c", shade[shade_idx], shade[shade_idx]);
        }
        printf("\n");
    }

    printf("\nTotal Glyph Ink Energy: %llu units\n", (unsigned long long)total_energy);
    printf("Pixel Mode: %d (FT_PIXEL_MODE_GRAY = 256 levels)\n", slot->bitmap.pixel_mode);

    FT_Done_Face(face);
    FT_Done_FreeType(library);
    printf("\n[6] Cleanup: FT_Done_Face & FT_Done_FreeType OK.\n");
    printf("All C ABI checks passed: Honey successfully acts as a FreeType replacement.\n");
    return 0;
}
