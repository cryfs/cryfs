#include "ConsoleTest.h"

TEST_F(ConsoleTest, Print) {
    print("Bla Blub");
    EXPECT_OUTPUT_LINE("Bla Blu", 'b'); // 'b' is the delimiter for reading
}
