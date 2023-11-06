
add_library(mylib SHARED mylib.c otherfile.c)
target_include_directories(mylib PUBLIC include)
target_link_libraries(mylib PRIVATE utils)
