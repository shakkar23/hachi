project("bots" LANGUAGES CXX)

include(FetchContent)
# fetch shakkar/fast-reachability repo
FetchContent_Declare(
	fast_reachability
	GIT_REPOSITORY https://github.com/shakkar23/fast-reachability.git
	GIT_TAG 7285c5741648ce09e81fb8b7b66142258834e05c
)
	
FetchContent_MakeAvailable(fast_reachability Fast_Reachability)

# Board-only inputs for the mixture-of-experts router.
add_library(bot_features "feature_extractor.cpp")
target_compile_features(bot_features PUBLIC cxx_std_23)
target_link_libraries(bot_features PUBLIC Fast_Reachability)
target_include_directories(bot_features PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

add_executable(feature_extractor_test "test/feature_extractor_test.cpp")
target_link_libraries(feature_extractor_test PRIVATE bot_features)

# DOWN STACKER EXPERT
add_library(bot_downstacker "downstacker.cpp")
set_property(TARGET bot_downstacker PROPERTY CXX_STANDARD 23)
target_link_libraries(bot_downstacker  Fast_Reachability)

target_include_directories(bot_downstacker INTERFACE
$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
$<INSTALL_INTERFACE:include>
)


add_library(movegen "util/movegen.cpp")
set_property(TARGET movegen PROPERTY CXX_STANDARD 23)
target_link_libraries(movegen  Fast_Reachability)


# COMBO EXPERT
add_library(bot_comboer "comboer.cpp")
set_property(TARGET bot_comboer PROPERTY CXX_STANDARD 23)
target_link_libraries(bot_comboer  movegen)

target_include_directories(bot_comboer INTERFACE
$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
$<INSTALL_INTERFACE:include>
)
target_compile_options(bot_comboer PUBLIC "-fsanitize=undefined")
target_link_options(bot_comboer PUBLIC -fsanitize=undefined)

# T SPIN EXPERT
add_library(bot_spinner "spinner.cpp")
set_property(TARGET bot_spinner PROPERTY CXX_STANDARD 23)
target_link_libraries(bot_spinner  movegen)

target_include_directories(bot_spinner INTERFACE
$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
$<INSTALL_INTERFACE:include>
)
target_compile_options(bot_spinner PUBLIC "-fsanitize=undefined")
target_link_options(bot_spinner PUBLIC -fsanitize=undefined)


target_include_directories(movegen INTERFACE
$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/util>
$<INSTALL_INTERFACE:include>
)
add_executable(bot_tester "test/test.cpp")

target_link_libraries(bot_tester PRIVATE bot_downstacker bot_comboer)
set_property(TARGET bot_tester PROPERTY CXX_STANDARD 23)
#add_compile_options(TARGET bot_tester PRIVATE -march=native -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer)
target_compile_options(bot_tester PUBLIC "-fsanitize=undefined")
target_link_options(bot_tester PUBLIC -fsanitize=undefined)
