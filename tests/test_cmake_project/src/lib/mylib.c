#include <stdint.h>

#include "test_project/lib/otherfile.h"
#include "test_project/utils/utils.h"

uint64_t do_some_library_fn_thing(uint32_t blah) {
    uint32_t c = some_other_fn(blah);
    return c * some_utils_fn(blah);
}