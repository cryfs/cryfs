#include <cpp-utils/thread/debugging.h>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/lock/ConditionBarrier.h>
#include <gtest/gtest.h>

using namespace cpputils;
using std::string;

TEST(ThreadDebuggingTest_ThreadName, givenMainThread_whenSettingAndGetting_thenDoesntCrash) {
	set_thread_name("my_thread_name");
	get_thread_name();
}

TEST(ThreadDebuggingTest_ThreadName, givenChildThread_whenSettingAndGetting_thenDoesntCrash) {
    ConditionBarrier nameIsChecked;

	bool child_didnt_crash = false;
	std::thread child([&] {
		set_thread_name("my_thread_name");
		get_thread_name();
		child_didnt_crash = true;
		nameIsChecked.wait();
	});
	get_thread_name(&child);
	nameIsChecked.release(); // getting the name of a not-running thread would cause errors, so let's make sure we only exit after getting the name
	child.join();
	EXPECT_TRUE(child_didnt_crash);
}

TEST(ThreadDebuggingTest_ThreadName, givenMainThread_whenGettingFromInside_thenIsCorrect) {
    set_thread_name("my_thread_name");
    string name = get_thread_name();
    EXPECT_EQ("my_thread_name", name);
}

TEST(ThreadDebuggingTest_ThreadName, givenChildThread_whenGettingFromInside_thenIsCorrect) {
  std::thread child([] {
    set_thread_name("my_thread_name");
    string name = get_thread_name();
    EXPECT_EQ("my_thread_name", name);
  });
  child.join();
}

TEST(ThreadDebuggingTest_ThreadName, givenChildThread_whenGettingFromOutside_thenIsCorrect) {
  ConditionBarrier nameIsSet;
  ConditionBarrier nameIsChecked;

  std::thread child([&] {
    set_thread_name("my_thread_name");
    nameIsSet.release();
    nameIsChecked.wait();
  });

  nameIsSet.wait();
  set_thread_name("outer_thread_name"); // just to make sure the next line doesn't read the outer thread name
  string name = get_thread_name(&child);
  EXPECT_EQ("my_thread_name", name);

  nameIsChecked.release();
  child.join();
}

