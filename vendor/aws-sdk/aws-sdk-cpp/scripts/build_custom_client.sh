#!/bin/bash

# Generate custom client source code under custom-service/ with API description file located at code-generation/api-description/custom-service.
python scripts/generate_sdks.py --pathToApiDefinitions=code-generation/api-descriptions/custom-service --outputLocation custom-service --serviceName custom-service --apiVersion 2017-11-03 --namespace Custom --prepareTool

# Build and install aws-cpp-sdk-core
SDK_ROOT=$(pwd)
mkdir -p $SDK_ROOT/build/AWSSDK
mkdir -p $SDK_ROOT/install
cd $SDK_ROOT/build/AWSSDK
cmake $SDK_ROOT -DBUILD_ONLY="core" -DCMAKE_BUILD_TYPE=Debug -DCMAKE_INSTALL_PREFIX=$SDK_ROOT/install -DBUILD_SHARED_LIBS=ON
make -j 8
make install

# Build custom-service
mkdir -p $SDK_ROOT/build/custom-service
cd $SDK_ROOT/build/custom-service
cmake $SDK_ROOT/custom-service/aws-cpp-sdk-custom-service -DCMAKE_BUILD_TYPE=Debug -DCMAKE_PREFIX_PATH="$SDK_ROOT/install" -DAWSSDK_ROOT_DIR="$SDK_ROOT/install" -DBUILD_SHARED_LIBS=ON
make -j 8

# Build and run custom-service integration tests
mkdir -p $SDK_ROOT/build/custom-service-integration-tests
cd $SDK_ROOT/build/custom-service-integration-tests
cmake $SDK_ROOT/aws-cpp-sdk-custom-service-integration-tests -DCMAKE_BUILD_TYPE=Debug -DCMAKE_PREFIX_PATH="$SDK_ROOT/install;$SDK_ROOT/build/custom-service" -DAWSSDK_ROOT_DIR="$SDK_ROOT/install" -DBUILD_SHARED_LIBS=ON
make -j 8
$SDK_ROOT/build/custom-service-integration-tests/aws-cpp-sdk-custom-service-integration-tests
