#!/bin/sh
(
   export PATH="/home/travis/bin:/home/travis/.local/bin:/home/travis/.gimme/versions/go1.4.2.linux.amd64/bin:/home/travis/.rvm/gems/ruby-1.9.3-p551/bin:/home/travis/.rvm/gems/ruby-1.9.3-p551@global/bin:/home/travis/.rvm/rubies/ruby-1.9.3-p551/bin::/usr/local/phantomjs/bin:/home/travis/.nvm/v0.10.36/bin:./node_modules/.bin:/usr/local/maven-3.2.5/bin:/usr/local/clang-3.4/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/travis/.rvm/bin"
   set -e
   insmod /usr/lib/uml/modules/`uname -r`/kernel/fs/fuse/fuse.ko
   cd "/home/travis/build/cryfs/cryfs"
   ./bin/messmer_cryfs_test_main
)
echo "$?" > "/home/travis/build/cryfs/cryfs"/umltest.status
halt -f
