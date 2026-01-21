project("bots" LANGUAGES CXX)

include(FetchContent)
# fetch shakkar/fast-reachability repo
FetchContent_Declare(
	fast_reachability
	GIT_REPOSITORY https://github.com/shakkar23/fast-reachability.git
	GIT_TAG 5394322bc459a0565731eadb1f939bccc64b8b61
)
	
FetchContent_MakeAvailable(fast_reachability Fast_Reachability)

add_library(bot_downstacker "downstacker.cpp")
set_property(TARGET bot_downstacker PROPERTY CXX_STANDARD 23)
target_link_libraries(bot_downstacker  Fast_Reachability)

target_include_directories(bot_downstacker INTERFACE
$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
$<INSTALL_INTERFACE:include>
)
add_executable(bot_tester "test/test.cpp")

target_link_libraries(bot_tester PRIVATE bot_downstacker)
set_property(TARGET bot_tester PROPERTY CXX_STANDARD 23)
