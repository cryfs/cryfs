# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.205.2/containers/python-3/.devcontainer/base.Dockerfile

# [Choice] Python version (use -bullseye variants on local arm64/Apple Silicon): 3, 3.10, 3.9, 3.8, 3.7, 3.6, 3-bullseye, 3.10-bullseye, 3.9-bullseye, 3.8-bullseye, 3.7-bullseye, 3.6-bullseye, 3-buster, 3.10-buster, 3.9-buster, 3.8-buster, 3.7-buster, 3.6-buster
ARG VARIANT="3.10-bullseye"
FROM mcr.microsoft.com/vscode/devcontainers/python:0-${VARIANT}

# [Choice] Node.js version: none, lts/*, 16, 14, 12, 10
ARG NODE_VERSION="none"
RUN if [ "${NODE_VERSION}" != "none" ]; then su vscode -c "umask 0002 && . /usr/local/share/nvm/nvm.sh && nvm install ${NODE_VERSION} 2>&1"; fi

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
  && apt-get -y install lsb-release wget software-properties-common \
  && apt-get -y install build-essential ninja-build valgrind gdb lcov doxygen graphviz

# Install minimum cmake 3.21.7 so we can test with the minimukm required version
RUN apt-get -y install libidn11
RUN wget https://github.com/Kitware/CMake/releases/download/v3.21.7/cmake-3.21.7-Linux-x86_64.sh \
  -q -O /tmp/cmake-install.sh \
  && chmod u+x /tmp/cmake-install.sh \
  && /tmp/cmake-install.sh --skip-license --prefix=/usr \
  && rm /tmp/cmake-install.sh

# Install clang-14
RUN wget -qO - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN add-apt-repository 'deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-14 main'
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
  && apt-get -y install clang-14 clangd-14 lldb-14 llvm-14 clang-format-14 clang-tidy-14

RUN update-alternatives --install /usr/bin/clang clang /usr/bin/clang-14 100
RUN update-alternatives --install /usr/bin/clang++ clang++ /usr/bin/clang++-14 100
RUN update-alternatives --install /usr/bin/clang-apply-replacements clang-apply-replacements /usr/bin/clang-apply-replacements-14 100
RUN update-alternatives --install /usr/bin/clang-check clang-check /usr/bin/clang-check-14 100
RUN update-alternatives --install /usr/bin/clang-query clang-query /usr/bin/clang-query-14 100
RUN update-alternatives --install /usr/bin/clang-tidy clang-tidy /usr/bin/clang-tidy-14 100
RUN update-alternatives --install /usr/bin/clang-format clang-format /usr/bin/clang-format-14 100
RUN update-alternatives --install /usr/bin/scan-build scan-build /usr/bin/scan-build-14 100
RUN update-alternatives --install /usr/bin/scan-view scan-view /usr/bin/scan-view-14 100
RUN update-alternatives --install /usr/bin/llvm-cov llvm-cov /usr/bin/llvm-cov-14 100
RUN update-alternatives --install /usr/bin/llvm-profdata llvm-profdata /usr/bin/llvm-profdata-14 100

# Install global node packages and enable husky.
RUN su vscode -c "source /usr/local/share/nvm/nvm.sh && npm install -g husky standard-version @commitlint/cli @commitlint/config-conventional " 2>&1

# Clean-up
RUN apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Setup oh-my-zsh and plugins for user vscode
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID
COPY --chown=$USER_UID:$USER_GID .devcontainer/.zshrc /home/$USERNAME
RUN chmod 644 /home/$USERNAME/.zshrc
# the user we're applying this too (otherwise it most likely install for root)
USER $USERNAME
RUN git clone https://github.com/romkatv/powerlevel10k.git ~/.oh-my-zsh/custom/themes/powerlevel10k
RUN git clone https://github.com/zsh-users/zsh-autosuggestions ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-autosuggestions
RUN git clone https://github.com/zsh-users/zsh-syntax-highlighting.git ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-syntax-highlighting
ENV SHELL=/bin/zsh
