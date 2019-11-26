#include "CryDir.h"
#include "CryFile.h"
#include "CrySymlink.h"
#include <vector>

#include "OnDemandDevice.h"

using boost::none;

namespace cryfs {
static
void dummy_restart() {
}

OnDemandDevice::OnDemandDevice(bool delayMount,
                               bool onDemand,
                               std::function<cpputils::unique_ref<cryfs::CryDevice>()> device_creator_func)
    : _delayMount{delayMount}
    , _onDemand{onDemand}
    , _device{none}
    , _device_creator_func{std::move(device_creator_func)}
    , _onFsAction{}
    , _mutex{}
    , _context_set{false}
    , _timer_restart_func{dummy_restart}
{
    if (!_delayMount) {
        _device = boost::optional<cpputils::unique_ref<cryfs::CryDevice>>(_device_creator_func());
    }
}

OnDemandDevice::~OnDemandDevice() {
}

void OnDemandDevice::onFsAction(std::function<void()> callback) {
    _onFsAction.push_back(callback);

    std::lock_guard<std::recursive_mutex> lock(_mutex);

    if (_device) {
        _device->get()->onFsAction(callback);
    }
}

fspp::Device::statvfs OnDemandDevice::statfs() {
    std::lock_guard<std::recursive_mutex> lock(_mutex);
    CreateDevice();

    return _device->get()->statfs();
}

boost::optional<cpputils::unique_ref<fspp::Node>> OnDemandDevice::Load(const boost::filesystem::path &path) {
    std::lock_guard<std::recursive_mutex> lock(_mutex);
    CreateDevice();

    return (*_device)->Load(path);
}

boost::optional<cpputils::unique_ref<fspp::File>> OnDemandDevice::LoadFile(const boost::filesystem::path &path) {
    std::lock_guard<std::recursive_mutex> lock(_mutex);
    CreateDevice();

    return _device->get()->LoadFile(path);
}

boost::optional<cpputils::unique_ref<fspp::Dir>> OnDemandDevice::LoadDir(const boost::filesystem::path &path) {
    std::lock_guard<std::recursive_mutex> lock(_mutex);
    CreateDevice();

    return _device->get()->LoadDir(path);
}

boost::optional<cpputils::unique_ref<fspp::Symlink>> OnDemandDevice::LoadSymlink(const boost::filesystem::path &path) {
    std::lock_guard<std::recursive_mutex> lock(_mutex);
    CreateDevice();

    return _device->get()->LoadSymlink(path);
}

void OnDemandDevice::CreateDevice() {
    if (_device || (!_onDemand && !_delayMount))
        return;

    _device = boost::optional<cpputils::unique_ref<cryfs::CryDevice>>(_device_creator_func());

    if (_context_set)
        (*_device)->setContext(fspp::Context {getContext()});

    for (const auto &callback : _onFsAction) {
        _device->get()->onFsAction(callback);
    }

    _timer_restart_func();
}

void OnDemandDevice::deref() {
    if (!_onDemand) return;

    std::lock_guard<std::recursive_mutex> lock(_mutex);
    _device = none;
}

void OnDemandDevice::setTimerRestartFunc(std::function<void()> func) {
    _timer_restart_func = std::move(func);
}

} // namespace cryfs
