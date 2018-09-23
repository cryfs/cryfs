#include <gtest/gtest.h>
#include <cpp-utils/process/daemon/daemonize.h>
#include <chrono>
#include <boost/filesystem.hpp>
#include <fstream>
#include <thread>
#include <functional>
#include <fcntl.h>

using namespace cpputils::process;
using boost::none;
using std::function;
namespace bf = boost::filesystem;

class DaemonizeTest : public ::testing::Test {
public:
    static void createFile(const bf::path &path) {
      std::ofstream str(path.native().c_str());
    }

    static void daemonizeWithChildExpect(function<bool ()> childExpectation) {
        PipeFromChild childPipe = daemonize([childExpectation](PipeToParent *pipe) {
           if (childExpectation()) {
               pipe->notifyReady();
           } else {
               pipe->notifyError("Child expectation not fulfilled");
           }
        });

        EXPECT_EQ(none, childPipe.waitForReadyReturnError());
    }

    static bool descriptorIsClosed(int fd) {
        int res = fcntl(fd, F_GETFD);
        return -1 == res && EBADF == errno;
    }
};

TEST_F(DaemonizeTest, ReadySignalSend) {
  PipeFromChild childPipe = daemonize([](PipeToParent *pipe) {
    pipe->notifyReady();
  });

  EXPECT_EQ(none, childPipe.waitForReadyReturnError());
}

TEST_F(DaemonizeTest, WaitsForReadySignal) {
  bf::path markerFile = bf::unique_path(bf::temp_directory_path() / "%%%%-%%%%-%%%%-%%%%");

  PipeFromChild childPipe = daemonize([markerFile](PipeToParent *pipe) {
      std::this_thread::sleep_for(std::chrono::seconds(1));
      createFile(markerFile);
      pipe->notifyReady();
  });

  EXPECT_FALSE(bf::exists(markerFile));
  EXPECT_EQ(none, childPipe.waitForReadyReturnError());
  EXPECT_TRUE(bf::exists(markerFile));

  bf::remove(markerFile);
}

TEST_F(DaemonizeTest, ErrorSend) {
  PipeFromChild childPipe = daemonize([](PipeToParent *pipe) {
     pipe->notifyError("Error message");
  });

  EXPECT_EQ("Error message", childPipe.waitForReadyReturnError().value());
}

TEST_F(DaemonizeTest, Exception) {
    PipeFromChild childPipe = daemonize([](PipeToParent *) {
        throw std::runtime_error("My error message");
    });

    EXPECT_EQ("My error message", childPipe.waitForReadyReturnError().value());
}

TEST_F(DaemonizeTest, ChildExitSuccess) {
    PipeFromChild childPipe = daemonize([](PipeToParent *) {
        exit(EXIT_SUCCESS);
    });

    EXPECT_EQ("Child exited before being ready.", childPipe.waitForReadyReturnError().value());
}

TEST_F(DaemonizeTest, ChildExitFailure) {
    PipeFromChild childPipe = daemonize([](PipeToParent *) {
        exit(EXIT_FAILURE);
    });

    EXPECT_EQ("Child exited before being ready.", childPipe.waitForReadyReturnError().value());
}

TEST_F(DaemonizeTest, ChildAbort) {
    PipeFromChild childPipe = daemonize([](PipeToParent *) {
        abort();
    });

    EXPECT_EQ("Child exited before being ready.", childPipe.waitForReadyReturnError().value());
}

TEST_F(DaemonizeTest, ChildCwdIsRoot) {
    daemonizeWithChildExpect([]() {
        return bf::current_path() == bf::path("/");
    });
}

TEST_F(DaemonizeTest, ChildIsChildProcess) {
    pid_t parent_pid = getpid();

    daemonizeWithChildExpect([parent_pid]() {
        return getpid() != parent_pid && getppid() == parent_pid;
    });
}

TEST_F(DaemonizeTest, ChildHasNewSessionid) {
    pid_t parent_sid = getsid(0);

    daemonizeWithChildExpect([parent_sid]() {
       return getsid(0) != parent_sid;
    });
}

TEST_F(DaemonizeTest, ChildHasStdinClosed) {
    daemonizeWithChildExpect([]() {
        return descriptorIsClosed(STDIN_FILENO);
    });
}

TEST_F(DaemonizeTest, ChildHasStdoutClosed) {
    daemonizeWithChildExpect([]() {
        return descriptorIsClosed(STDOUT_FILENO);
    });
}

TEST_F(DaemonizeTest, ChildHasStderrClosed) {
    daemonizeWithChildExpect([]() {
        return descriptorIsClosed(STDERR_FILENO);
    });
}

TEST_F(DaemonizeTest, ChildHasEmptyUmask) {
    daemonizeWithChildExpect([]() {
        return 0 == (umask(0) & 0777);
    });
}
