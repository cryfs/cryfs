#include "CryDir.h"
#include "CryFile.h"
#include "CrySymlink.h"
#include <vector>

#include "OnDemandDevice.h"

using boost::none;

namespace cryfs {
OnDemandDevice::OnDemandDevice(bool delayMount,
                               bool onDemand,
                               std::function<cpputils::unique_ref<cryfs::CryDevice>()> device_creator_func)
    : _delayMount{delayMount}
    , _onDemand{onDemand}
    , _device{none}
    , _device_creator_func{std::move(device_creator_func)}
    , _onFsAction{}
{
    if (!_delayMount) {
        _device = boost::optional<cpputils::unique_ref<cryfs::CryDevice>>(_device_creator_func());
    }
}

OnDemandDevice::~OnDemandDevice() {
}

void OnDemandDevice::onFsAction(std::function<void()> callback) {
    _onFsAction.push_back(callback);

    if (_device) {
        _device->get()->onFsAction(callback);
    }
}

fspp::Device::statvfs OnDemandDevice::statfs() {
    CreateDevice();

    return _device->get()->statfs();
}

boost::optional<cpputils::unique_ref<fspp::Node>> OnDemandDevice::Load(const boost::filesystem::path &path) {
    CreateDevice();

    return (*_device)->Load(path);
}

boost::optional<cpputils::unique_ref<fspp::File>> OnDemandDevice::LoadFile(const boost::filesystem::path &path) {
    CreateDevice();

    return _device->get()->LoadFile(path);
}

boost::optional<cpputils::unique_ref<fspp::Dir>> OnDemandDevice::LoadDir(const boost::filesystem::path &path) {
    CreateDevice();

    return _device->get()->LoadDir(path);
}

boost::optional<cpputils::unique_ref<fspp::Symlink>> OnDemandDevice::LoadSymlink(const boost::filesystem::path &path) {
    CreateDevice();

    return _device->get()->LoadSymlink(path);
}

void OnDemandDevice::CreateDevice() {
    if (_device || (!_onDemand && !_delayMount))
        return;

    _device = boost::optional<cpputils::unique_ref<cryfs::CryDevice>>(_device_creator_func());

    for (const auto &callback : _onFsAction) {
        _device->get()->onFsAction(callback);
    }
}

void OnDemandDevice::DerefFileSystem() {
    if (!_onDemand) return;

    _device = none;
}

} // namespace cryfs
